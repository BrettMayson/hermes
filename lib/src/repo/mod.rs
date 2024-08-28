#![deny(clippy::all, clippy::nursery, missing_docs)]

//! hermes - Repository
//!
//! This library provides the repository format for hermes.

mod delta;
mod dlc;
mod file;
mod layer;
mod pack;
mod password;
mod server;
mod unit;

use std::{
    collections::{HashMap, HashSet},
    io::BufReader,
    path::PathBuf,
    sync::RwLock,
    time::SystemTime,
};

pub use delta::{FileDelta, ModDelta};
pub use dlc::DLC;
pub use file::File;
use indicatif::{ProgressBar, ProgressStyle};
pub use layer::Layer;
pub use pack::Pack;
pub use password::Password;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use ring::digest::{Context, Digest, SHA256};
use serde::{Deserialize, Serialize};
pub use server::Server;
pub use unit::Unit;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
/// A configuration file for a hermes repository.
pub struct Repository {
    #[serde(rename = "v")]
    /// Version of the repository
    version: u8,
    #[serde(rename = "u")]
    /// The name of the repository.
    unit: Unit,
    #[serde(rename = "m")]
    /// The mods in the repository.
    mods: Vec<Mod>,
    #[serde(rename = "p")]
    /// The packs in the repository.
    packs: HashMap<String, Pack>,
    #[serde(rename = "s")]
    /// The servers in the repository.
    servers: Vec<Server>,
    #[serde(rename = "t")]
    /// Generation time
    time: u64,
    #[serde(rename = "h")]
    /// Hash of all mods
    hash: Vec<u8>,
}

impl Repository {
    #[must_use]
    /// Creates a new repository.
    pub fn new(
        unit: Unit,
        mods: Vec<Mod>,
        packs: HashMap<String, Pack>,
        servers: Vec<Server>,
        time: u64,
    ) -> Self {
        let mut hash = Context::new(&SHA256);
        for m in &mods {
            hash.update(m.hash());
        }
        Self {
            version: 1,
            unit,
            mods,
            packs,
            servers,
            time,
            hash: hash.finish().as_ref().to_vec(),
        }
    }

    #[must_use]
    /// Repo spec version
    pub const fn version(&self) -> u8 {
        self.version
    }

    #[must_use]
    /// Gets the unit of the repository.
    pub const fn unit(&self) -> &Unit {
        &self.unit
    }

    #[must_use]
    /// Gets the mods in the repository.
    pub fn mods(&self) -> &[Mod] {
        &self.mods
    }

    #[must_use]
    /// Gets the packs in the repository.
    pub const fn packs(&self) -> &HashMap<String, Pack> {
        &self.packs
    }

    #[must_use]
    /// Gets the servers in the repository.
    pub fn servers(&self) -> &[Server] {
        &self.servers
    }

    #[must_use]
    /// Gets the generation time.
    pub const fn time(&self) -> u64 {
        self.time
    }

    #[must_use]
    /// Get the hash of the repo
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    /// Create a blob for sending the Repo over the internet
    ///
    /// Format:
    /// 0: Version
    /// 1-32: Sha256 Hash
    /// 33..: MessagePack serialized
    pub fn to_blob(&self) -> Vec<u8> {
        let mut buf = vec![self.version];
        buf.extend_from_slice(&self.hash);
        self.serialize(&mut rmp_serde::Serializer::new(&mut buf))
            .unwrap();
        buf
    }

    /// Read a repo from a MessagePack blob
    pub fn from_blob(source: &[u8]) -> Result<Self, String> {
        let version = source[0];
        if version != 1 {
            return Err(format!("Unsupported Version: {version}"));
        }
        let read = BufReader::new(&source[33..]);
        Self::deserialize(&mut rmp_serde::Deserializer::new(read)).map_err(|e| e.to_string())
    }

    /// Create a repo from a config
    pub fn from_config(config: Config) -> Result<Self, String> {
        let mut mods_to_scan = Vec::with_capacity(60);
        for pack in config.packs() {
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
        #[allow(clippy::significant_drop_tightening)]
        // I believe this is a false positive, check later when this is out of the nursery
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
        let (unit, pack, server) = config.into_parts();
        Ok(Self::new(
            unit,
            mods.into_inner().unwrap(),
            pack,
            server.into_values().collect::<Vec<_>>(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// A mod.
pub struct Mod {
    /// The name of the mod.
    name: String,
    /// The root layer
    root: Layer,
}

impl Mod {
    #[must_use]
    /// Creates a new mod.
    pub const fn new(name: String, root: Layer) -> Self {
        Self { name, root }
    }

    #[must_use]
    /// Gets the name of the mod.
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Gets the root layer.
    pub const fn root(&self) -> &Layer {
        &self.root
    }

    /// Get the hash of the root layer
    pub fn hash(&self) -> &[u8] {
        self.root.hash()
    }

    /// Create a mod from a folder
    pub fn from_folder(name: &str) -> Result<Self, String> {
        let path = PathBuf::from(name);
        if !path.exists() {
            return Err(format!("No mod folder `{name}`"));
        }
        let root = Layer::from_folder(path)?;
        Ok(Self {
            name: name.to_string(),
            root,
        })
    }
}

fn sha256_digest<R: std::io::Read>(mut reader: R) -> Result<Digest, String> {
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}
