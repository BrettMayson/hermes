use std::{
    io::{BufReader, Read},
    path::PathBuf,
};

use hemtt_pbo::ReadablePbo;
use indexmap::IndexMap;
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};

use super::sha256_digest;

#[derive(Debug, Serialize, Deserialize)]
/// A file.
pub enum File {
    #[serde(rename = "g")]
    /// Any file that is not a PBO.
    Generic {
        #[serde(rename = "n")]
        /// The name of the file.
        name: String,
        #[serde(rename = "s")]
        /// The size of the file
        size: u64,
        #[serde(rename = "h")]
        /// The hash of the file.
        hash: Vec<u8>,
    },
    #[serde(rename = "p")]
    /// A PBO file.
    Pbo {
        #[serde(rename = "n")]
        /// The name of the file.
        name: String,
        #[serde(rename = "s")]
        /// The size of the file
        size: u64,
        #[serde(rename = "pr")]
        /// The extenstions of the file.
        props: IndexMap<String, String>,
        #[serde(rename = "pa")]
        /// The parts of the file.
        parts: Vec<Part>,
        #[serde(rename = "h")]
        /// The hash of the file.
        hash: Vec<u8>,
    },
}

impl File {
    #[must_use]
    /// Creates a new generic file.
    pub const fn new_generic(name: String, size: u64, hash: Vec<u8>) -> Self {
        Self::Generic { name, size, hash }
    }

    #[must_use]
    /// Creates a new PBO file.
    pub const fn new_pbo(
        name: String,
        size: u64,
        props: IndexMap<String, String>,
        parts: Vec<Part>,
        hash: Vec<u8>,
    ) -> Self {
        Self::Pbo {
            name,
            size,
            props,
            parts,
            hash,
        }
    }

    #[must_use]
    /// Gets the name of the file.
    pub fn name(&self) -> &str {
        match self {
            Self::Pbo { name, .. } | Self::Generic { name, .. } => name,
        }
    }

    #[must_use]
    /// Gets the hash of the file.
    pub fn hash(&self) -> &[u8] {
        match self {
            Self::Pbo { hash, .. } | Self::Generic { hash, .. } => hash,
        }
    }

    /// Create a file
    pub fn from(path: PathBuf) -> Result<Self, String> {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let path = if name != name.to_lowercase() {
            let lower_path = {
                let mut path = path.clone();
                path.set_file_name(name.to_lowercase());
                path
            };
            if let Err(e) = std::fs::rename(&path, &lower_path) {
                return Err(format!(
                    "Failed to rename `{}` to lowercase: {}",
                    path.display(),
                    e
                ));
            }
            lower_path
        } else {
            path
        };
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let input = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        let size = input.metadata().unwrap().len();
        if path.extension() == Some(std::ffi::OsStr::new("pbo")) {
            let mut pbo = ReadablePbo::from(BufReader::new(input)).unwrap();
            let mut parts = Vec::new();
            let mut hash = Context::new(&SHA256);
            for prop in pbo.properties() {
                hash.update(prop.0.as_bytes());
                hash.update(prop.1.as_bytes());
            }
            for file in pbo.files_sorted() {
                hash.update(file.filename().as_bytes());
                let mut reader = pbo.file(file.filename()).unwrap().unwrap();
                let mut buffer = [0; 1024];
                let mut file_hash = Context::new(&SHA256);
                loop {
                    let count = reader.read(&mut buffer).map_err(|e| e.to_string())?;
                    if count == 0 {
                        break;
                    }
                    file_hash.update(&buffer[..count]);
                }
                let file_hash = file_hash.finish().as_ref().to_vec();
                hash.update(&file_hash);
                parts.push(Part {
                    name: file.filename().to_string(),
                    hash: file_hash,
                    offset: pbo.file_offset(file.filename()).unwrap().unwrap(),
                })
            }
            Ok(Self::Pbo {
                name,
                size,
                props: pbo.properties().to_owned(),
                parts,
                hash: hash.finish().as_ref().to_vec(),
            })
        } else {
            let reader = BufReader::new(input);
            let hash = sha256_digest(reader)?.as_ref().to_vec();
            Ok(Self::Generic { name, size, hash })
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// A part of a PBO file.
pub struct Part {
    /// The name of the part.
    name: String,
    /// The hash of the part.
    hash: Vec<u8>,
    /// The offset in the PBO file.
    offset: u64,
}

impl Part {
    #[must_use]
    /// Creates a new part.
    pub const fn new(name: String, hash: Vec<u8>, offset: u64) -> Self {
        Self { name, hash, offset }
    }

    #[must_use]
    /// Gets the name of the part.
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Gets the hash of the part.
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    #[must_use]
    /// Gets the offset in the PBO file.
    pub const fn offset(&self) -> u64 {
        self.offset
    }
}
