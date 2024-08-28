use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    LinuxNative,
    LinuxFlatpak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setup {
    pub arma_3_location: Option<PathBuf>,
    pub platform: Platform,
}
