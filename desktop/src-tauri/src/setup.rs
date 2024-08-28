use futures::StreamExt;
use steamlocate::SteamDir;
use hermes_desktop_comm::setup::{Platform, Setup};
use tauri::{async_runtime::spawn, Emitter, Window};

pub fn setup(window: Window) {
    let theme_window = window.clone();
    spawn(async move {
        fn send (window: &Window, mode: dark_light::Mode) {
            window.emit("theme", if mode == dark_light::Mode::Dark { "dark" } else { "light" }).unwrap();
        }
        let mode = dark_light::detect();
        send(&theme_window, mode);
        while let Some(mode) = dark_light::subscribe().await.unwrap().next().await {
            send(&theme_window, mode);
        }
    });
    
    let setup = Setup {
        arma_3_location: arma_3_location(),
        platform: platform(),
    };
    window.emit("setup", setup).unwrap();
}

fn arma_3_location() -> Option<std::path::PathBuf> {
    let Ok(Some((app, library))) = SteamDir::locate().and_then(|s| s.find_app(107_410)) else {
        return None;
    };
    Some(library.resolve_app_dir(&app))
}

fn platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "linux") {
        let flatpak = std::process::Command::new("flatpak")
            .arg("list")
            .arg("--app")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("com.valvesoftware.Steam")).unwrap_or_default();
        if flatpak {
            Platform::LinuxFlatpak
        } else {
            Platform::LinuxNative
        }
    } else {
        panic!("unsupported platform");
    }
}
