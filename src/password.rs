//! A module with a safe password wrapper.

use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::Hidden;

/// A hidden string that implements [`Zeroize`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct SafePassword {
    password: Hidden<Box<[u8]>>,
}

impl<S: Into<String>> From<S> for SafePassword {
    fn from(s: S) -> Self {
        Self {
            password: Hidden::from(s.into().into_bytes().into_boxed_slice()),
        }
    }
}

impl Drop for SafePassword {
    fn drop(&mut self) {
        self.password.reveal_mut().zeroize();
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
        Ok(Self::from(s.to_owned()))
    }
}

impl SafePassword {
    /// Gets a reference to bytes of a passphrase.
    pub fn reveal(&self) -> &[u8] {
        self.password.reveal()
    }
}

#[cfg(test)]
mod tests {
    use crate::SafePassword;

    #[test]
    fn test_password() {
        assert_eq!(
            SafePassword::from("secret_must_match".to_string()),
            SafePassword::from("secret_must_match")
        );
    }
}
