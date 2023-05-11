use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// The Unit the pack is for
pub struct Unit {
    name: String,
    id: Option<String>,
}

impl Unit {
    #[must_use]
    /// Get the unit name
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Get the unit id
    pub const fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }
}
