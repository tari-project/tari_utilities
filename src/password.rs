use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::Hidden;

/// A hidden string that implements [`Zeroize`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct SafePassword {
    password: Hidden<String>,
}

impl From<String> for SafePassword {
    fn from(s: String) -> Self {
        Self {
            password: Hidden::from(s),
        }
    }
}

impl Drop for SafePassword {
    fn drop(&mut self) {
        self.password.zeroize();
    }
}

/// An error for parsing a password from string.
#[derive(Debug)]
pub struct PasswordError;

impl Error for PasswordError {}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PasswordError")
    }
}

impl FromStr for SafePassword {
    type Err = PasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            password: Hidden::from(s.to_owned()),
        })
    }
}

impl SafePassword {
    /// Gets a reference to bytes of a passphrase.
    pub fn reveal(&self) -> &[u8] {
        self.password.as_bytes()
    }
}
