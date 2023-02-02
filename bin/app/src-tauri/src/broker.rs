use std::sync::mpsc::{self, Receiver, Sender};

use tauri::{App, Manager, Wry};

use crate::{worker::Command, config::RootConfig};

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

    let txs = tx;
    app.listen_global("global:awake", move |_event| {
        txs.send(Command::Awake);
    });

    let handle = app.handle();
    std::thread::spawn(move || loop {
        let Ok(req) = wrx.recv() else {
                break;
            };
        match req {
            WebEvent::RootConfigLoad(rc) => {
                println!("EMIT global:RootConfigLoad");
                handle.emit_all("global:RootConfigLoad", &rc).unwrap()
            }
            WebEvent::RootConfigSave(_) => todo!(),
        }
    });

    (rx, wtx)
}

pub enum WebEvent {
    RootConfigLoad((bool, RootConfig)),
    RootConfigSave(Result<(), String>),
}
