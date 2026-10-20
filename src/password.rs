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
#[cfg(feature = "serde")]
use core::fmt;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
    de::{SeqAccess, Visitor},
    ser::Error,
};
#[cfg(feature = "serde")]
use zeroize::Zeroizing;

use crate::hidden::Hidden;

/// A representation of a passphrase that zeroizes on drop, prevents display and debug output, and limits access to
/// references
///
/// The passphrase can be instantiated from a string or any type that can become a string.
/// It is converted to a byte array, which can be accessed as a mutable or immutable reference.
///
/// It deserializes from a sequence of bytes. It can't be serialized: serializing always returns an error, so the
/// passphrase is never written out.
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
pub struct SafePassword {
    passphrase: Hidden<Vec<u8>>,
}

impl SafePassword {
    /// Get an immutable reference to the passphrase
    pub fn reveal(&self) -> &Vec<u8> {
        self.passphrase.reveal()
    }

    /// Get a mutable reference to the passphrase bytes. The length can't be changed, so the passphrase is never
    /// reallocated, which would leave an unzeroized copy behind.
    pub fn reveal_mut(&mut self) -> &mut [u8] {
        self.passphrase.reveal_mut().as_mut_slice()
    }
}

impl FromStr for SafePassword {
    type Err = String;

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

/// The upper bound on the up-front allocation taken from an untrusted length hint when deserializing.
#[cfg(feature = "serde")]
const MAX_PREALLOCATION: usize = 4096;

#[cfg(feature = "serde")]
impl Serialize for SafePassword {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        Err(S::Error::custom("SafePassword can't be serialized"))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for SafePassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(PasswordVisitor)
    }
}

#[cfg(feature = "serde")]
struct PasswordVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for PasswordVisitor {
    type Value = SafePassword;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence of bytes")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de> {
        let capacity = seq.size_hint().unwrap_or(0).min(MAX_PREALLOCATION);
        let mut buf = Zeroizing::new(Vec::with_capacity(capacity));
        while let Some(byte) = seq.next_element::<u8>()? {
            if buf.len() == buf.capacity() {
                // Grow by hand so the old buffer is zeroized when dropped, instead of being freed by `Vec`'s realloc
                let mut bigger = Zeroizing::new(Vec::with_capacity((buf.capacity() * 2).max(32)));
                bigger.extend_from_slice(&buf);
                buf = bigger;
            }
            buf.push(byte);
        }
        Ok(SafePassword {
            passphrase: Hidden::hide(core::mem::take(&mut *buf)),
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec::Vec};
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
    fn serialization_fails_closed() {
        let safe_password = SafePassword::from("password");
        let err = serde_json::to_string(&safe_password).unwrap_err();
        assert!(err.to_string().contains("SafePassword can't be serialized"));
        assert!(bincode::serialize(&safe_password).is_err());

        #[derive(serde::Serialize)]
        struct Config {
            password: Option<SafePassword>,
        }
        let config = Config { password: None };
        assert_eq!(serde_json::to_string(&config).unwrap(), "{\"password\":null}");
        let config = Config {
            password: Some(safe_password),
        };
        assert!(serde_json::to_string(&config).is_err());
    }

    #[test]
    fn deserialize_byte_sequence() {
        // The format written by tari_utilities <= 0.10
        let deser: SafePassword = serde_json::from_str("[112,97,115,115,119,111,114,100]").unwrap();
        assert_eq!(deser.reveal(), b"password");

        let bin = bincode::serialize(&b"password".to_vec()).unwrap();
        let deser: SafePassword = bincode::deserialize(&bin).unwrap();
        assert_eq!(deser.reveal(), b"password");

        // Longer than the initial allocation, to exercise growth
        let long: Vec<u8> = (0..200u8).collect();
        let deser: SafePassword = serde_json::from_str(&serde_json::to_string(&long).unwrap()).unwrap();
        assert_eq!(deser.reveal(), &long);

        let deser: SafePassword = serde_json::from_str("[]").unwrap();
        assert!(deser.reveal().is_empty());
        assert!(serde_json::from_str::<SafePassword>("[256]").is_err());
    }

    #[test]
    fn reveal_mut_is_a_slice() {
        let mut password = SafePassword::from("abc");
        password.reveal_mut().copy_from_slice(b"xyz");
        assert_eq!(password.reveal(), b"xyz");
    }

    #[test]
    // If the error type in `impl FromStr for SafePassword` is wrong, this test will fail to compile
    fn use_with_clap_and_serde() {
        use alloc::string::String;

        use clap::Parser;
        use serde::{Deserialize, Serialize};

        fn deserialize_safe_password_option<'de, D>(deserializer: D) -> Result<Option<SafePassword>, D::Error>
        where D: serde::Deserializer<'de> {
            let password: Option<String> = Deserialize::deserialize(deserializer)?;
            println!("Password: {:?}", password);
            Ok(password.map(SafePassword::from))
        }

        #[derive(Parser, Serialize, Deserialize, Debug)]
        #[clap(author, version, about, long_about = None)]
        #[clap(propagate_version = true)]
        struct Cli {
            #[serde(deserialize_with = "deserialize_safe_password_option")]
            #[clap(long, env = "MY_PASSWORD", hide_env_values = true)]
            pub password: Option<SafePassword>,
        }

        let hex_password = "48656c6c6f20576f726c64";
        let input = serde_json::json!({
            "password": hex_password,
        });

        // Deserialize the JSON input into the Cli struct
        let cli: Cli = serde_json::from_value(input).unwrap();

        // Access the SafePassword field
        if let Some(password) = cli.password {
            let password_reveal = String::from_utf8(password.reveal().clone()).unwrap();
            assert_eq!(hex_password.to_string(), password_reveal);
        }
    }
}
