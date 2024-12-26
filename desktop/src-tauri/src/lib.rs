mod setup;

use tauri::{Emitter, Window};

// init a background process on the command, and emit periodic events only to the window that used the command
#[tauri::command]
fn init_process(window: Window) {
    if window.title().unwrap().to_lowercase() != "hermes" {
        println!(
            "ignoring init_process for window '{}'",
            window.title().unwrap()
        );
        return;
    }
    std::thread::spawn(move || {
        window.emit("connected", ()).unwrap();
        setup::setup(window);
    });
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![init_process])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
