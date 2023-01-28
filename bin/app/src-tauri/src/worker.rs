use std::sync::mpsc::{Receiver, Sender};

use crate::broker::WebEvent;

pub fn run(rx: Receiver<Command>, tx: Sender<WebEvent>) {
    loop {
        let Ok(ev) = rx.recv() else {
            break;
        };
        match ev {
            Command::Awake => {
                tx.send(WebEvent::AskArma3Folder);
            }
            Command::Log(l) => println!("LOG {}", l),
            Command::Arma3Folder(response) => match response {
                Arma3FolderResponse::Ok(p) => {
                    println!("Path: {}", p);
                }
                Arma3FolderResponse::Cancel => {
                    println!("Cancelled");
                }
            },
        }
    }
}

pub enum Command {
    /// The webview is alive
    Awake,
    Log(String),
    Arma3Folder(Arma3FolderResponse),
}

pub enum Arma3FolderResponse {
    Ok(String),
    Cancel,
}
