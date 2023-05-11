use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::repo::{Pack, Server, Unit};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    unit: Unit,
    pack: HashMap<String, Pack>,
    server: HashMap<String, Server>,
}

impl Config {
    pub fn from_toml(source: &str) -> Result<Self, String> {
        Self::from_str(source)
    }

    pub fn into_parts(self) -> (Unit, HashMap<String, Pack>, HashMap<String, Server>) {
        (self.unit, self.pack, self.server)
    }

    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    pub fn pack(&self, name: &str) -> Option<&Pack> {
        self.pack.get(name)
    }

    pub fn packs(&self) -> impl Iterator<Item = &Pack> {
        self.pack.values()
    }

    pub fn server(&self, name: &str) -> Option<&Server> {
        self.server.get(name)
    }

    pub fn servers(&self) -> impl Iterator<Item = &Server> {
        self.server.values()
    }

    pub fn validate(&self) -> Result<(), String> {
        for server in self.server.values() {
            if !self.pack.keys().any(|pack| pack == server.pack()) {
                return Err(format!("Pack `{}` does not exist", server.pack()));
            }
        }
        for pack in self.pack.values() {
            for m in pack.mods() {
                if m.to_lowercase() != *m {
                    return Err(format!("Mod `{m}` must be lowercase"));
                }
            }
        }
        Ok(())
    }
}

impl FromStr for Config {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config: Self = toml::from_str(s).map_err(|e| e.to_string())?;
        config.validate()?;
        Ok(config)
    }
}
