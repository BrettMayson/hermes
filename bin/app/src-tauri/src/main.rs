#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

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

// pub fn config_dir() -> PathBuf {
//     dirs::config_dir().unwrap().join("arma-harmony")
// }

// pub fn find_arma() -> PathBuf {
//     let Some(arma3dir) = SteamDir::locate().and_then(|mut s| s.app(&107_410).map(std::borrow::ToOwned::to_owned)) else {
//         return Err(Error::Arma3NotFound);
//     };
// }
