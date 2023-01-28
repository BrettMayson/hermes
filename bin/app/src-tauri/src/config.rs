use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    arma3folder: Option<String>,
    communities: HashMap<Uuid, Community>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Community {
    name: String,
}
