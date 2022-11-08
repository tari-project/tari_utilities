//! A module with a safe password wrapper.

use std::{error::Error, fmt::Display, str::FromStr};

use crate::{
    hidden::{Hidden, HiddenLabel},
    hidden_label,
};

hidden_label!(SafePasswordLabel);

/// A hidden password type that zeroizes when it goes away
pub type SafePassword = Hidden<Vec<u8>, SafePasswordLabel>;

/// An error for parsing a password from a string
#[derive(Debug)]
pub struct PasswordError;

impl Error for PasswordError {}

impl Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PasswordError")
    }
}

impl FromStr for SafePassword {
    type Err = PasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::hide(s.as_bytes().to_vec()))
    }
}

impl<S: Into<String>> From<S> for SafePassword {
    fn from(s: S) -> Self {
        Self::hide(s.into().as_bytes().to_vec())
    }
}
