//! Provides configuration for a server.

use serde::{Deserialize, Serialize};

use super::Password;

#[derive(Debug, Serialize, Deserialize)]
/// An Arma 3 server.
pub struct Server {
    #[serde(rename(serialize = "n"), alias = "n")]
    /// The name of the server.
    name: String,
    #[serde(rename(serialize = "a"), alias = "a")]
    /// The address of the server.
    address: String,
    #[serde(default = "default_port", rename(serialize = "po"), alias = "po")]
    /// The port of the server.
    port: u16,
    #[serde(rename(serialize = "ps"), alias = "ps")]
    /// The password of the server.
    password: Password,
    #[serde(rename(serialize = "pk"), alias = "pk")]
    /// The pack used by the server.
    pack: String,
    #[serde(default = "default_battleye", rename(serialize = "b"), alias = "b")]
    /// Battleye enabled
    battleye: bool,
}

impl Server {
    #[must_use]
    /// Creates a new server.
    pub const fn new(
        name: String,
        address: String,
        port: u16,
        password: Password,
        pack: String,
        battleye: bool,
    ) -> Self {
        Self {
            name,
            address,
            port,
            password,
            pack,
            battleye,
        }
    }

    #[must_use]
    /// Gets the name of the server.
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    /// Gets the address of the server.
    pub const fn address(&self) -> &String {
        &self.address
    }

    #[must_use]
    /// Gets the port of the server.
    pub const fn port(&self) -> u16 {
        self.port
    }

    #[must_use]
    /// Gets the password of the server.
    pub const fn password(&self) -> &Password {
        &self.password
    }

    #[must_use]
    /// Gets the pack used by the server.
    pub fn pack(&self) -> &str {
        &self.pack
    }
}

const fn default_port() -> u16 {
    2302
}

const fn default_battleye() -> bool {
    true
}
