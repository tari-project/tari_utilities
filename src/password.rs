// Copyright 2022. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! A type for handling a passphrase safely.
use alloc::{string::String, vec::Vec};
use core::{fmt::Display, str::FromStr};

#[cfg(feature = "serde")]
use serde::{ser::SerializeSeq, Serialize, Serializer};

use crate::hidden::Hidden;

/// A representation of a passphrase that zeroizes on drop, prevents display and debug output, and limits access to
/// references
///
/// The passphrase can be instantiated from a string or any type that can become a string.
/// It is converted to a byte array, which can be accessed as a mutable or immutable reference.
/// You can serialize and deserialize it transparently.
///
/// ```edition2018
/// # use tari_utilities::SafePassword;
///
/// // Create a safe passphrase
/// let passphrase = SafePassword::from("my secret passphrase");
///
/// // We can also use a string directly
/// assert_eq!(
///     passphrase.reveal(),
///     SafePassword::from("my secret passphrase".to_string()).reveal()
/// );
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct SafePassword {
    passphrase: Hidden<Vec<u8>>,
}

impl SafePassword {
    /// Get an immutable reference to the passphrase
    pub fn reveal(&self) -> &Vec<u8> {
        self.passphrase.reveal()
    }

    /// Get a mutable reference to the passphrase
    pub fn reveal_mut(&mut self) -> &mut Vec<u8> {
        self.passphrase.reveal_mut()
    }
}

/// An error for parsing a password from a string
#[derive(Debug)]
pub struct PasswordError;

impl Display for PasswordError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PasswordError")
    }
}

impl FromStr for SafePassword {
    type Err = PasswordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            passphrase: Hidden::<Vec<u8>>::hide(String::from(s).into_bytes()),
        })
    }
}

impl<S: Into<String>> From<S> for SafePassword {
    fn from(s: S) -> Self {
        Self {
            passphrase: Hidden::<Vec<u8>>::hide(s.into().into_bytes()),
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for SafePassword {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(self.passphrase.reveal().len()))?;
        for e in self.passphrase.reveal() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use std::str::FromStr;

    use super::SafePassword;

    #[test]
    fn from_strings() {
        let password = "password";

        let from_str = SafePassword::from_str(password).unwrap();
        let from_string = SafePassword::from(password.to_string());
        let from_string_ref = SafePassword::from(password);

        assert_eq!(from_str.reveal(), from_string.reveal());
        assert_eq!(from_string.reveal(), from_string_ref.reveal());
    }

    #[test]
    fn serialization() {
        let safe_password = SafePassword::from("password");
        let ser = serde_json::to_string(&safe_password).unwrap();
        let deser: SafePassword = serde_json::from_str(&ser).unwrap();

        assert_eq!(safe_password.reveal(), deser.reveal());
    }
}
