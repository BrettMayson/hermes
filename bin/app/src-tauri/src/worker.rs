use std::sync::mpsc::{Receiver, Sender};

use crate::{broker::WebEvent, config::Config};

pub fn run(rx: Receiver<Command>, tx: Sender<WebEvent>) {
    loop {
        let Ok(ev) = rx.recv() else {
            break;
        };
        match ev {
            Command::Awake => {
                println!("AWAKE");
                let (first_time, config) = Config::load();
                tx.send(WebEvent::RootConfigLoad((
                    first_time,
                    config.root().to_owned(),
                )))
                .unwrap();
            }
            Command::Log(l) => println!("LOG {}", l),
        }
    }
}

pub enum Command {
    /// The webview is alive
    Awake,
    Log(String),
}
