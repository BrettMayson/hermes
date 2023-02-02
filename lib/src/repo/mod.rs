#![deny(clippy::all, clippy::nursery, missing_docs)]

//! Harmony - Repository
//!
//! This library provides the repository format for Harmony.

mod delta;
mod dlc;
mod file;
mod layer;
mod pack;
mod password;
mod server;
mod unit;

use std::{collections::HashMap, io::BufReader, path::PathBuf};

pub use delta::{FileDelta, ModDelta};
pub use dlc::DLC;
pub use file::File;
pub use layer::Layer;
pub use pack::Pack;
pub use password::Password;
use ring::digest::{Context, Digest, SHA256};
use serde::{Deserialize, Serialize};
pub use server::Server;
pub use unit::Unit;

#[derive(Debug, Serialize, Deserialize)]
/// A configuration file for a Harmony repository.
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
