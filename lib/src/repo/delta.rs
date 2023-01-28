use std::collections::HashMap;

use super::{Layer, Mod};

/// How has a repo changed between updates
#[derive(Debug, PartialEq, Eq)]
/// How has a mod changed between updates
pub enum ModDelta {
    /// A new mod has been added
    Added,
    /// An existing mod has changes
    Changed(HashMap<String, FileDelta>),
    /// A mod has been removed
    Removed,
    /// An existing mod has no changes
    Unchanged,
}

impl ModDelta {
    /// Compare two repos to find how they've changed
    pub fn new(old: &Mod, new: &Mod) -> Result<Self, String> {
        if old.hash() == new.hash() {
            Ok(Self::Unchanged)
        } else {
            let changed = check_layer(old.root(), new.root());
            if changed.is_empty() {
                println!("Hashes don't match, but no changes found. ({})", new.name());
                Ok(Self::Unchanged)
            } else {
                Ok(Self::Changed(changed))
            }
        }
    }
}

fn check_layer(old: &Layer, new: &Layer) -> HashMap<String, FileDelta> {
    let mut changed = HashMap::new();
    for file in old.files() {
        if let Some(nf) = new.files().iter().find(|nf| nf.name() == file.name()) {
            if nf.hash() != file.hash() {
                changed.insert(file.name().to_string(), FileDelta::GenericChanged);
            }
        } else {
            changed.insert(file.name().to_string(), FileDelta::Deleted);
        }
    }
    for file in new.files() {
        if !changed.contains_key(file.name()) {
            changed.insert(file.name().to_string(), FileDelta::New);
        }
    }
    for layer in old.layers() {
        if let Some(nl) = new.layers().iter().find(|nl| nl.name() == layer.name()) {
            for sub in check_layer(layer, nl) {
                changed.insert(format!("{}/{}", layer.name(), sub.0), sub.1);
            }
        } else {
            changed.insert(layer.name().to_string(), FileDelta::Deleted);
        }
    }
    for layer in new.layers() {
        if !changed.contains_key(layer.name()) {
            changed.insert(layer.name().to_string(), FileDelta::New);
        }
    }
    changed
}

#[derive(Debug, PartialEq, Eq)]
/// How has a file changed between updates
pub enum FileDelta {
    /// A file has been added to a mod
    New,
    /// A file has been removed from a mod
    Deleted,
    /// A generic file has been changed
    GenericChanged,
    // /// A PBO file has been changed
    // PboChanged,
}
