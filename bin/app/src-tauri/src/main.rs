#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::path::PathBuf;

mod broker;
mod config;
mod worker;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let (rx, tx) = broker::setup(app);
            std::thread::spawn(move || {
                worker::run(rx, tx);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn data_dir() -> PathBuf {
    dirs::data_local_dir()
        .expect("No local data dir")
        .join("harmony-arma")
}
