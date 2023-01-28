use std::sync::mpsc::{self, Receiver, Sender};

use tauri::{App, Manager, Wry};

use crate::worker::{Arma3FolderResponse, Command};

pub fn setup(app: &mut App<Wry>) -> (Receiver<Command>, Sender<WebEvent>) {
    let (tx, rx) = mpsc::channel();
    let (wtx, wrx) = mpsc::channel();

    let txs = tx.clone();
    app.listen_global("global:log", move |event| {
        if let Some(payload) = event.payload() {
            txs.send(Command::Log(payload.to_string()));
        } else {
            println!("ERR log missing payload");
        }
    });

    let txs = tx.clone();
    app.listen_global("global:awake", move |_event| {
        txs.send(Command::Awake);
    });

    // global:arma3folder
    let txs = tx.clone();
    app.listen_global("global:arma3folder:ok", move |event| {
        txs.send(Command::Arma3Folder(Arma3FolderResponse::Ok(
            event.payload().unwrap().to_string(),
        )));
    });
    let txs = tx.clone();
    app.listen_global("global:arma3folder:cancel", move |event| {
        txs.send(Command::Arma3Folder(Arma3FolderResponse::Cancel));
    });

    let handle = app.handle();
    std::thread::spawn(move || loop {
        let Ok(req) = wrx.recv() else {
                break;
            };
        match req {
            WebEvent::AskArma3Folder => {
                println!("EMIT global:arma3folder");
                handle.emit_all("global:arma3folder", "").unwrap()
            }
        }
    });

    (rx, wtx)
}

pub enum WebEvent {
    AskArma3Folder,
}
