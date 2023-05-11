use std::time::Duration;

use reqwest::{Client, ClientBuilder};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    watch,
};

use super::pool::DownloadKey;

pub struct Worker {
    /// Worker ID, used in update messages.
    id: u8,

    client: Client,
    /// Channel to send updates to the pool.
    update: Sender<Update>,
    /// Channel to receive commands from the pool.
    command: Receiver<Command>,

    /// Current download speed limit.
    rate_limit: watch::Receiver<Option<u64>>,
}

impl Worker {
    pub fn new(
        id: u8,
        update: Sender<Update>,
        command: Receiver<Command>,
        rate_limit: watch::Receiver<Option<u64>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            id,
            client: ClientBuilder::new()
                .timeout(Duration::from_secs(60 * 60))
                .tcp_nodelay(true)
                .build()?,
            update,
            command,

            rate_limit,
        })
    }

    pub async fn run(&mut self) {
        while let Some(command) = self.command.recv().await {
            match command {
                Command::Download(key) => {
                    let mut output = Vec::new();
                    let mut response = {
                        let mut req = self.client.get(key.url());
                        if let Some((start, end)) = key.range() {
                            req = req.header("Range", format!("bytes={}-{}", start, end));
                        }
                        req.send().await.unwrap()
                    };

                    let total = response.content_length().unwrap_or(0);
                    let mut downloaded = 0u64;
                    let mut last_update = std::time::Instant::now();
                    let mut last_sleep = std::time::Instant::now();
                    let mut last_sleep_downloaded_since = 0u64;
                    let mut last_downloaded = 0;

                    while let Some(chunk) = response.chunk().await.unwrap() {
                        downloaded += chunk.len() as u64;
                        output.extend_from_slice(&chunk);

                        if last_update.elapsed() > Duration::from_millis(500) {
                            let speed = (downloaded - last_downloaded) as f64
                                / last_update.elapsed().as_secs_f64();
                            last_downloaded = downloaded;
                            last_update = std::time::Instant::now();

                            let _ = self
                                .update
                                .send(Update::Progress {
                                    id: self.id,
                                    key: key.clone(),
                                    downloaded,
                                    total,
                                    speed,
                                })
                                .await;
                        }

                        if let Some((_, end)) = key.range() {
                            if downloaded >= end {
                                break;
                            }
                        }

                        let rate_limit = *self.rate_limit.borrow_and_update();
                        if let Some(speed_limit) = rate_limit {
                            let required_time =
                                (downloaded - last_sleep_downloaded_since) * 1000 / speed_limit;
                            let sleep_time = Duration::from_millis(required_time);
                            if required_time > 100 && last_sleep.elapsed() < sleep_time {
                                tokio::time::sleep(sleep_time - last_sleep.elapsed()).await;
                                last_sleep = std::time::Instant::now();
                                last_sleep_downloaded_since = downloaded;
                            }
                        }
                    }

                    self.update
                        .send(Update::Done(self.id, key, output))
                        .await
                        .unwrap();
                }
                Command::Stop => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Update {
    Progress {
        id: u8,
        key: DownloadKey,
        downloaded: u64,
        total: u64,
        speed: f64,
    },
    Done(u8, DownloadKey, Vec<u8>),
}

impl Update {
    pub fn id(&self) -> u8 {
        match self {
            Self::Progress { id, .. } => *id,
            Self::Done(id, _, _) => *id,
        }
    }

    pub fn key(&self) -> &DownloadKey {
        match self {
            Self::Progress { key, .. } => key,
            Self::Done(_, key, _) => key,
        }
    }

    pub fn url(&self) -> &str {
        match self {
            Self::Progress { key, .. } => key.url(),
            Self::Done(_, key, _) => key.url(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    /// Download a blob
    Download(DownloadKey),
    /// Stop the worker
    Stop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download() {
        let (update_tx, mut update_rx) = tokio::sync::mpsc::channel(1);
        let (command_tx, command_rx) = tokio::sync::mpsc::channel(1);
        let (rate_limit_tx, rate_limit_rx) = tokio::sync::watch::channel(None);
        let mut worker = Worker::new(0, update_tx, command_rx, rate_limit_rx).unwrap();

        let url = String::from("https://images.unsplash.com/photo-1650409476524-7eb2f71b6cc8");

        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        const RATE_LIMIT: u64 = 1024 * 1024;

        println!(
            "Downloading at {} bytes/s",
            human_bytes::human_bytes(RATE_LIMIT as f64)
        );

        rate_limit_tx.send(Some(RATE_LIMIT)).unwrap();

        command_tx
            .send(Command::Download(DownloadKey::new(url, None)))
            .await
            .unwrap();

        while let Some(update) = update_rx.recv().await {
            match update {
                Update::Progress {
                    id: _,
                    key: _,
                    downloaded,
                    total,
                    speed,
                } => {
                    println!(
                        "Progress: {}% at {}/s",
                        downloaded as f64 / total as f64 * 100.0,
                        human_bytes::human_bytes(speed)
                    );
                }
                Update::Done(_, _, _) => {
                    command_tx.send(Command::Stop).await.unwrap();
                    break;
                }
            }
        }

        handle.await.unwrap();
    }
}
