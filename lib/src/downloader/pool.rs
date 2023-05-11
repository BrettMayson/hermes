use std::{
    hash::Hash,
    sync::{
        atomic::{AtomicU64, AtomicU8},
        Arc,
    },
};

use tokio::sync::{
    mpsc::{Receiver, Sender},
    RwLock,
};

use super::worker::{Command, Update, Worker};

type Workers = RwLock<Vec<(u8, Sender<Command>)>>;
type Pending = RwLock<Vec<(DownloadKey, Sender<Update>)>>;
type Subscribers = RwLock<Vec<(DownloadKey, Sender<Update>)>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DownloadKey {
    url: String,
    range: Option<(u64, u64)>,
}
impl DownloadKey {
    pub fn new(url: String, range: Option<(u64, u64)>) -> Self {
        Self { url, range }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn range(&self) -> Option<(u64, u64)> {
        self.range
    }
}

pub struct DownloadPool {
    /// The maximum number of concurrent downloads.
    max_concurrent: AtomicU8,
    /// The current number of concurrent downloads.
    current_concurrent: Arc<AtomicU8>,

    /// The maximum total download speed in bytes per second.
    rate_limit: Arc<AtomicU64>,
    rate_limit_watch: Arc<tokio::sync::watch::Sender<Option<u64>>>,

    /// The current worker threads.
    workers: Arc<Workers>,

    /// Pending downloads, waiting for a free worker.
    pending: Arc<Pending>,

    /// Subscribers to the updates.
    broadcast: tokio::sync::broadcast::Sender<Event>,

    /// Subscribers to specific downloads.
    subscribers: Arc<Subscribers>,

    global: Sender<Update>,
}

impl DownloadPool {
    /// A new download pool.
    // Async just to be able to use tokio::spawn.
    pub async fn new(max_concurrent: u8, rate_limit: Option<u64>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Update>(10);
        let current_concurrent = Arc::new(AtomicU8::new(0));
        let pending: Arc<Pending> = Arc::new(RwLock::new(Vec::new()));
        let workers: Arc<Workers> = Arc::new(RwLock::new(Vec::new()));
        let broadcast = tokio::sync::broadcast::channel(10).0;
        let subscribers: Arc<Subscribers> = Arc::new(RwLock::new(Vec::new()));
        let rate_limit_watch = Arc::new(tokio::sync::watch::channel(rate_limit).0);
        let rate_limit = Arc::new(AtomicU64::new(rate_limit.unwrap_or(0)));
        {
            let current_concurrent = current_concurrent.clone();
            let pending = pending.clone();
            let workers = workers.clone();
            let broadcast = broadcast.clone();
            let subscribers = subscribers.clone();
            let rate_limit_watch = rate_limit_watch.clone();
            let rate_limit = rate_limit.clone();
            tokio::spawn(async move {
                loop {
                    let update = rx.recv().await.unwrap();
                    let _ = broadcast.send(Event::WorkerUpdate(update.clone()));
                    for (key, tx) in subscribers.read().await.iter() {
                        if key == update.key() {
                            let _ = tx.send(update.clone()).await;
                        }
                    }
                    if let Update::Done(id, _, _) = update {
                        // If there are pending downloads, send one to the response worker.
                        if let Some((key, tx)) = { pending.write().await.pop() } {
                            // Register the subscriber.
                            subscribers.write().await.push((key.clone(), tx.clone()));
                            // Send the download to the worker.
                            workers
                                .read()
                                .await
                                .iter()
                                .find(|(wid, _)| *wid == id)
                                .unwrap()
                                .1
                                .send(Command::Download(key))
                                .await
                                .unwrap();
                        } else {
                            // Otherwise, remove the worker.
                            workers
                                .read()
                                .await
                                .iter()
                                .find(|(wid, _)| *wid == id)
                                .unwrap()
                                .1
                                .send(Command::Stop)
                                .await
                                .unwrap();
                            workers.write().await.retain(|(wid, _)| *wid != id);
                            current_concurrent.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                            Self::workers_set_rate_limit(
                                &rate_limit_watch,
                                &rate_limit,
                                &current_concurrent,
                            )
                            .await;
                            let _ = broadcast.send(Event::WorkerRemoved(id));
                        }
                    }
                }
            });
        }
        Self {
            max_concurrent: AtomicU8::new(max_concurrent),
            current_concurrent,
            rate_limit,
            rate_limit_watch,
            workers,
            pending,
            broadcast,
            subscribers,
            global: tx,
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.broadcast.subscribe()
    }

    pub fn set_max_concurrent(&self, max_concurrent: u8) {
        self.max_concurrent
            .store(max_concurrent, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn max_concurrent(&self) -> u8 {
        self.max_concurrent
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn current_concurrent(&self) -> u8 {
        self.current_concurrent
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn set_rate_limit(&self, rate_limit: Option<u64>) {
        self.rate_limit.store(
            rate_limit.unwrap_or(0),
            std::sync::atomic::Ordering::Relaxed,
        );
        Self::workers_set_rate_limit(
            &self.rate_limit_watch,
            &self.rate_limit,
            &self.current_concurrent,
        )
        .await;
    }

    pub fn rate_limit(&self) -> Option<u64> {
        if self.rate_limit.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            None
        } else {
            Some(self.rate_limit.load(std::sync::atomic::Ordering::Relaxed))
        }
    }

    pub async fn download(&self, key: DownloadKey) -> Receiver<Update> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        // If the URL is already in the subscribers list, add a new subscriber and return.
        let mut subscribers = self.subscribers.write().await;
        for (key2, _) in subscribers.iter() {
            if key == *key2 {
                subscribers.push((key.clone(), tx.clone()));
                return rx;
            }
        }

        // Create a new worker if we are not at the maximum number of concurrent downloads.
        let mut workers = self.workers.write().await;
        if workers.len()
            < self
                .max_concurrent
                .load(std::sync::atomic::Ordering::Relaxed) as usize
        {
            let id = (0..=u8::MAX)
                .find(|id| !workers.iter().any(|(id2, _)| id == id2))
                .unwrap();
            let (command_tx, command_rx) = tokio::sync::mpsc::channel(3);
            let update_tx = self.global.clone();
            let mut worker =
                Worker::new(id, update_tx, command_rx, self.rate_limit_watch.subscribe()).unwrap();
            tokio::spawn(async move {
                worker.run().await;
            });
            workers.push((id, command_tx));
            subscribers.push((key.clone(), tx.clone()));
            workers
                .iter()
                .find(|(wid, _)| *wid == id)
                .unwrap()
                .1
                .send(Command::Download(key))
                .await
                .unwrap();
            self.current_concurrent
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            drop(workers);
            Self::workers_set_rate_limit(
                &self.rate_limit_watch,
                &self.rate_limit,
                &self.current_concurrent,
            )
            .await;
            let _ = self.broadcast.send(Event::WorkerAdded(id));
        } else {
            // Otherwise, add the download to the pending list.
            self.pending.write().await.push((key, tx.clone()));
        }

        rx
    }

    async fn workers_set_rate_limit(
        rate_limit_watch: &tokio::sync::watch::Sender<Option<u64>>,
        rate_limit: &AtomicU64,
        current_concurrent: &AtomicU8,
    ) -> Option<u64> {
        let max = rate_limit.load(std::sync::atomic::Ordering::Relaxed);
        let current = current_concurrent.load(std::sync::atomic::Ordering::Relaxed) as u64;
        let max = if max == 0 {
            None
        } else if current == 0 {
            Some(max)
        } else {
            Some(max / current)
        };
        rate_limit_watch.send(max).unwrap();
        max
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    WorkerUpdate(Update),
    WorkerAdded(u8),
    WorkerRemoved(u8),
}
