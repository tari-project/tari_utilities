//! A type for handling a passphrase safely.

use std::{error::Error, fmt::Display, str::FromStr};

use crate::{
    hidden::{Hidden, HiddenLabel},
    hidden_label,
};

hidden_label!(SafePasswordLabel);

/// A representation of a passphrase that zeroizes on drop, prevents display and debug output, and limits access to
/// references
///
/// The passphrase can be instantiated from a string or any type that can become a string.
/// It is converted to a byte array, which can be accessed as a mutable or immutable reference.
///
/// ```edition2018
/// # use tari_utilities::SafePassword
///
/// // Create a safe passphrase
/// let passphrase = SafePassword::from("my secret passphrase");
///
/// // We can also use a string directly
/// assert_eq!(
///     passphrase.reveal(),
///     SafePassword::from("my secret passphrase".to_string())
/// );
/// ```
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
        Ok(Self::hide(<&str as Into<String>>::into(s).into_bytes()))
    }
}

impl<S: Into<String>> From<S> for SafePassword {
    fn from(s: S) -> Self {
        Self::hide(s.into().into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::SafePassword;

    #[test]
    fn from_strings() {
        let password = "password";

        let from_str = SafePassword::from_str(password).unwrap();
        let from_string = SafePassword::from(password.to_string());
        let from_string_ref = SafePassword::from(password);

        assert_eq!(from_str, from_string);
        assert_eq!(from_string, from_string_ref);
    }
}
