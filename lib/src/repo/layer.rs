use std::path::PathBuf;

use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};

use super::File;

#[derive(Debug, Serialize, Deserialize)]
/// A layer of a mod. Basically a directory.
pub struct Layer {
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "f")]
    /// The files in the layer.
    files: Vec<File>,
    #[serde(rename = "l")]
    /// The sublayers in the layer.
    layers: Vec<Layer>,
    #[serde(rename = "h")]
    /// Hash
    hash: Vec<u8>,
}

impl Layer {
    #[must_use]
    /// Creates a new layer.
    pub fn new(name: String, files: Vec<File>, layers: Vec<Self>) -> Self {
        let mut hash = Context::new(&SHA256);
        for file in &files {
            hash.update(file.name().as_bytes());
            hash.update(file.hash());
        }
        for layer in &layers {
            hash.update(layer.name().as_bytes());
            hash.update(layer.hash());
        }
        Self {
            name,
            files,
            layers,
            hash: hash.finish().as_ref().to_vec(),
        }
    }

    #[must_use]
    /// Gets the name of the layer
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Gets the files in the layer.
    pub fn files(&self) -> &[File] {
        &self.files
    }

    #[must_use]
    /// Gets the sublayers in the layer.
    pub fn layers(&self) -> &[Self] {
        &self.layers
    }

    #[must_use]
    /// Gets the hash of the layer
    /// Made up of the hash of all files and all layers
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    /// Create a layer from a folder
    pub fn from_folder(path: PathBuf) -> Result<Self, String> {
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
        let entries = std::fs::read_dir(&path);
        let Ok(entries) = entries else {
            return Err(format!("Failed to read_dir on `{}`", path.display()));
        };
        let mut layers = Vec::new();
        let mut files = Vec::new();
        for entry in entries {
            let Ok(entry) = entry else {
                return Err(format!("Invalid entry: {:?}", entry));
            };
            if entry
                .file_type()
                .expect("Failed to determine file type")
                .is_dir()
            {
                layers.push(Self::from_folder(entry.path())?);
            } else {
                files.push(File::from(entry.path())?);
            }
        }

        Ok(Self::new(
            path.file_name().unwrap().to_str().unwrap().to_string(),
            files,
            layers,
        ))
    }
}
