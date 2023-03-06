// Copyright 2022 The Tari Project
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

//! A module for smart bytes serialization.
//!
//! It stores bytes as hex for human readable formats and
//! uses bytes for binary formats.

use core::{fmt, marker::PhantomData};

use serde::{
    de::{Error, Visitor},
    Deserializer,
    Serializer,
};

use crate::{
    byte_array::ByteArray,
    hex::{from_hex, Hex},
};

/// Serializes a [`ByteArray`] to a hex string or a binary array.
pub fn serialize<S, T>(data: &T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ByteArray,
{
    if ser.is_human_readable() {
        ser.serialize_str(&data.to_hex())
    } else {
        ser.serialize_bytes(data.as_bytes())
    }
}

/// Serializes a [`ByteArray`] from a hex string or a binary array.
pub fn deserialize<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: ByteArray,
{
    let visitor = HexVisitor::default();
    if de.is_human_readable() {
        de.deserialize_string(visitor)
    } else {
        de.deserialize_bytes(visitor)
    }
}

struct HexVisitor<T> {
    _target: PhantomData<T>,
}

impl<T> Default for HexVisitor<T> {
    fn default() -> Self {
        Self { _target: PhantomData }
    }
}

impl<'de, T> Visitor<'de> for HexVisitor<T>
where T: ByteArray
{
    type Value = T;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("Expecting a binary array or hex string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where E: Error {
        let bytes = from_hex(v).map_err(|e| E::custom(e.to_string()))?;
        self.visit_bytes(&bytes)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where E: Error {
        self.visit_str(&v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where E: Error {
        T::from_bytes(v).map_err(|e| E::custom(e.to_string()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where E: Error {
        self.visit_bytes(v)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct HexOrBytes(#[serde(with = "super")] [u8; 4]);

    #[test]
    fn check_serde_hex_human_readable() {
        let hex_or_bytes = HexOrBytes([1, 2, 3, 255]);
        let expected = "\"010203ff\"";
        assert_eq!(serde_json::to_string(&hex_or_bytes).unwrap(), expected);
        let restored: HexOrBytes = serde_json::from_str(expected).unwrap();
        assert_eq!(hex_or_bytes, restored);
    }

    #[test]
    fn check_serde_hex_binary() {
        let hex_or_bytes = HexOrBytes([1, 2, 3, 255]);
        let mut expected: Vec<u8> = Vec::new();
        expected.write_all(&hex_or_bytes.0.len().to_le_bytes()).unwrap();
        expected.write_all(&[1, 2, 3, 255]).unwrap();
        assert_eq!(bincode::serialize(&hex_or_bytes).unwrap(), expected);
        let restored: HexOrBytes = bincode::deserialize(&expected).unwrap();
        assert_eq!(hex_or_bytes, restored);
    }
}
