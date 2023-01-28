use serde::{Deserialize, Serialize};

use super::DLC;

#[derive(Debug, Serialize, Deserialize)]
/// A pack of mods and DLCs.
pub struct Pack {
    #[serde(rename(serialize = "n"), alias = "n")]
    /// The name of the pack.
    name: String,
    #[serde(default, rename(serialize = "m"), alias = "m")]
    /// The mods in the pack.
    mods: Vec<String>,
    #[serde(default, rename(serialize = "d"), alias = "d")]
    /// DLCs in the pack.
    dlcs: Vec<DLC>,
}

impl Pack {
    #[must_use]
    /// Creates a new pack.
    pub const fn new(name: String, mods: Vec<String>, dlcs: Vec<DLC>) -> Self {
        Self { name, mods, dlcs }
    }

    #[must_use]
    /// Gets the name of the pack.
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Gets the mods in the pack.
    pub fn mods(&self) -> &[String] {
        &self.mods
    }

    #[must_use]
    /// Gets the DLCs in the pack.
    pub fn dlcs(&self) -> &[DLC] {
        &self.dlcs
    }
}
