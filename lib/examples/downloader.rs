use std::collections::HashMap;

use indicatif::{MultiProgress, ProgressBar};
use syncra::downloader::{DownloadKey, DownloadPool, Event, Update};

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://images.unsplash.com/photo-1650409476524-7eb2f71b6cc8",
        "https://images.unsplash.com/photo-1660101077376-08ffebd701f4",
        "https://images.unsplash.com/photo-1664262900842-45eb7101f131",
        "https://images.unsplash.com/photo-1649153512379-c86b9d06739d",
        "https://images.unsplash.com/photo-1643666183207-075183a0e3be",
        "https://images.unsplash.com/photo-1640462709185-77f39fc4a91f",
        "https://images.unsplash.com/photo-1640462709185-77f39fc4a91f",
        "https://images.unsplash.com/photo-1640229318581-8873d30a10a9",
        "https://images.unsplash.com/photo-1640299438750-a9a4a39d397e",
        "https://images.unsplash.com/photo-1640185623400-dc00cfacd1a5",
    ];

    let pool = DownloadPool::new(4, Some(1024 * 1024 * 3)).await;

    let mpb = MultiProgress::new();
    let mut pbs: HashMap<u8, ProgressBar> = HashMap::new();

    let mut sub = pool.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = sub.recv().await {
            match event {
                Event::WorkerUpdate(update) => match update {
                    Update::Progress {
                        id,
                        key,
                        downloaded,
                        total,
                        speed,
                    } => {
                        pbs[&id].set_message(format!(
                            "{} {}",
                            human_bytes::human_bytes(speed),
                            key.url()
                        ));
                        pbs[&id].set_position(downloaded);
                        pbs[&id].set_length(total);
                    }
                    Update::Done(id, key, _) => {
                        pbs[&id].println(format!("Done: {}", key.url()));
                        pbs[&id].finish();
                    }
                },
                Event::WorkerAdded(id) => {
                    let pb = ProgressBar::new(100);
                    pb.set_style(
                        indicatif::ProgressStyle::default_bar()
                            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) @ {msg}")
                            .unwrap()
                            .progress_chars("#>-"),
                    );
                    mpb.add(pb.clone());
                    pbs.insert(id, pb);
                }
                Event::WorkerRemoved(id) => {
                    if let Some(pb) = pbs.remove(&id) {
                        pb.finish_and_clear();
                        mpb.remove(&pb);
                    }
                }
            }
        }
    });

    for url in urls {
        pool.download(DownloadKey::new(url.to_string(), None)).await;
    }

    while pool.current_concurrent() != 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
