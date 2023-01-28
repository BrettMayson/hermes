//! Password struct to prevent passwords from being printed in logs

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
/// A password that cannot be printed in logs.
pub struct Password(String);

impl Password {
    #[must_use]
    /// Creates a new password.
    pub const fn new(password: String) -> Self {
        Self(password)
    }

    #[must_use]
    /// Reveals the password
    pub fn reveal(&self) -> &str {
        &self.0
    }
}

impl Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Password]")
    }
}

impl Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Password]")
    }
}

impl From<String> for Password {
    fn from(password: String) -> Self {
        Self::new(password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password() {
        let password = Password::new("password".to_string());
        assert_eq!(password.reveal(), "password");
    }

    #[test]
    fn test_password_display() {
        let password = Password::new("password".to_string());
        assert_eq!(format!("{password}"), "[Password]");
    }

    #[test]
    fn test_password_debug() {
        let password = Password::new("password".to_string());
        assert_eq!(format!("{password:?}"), "[Password]");
    }

    #[test]
    fn test_password_from() {
        let password = Password::from("password".to_string());
        assert_eq!(password.reveal(), "password");
    }
}
