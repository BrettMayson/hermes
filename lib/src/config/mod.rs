use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::RwLock,
    time::SystemTime,
};

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::repo::{Mod, Pack, Repository, Server, Unit};

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

    pub fn unit(&self) -> &Unit {
        &self.unit
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

impl TryInto<Repository> for Config {
    type Error = String;

    fn try_into(self) -> Result<Repository, Self::Error> {
        let mut mods_to_scan = Vec::with_capacity(60);
        for pack in self.pack.values() {
            println!("Collecting Pack: {}", pack.name());
            let mut pack_mods = Vec::new();
            for m in pack.mods() {
                if m == "*" {
                    for entry in std::fs::read_dir(".").expect("Failed to list directory for *") {
                        let entry = entry.unwrap();
                        if entry.file_type().unwrap().is_dir() {
                            let name = entry.file_name().into_string().unwrap();
                            if name.starts_with('@') && !pack_mods.contains(&name) {
                                pack_mods.push(name);
                            }
                        }
                    }
                } else if m.starts_with('-') {
                    let name = m.trim_start_matches('-');
                    if name.starts_with('@') {
                        if let Some(idx) = pack_mods.iter().position(|i| i == name) {
                            pack_mods.remove(idx);
                        }
                    }
                } else {
                    pack_mods.push(m.to_string());
                }
            }
            for m in pack_mods {
                if !mods_to_scan.contains(&m) {
                    mods_to_scan.push(m.to_string());
                }
            }
        }
        let style =
            ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}").unwrap();
        let pb = ProgressBar::new(mods_to_scan.len() as u64).with_style(style);
        let mods = RwLock::new(Vec::new());
        let active = RwLock::new(HashSet::new());
        mods_to_scan.par_iter().for_each(|m| {
            active.write().unwrap().insert(m);
            pb.set_message(
                (active
                    .read()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>())
                .join(","),
            );
            let obj = Mod::from_folder(m).unwrap();
            mods.write().unwrap().push(obj);
            active.write().unwrap().remove(m);
            pb.set_message(
                (active
                    .read()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>())
                .join(","),
            );
            pb.inc(1);
        });
        pb.finish();
        Ok(Repository::new(
            self.unit,
            mods.into_inner().unwrap(),
            self.pack,
            self.server.into_values().collect::<Vec<_>>(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))
    }
}
