use std::collections::HashMap;

use indexmap::IndexMap;

use super::{file::Part, File, Layer, Mod};

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
    /// Compare two mods to find how they've changed
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
                let File::Pbo {
                    props,
                    parts: new_parts,
                    ..
                } = nf
                else {
                    changed.insert(file.name().to_string(), FileDelta::GenericChanged);
                    continue;
                };
                let File::Pbo {
                    parts: old_parts, ..
                } = file
                else {
                    changed.insert(file.name().to_string(), FileDelta::GenericChanged);
                    continue;
                };
                let mut diff_changed = Vec::new();
                let mut diff_added = Vec::new();
                let mut diff_removed = Vec::new();
                for part in old_parts {
                    new_parts
                        .iter()
                        .find(|np| np.name() == part.name())
                        .map_or_else(
                            || {
                                diff_removed.push(part.name().to_string());
                            },
                            |np| {
                                if np.hash() != part.hash() {
                                    diff_changed.push(part.to_owned());
                                }
                            },
                        )
                }
                for part in new_parts {
                    if !old_parts.iter().any(|op| op.name() == part.name()) {
                        diff_added.push(part.to_owned());
                    }
                }
                changed.insert(
                    file.name().to_string(),
                    FileDelta::PboChanged {
                        props: props.clone(),
                        changed: diff_changed,
                        added: diff_added,
                        removed: diff_removed,
                    },
                );
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
    /// A PBO file has been changed
    PboChanged {
        /// The props in the new PBO file
        props: IndexMap<String, String>,
        /// The PBO file has been changed
        changed: Vec<Part>,
        /// The PBO file has been added
        added: Vec<Part>,
        /// The PBO file has been removed
        removed: Vec<String>,
    },
}
