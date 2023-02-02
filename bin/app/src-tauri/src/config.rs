use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use steamlocate::SteamDir;
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    root: RootConfig,
    communities: HashMap<Uuid, Community>,
}

impl Config {
    #[must_use]
    /// Creates a new config.
    pub fn new(root: RootConfig, communities: HashMap<Uuid, Community>) -> Self {
        Self { root, communities }
    }

    #[must_use]
    /// Load the config from the user data directory.
    /// If the config does not exist, it will be created.
    /// Use rmp_serde to serialize and deserialize.
    pub fn load() -> (bool, Self) {
        let config_path = crate::data_dir().join("harmony.config");
        if !config_path.exists() {
            let mut config = Self::default();
            if let Some(arma3) = find_arma() {
                println!("Found Arma 3 at {:?}", arma3);
                config.root.arma3folder = Some(arma3);
            } else {
                println!("Could not find Arma 3");
            }
            (true, config)
        } else {
            let config = std::fs::read(&config_path).expect("Failed to read config file");
            (
                false,
                rmp_serde::from_slice(&config).expect("Failed to deserialize config"),
            )
        }
    }

    #[must_use]
    /// Gets the root config.
    pub fn root(&self) -> &RootConfig {
        &self.root
    }

    #[must_use]
    /// Gets the communities.
    pub fn communities(&self) -> &HashMap<Uuid, Community> {
        &self.communities
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RootConfig {
    arma3folder: Option<PathBuf>,
    depotfolder: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Community {
    name: String,
}

pub fn find_arma() -> Option<PathBuf> {
    SteamDir::locate()
        .and_then(|mut s| s.app(&107_410).map(std::borrow::ToOwned::to_owned))
        .map(|s| s.path)
}
