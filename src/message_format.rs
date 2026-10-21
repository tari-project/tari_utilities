// Copyright 2018 The Tari Project
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

//! A `MessageFormat` trait that handles conversion from and to binary, json, or base64.
//!
//! Binary decoding rejects input over [MAX_BINARY_SIZE] bytes and rejects trailing bytes. The binary format is the same
//! as `bincode::serialize` (bincode 1.x, fixed-int encoding).
//!
//! The limit bounds the input size, not the memory or time used to decode it. These types must not be decoded from
//! untrusted input:
//! - recursive types (e.g. a struct containing an `Option<Box<Self>>`): bincode 1.x has no depth limit, so deeply
//!   nested input can overflow the stack;
//! - collections of zero-sized elements (e.g. `Vec<()>`): a small length prefix can make decoding loop for a very long
//!   time;
//! - collections whose elements are large in memory but small when encoded (e.g. `Vec<Option<LargeStruct>>`): 4 MiB of
//!   input can allocate gigabytes.

use alloc::{string::String, vec::Vec};

use bincode::Options;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use snafu::prelude::*;
use zeroize::Zeroizing;

/// The maximum input size in bytes `from_binary` and `from_base64` accept (4 MiB). This limits input size, not memory.
pub const MAX_BINARY_SIZE: u64 = 4 * 1024 * 1024;

/// Errors for [MessageFormat] trait.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum MessageFormatError {
    #[snafu(display("An error occurred serialising an object into binary"))]
    BinarySerializeError {},
    #[snafu(display("An error occurred deserialising binary data into an object"))]
    BinaryDeserializeError {},
    #[snafu(display("An error occurred de-/serialising an object from/into JSON"))]
    JSONError {},
    #[snafu(display("An error occurred deserialising an object from Base64"))]
    Base64DeserializeError {},
}

/// Trait for converting to/from binary/json/base64.
pub trait MessageFormat: Sized {
    /// Convert to binary.
    fn to_binary(&self) -> Result<Vec<u8>, MessageFormatError>;
    /// Convert to json.
    fn to_json(&self) -> Result<String, MessageFormatError>;
    /// Convert to base64.
    fn to_base64(&self) -> Result<String, MessageFormatError>;

    /// Convert from binary. Input over [MAX_BINARY_SIZE] bytes and trailing bytes are rejected. The limit does not
    /// bound decoded memory; see the module docs for types that must not be decoded from untrusted input.
    fn from_binary(msg: &[u8]) -> Result<Self, MessageFormatError> {
        Self::from_binary_with_limit(msg, MAX_BINARY_SIZE)
    }
    /// Convert from binary, rejecting input over `limit` bytes and trailing bytes. The limit does not bound decoded
    /// memory.
    fn from_binary_with_limit(msg: &[u8], limit: u64) -> Result<Self, MessageFormatError>;
    /// Convert from json. Trailing whitespace is allowed, any other trailing data is rejected.
    fn from_json(msg: &str) -> Result<Self, MessageFormatError>;
    /// Convert from base64. The decoded data is limited to [MAX_BINARY_SIZE] bytes.
    fn from_base64(msg: &str) -> Result<Self, MessageFormatError>;
}

impl<T> MessageFormat for T
where T: DeserializeOwned + Serialize
{
    fn to_binary(&self) -> Result<Vec<u8>, MessageFormatError> {
        bincode::serialize(self).map_err(|_| MessageFormatError::BinarySerializeError {})
    }

    fn to_json(&self) -> Result<String, MessageFormatError> {
        serde_json::to_string(self).map_err(|_| MessageFormatError::JSONError {})
    }

    fn to_base64(&self) -> Result<String, MessageFormatError> {
        let val = self.to_binary()?;
        Ok(base64::encode(val))
    }

    fn from_binary_with_limit(msg: &[u8], limit: u64) -> Result<Self, MessageFormatError> {
        // Checked up front because bincode 1.3's `Options::deserialize` replaces any `with_limit` with no limit for
        // slice input; a slice decoder can't read past the slice, so this gives the same bound on bytes read
        if msg.len() as u64 > limit {
            return Err(MessageFormatError::BinaryDeserializeError {});
        }
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .reject_trailing_bytes()
            .deserialize(msg)
            .map_err(|_| MessageFormatError::BinaryDeserializeError {})
    }

    fn from_json(msg: &str) -> Result<Self, MessageFormatError> {
        let mut de = serde_json::Deserializer::from_reader(msg.as_bytes());
        let value = Deserialize::deserialize(&mut de).map_err(|_| MessageFormatError::JSONError {})?;
        de.end().map_err(|_| MessageFormatError::JSONError {})?;
        Ok(value)
    }

    fn from_base64(msg: &str) -> Result<Self, MessageFormatError> {
        // Every 4 base64 characters decode to at most 3 bytes
        if msg.len() as u64 > MAX_BINARY_SIZE.div_ceil(3) * 4 {
            return Err(MessageFormatError::Base64DeserializeError {});
        }
        let buf = Zeroizing::new(base64::decode(msg).map_err(|_| MessageFormatError::Base64DeserializeError {})?);
        Self::from_binary(&buf)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, string::ToString};

    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct TestMessage {
        key: String,
        value: u64,
        sub_message: Option<Box<TestMessage>>,
    }

    impl TestMessage {
        pub fn new(key: &str, value: u64) -> TestMessage {
            TestMessage {
                key: key.to_string(),
                value,
                sub_message: None,
            }
        }

        pub fn set_sub_message(&mut self, msg: TestMessage) {
            self.sub_message = Some(Box::new(msg));
        }
    }

    #[test]
    fn binary_simple() {
        let val = TestMessage::new("twenty", 20);
        let msg = val.to_binary().unwrap();
        assert_eq!(
            msg,
            b"\x06\x00\x00\x00\x00\x00\x00\x00\x74\x77\x65\x6e\x74\x79\x14\x00\x00\x00\x00\x00\x00\x00\x00"
        );
        let val2 = TestMessage::from_binary(&msg).unwrap();
        assert_eq!(val, val2);
    }

    #[test]
    fn base64_simple() {
        let val = TestMessage::new("twenty", 20);
        let msg = val.to_base64().unwrap();
        assert_eq!(msg, "BgAAAAAAAAB0d2VudHkUAAAAAAAAAAA=");
        let val2 = TestMessage::from_base64(&msg).unwrap();
        assert_eq!(val, val2);
    }

    #[test]
    fn json_simple() {
        let val = TestMessage::new("twenty", 20);
        let msg = val.to_json().unwrap();
        assert_eq!(msg, "{\"key\":\"twenty\",\"value\":20,\"sub_message\":null}");
        let val2 = TestMessage::from_json(&msg).unwrap();
        assert_eq!(val, val2);
    }

    #[test]
    fn nested_message() {
        let inner = TestMessage::new("today", 100);
        let mut val = TestMessage::new("tomorrow", 50);
        val.set_sub_message(inner);

        let msg_json = val.to_json().unwrap();
        assert_eq!(
            msg_json,
            "{\"key\":\"tomorrow\",\"value\":50,\"sub_message\":{\"key\":\"today\",\"value\":100,\"sub_message\":\
             null}}"
        );

        let msg_base64 = val.to_base64().unwrap();
        assert_eq!(
            msg_base64,
            "CAAAAAAAAAB0b21vcnJvdzIAAAAAAAAAAQUAAAAAAAAAdG9kYXlkAAAAAAAAAAA="
        );

        let msg_bin = val.to_binary().unwrap();
        assert_eq!(
            msg_bin,
            b"\x08\x00\x00\x00\x00\x00\x00\x00\x74\x6f\x6d\x6f\x72\x72\x6f\x77\x32\x00\x00\x00\x00\x00\x00\x00\x01\x05\x00\x00\x00\x00\x00\x00\x00\x74\x6f\x64\x61\x79\x64\x00\x00\x00\x00\x00\x00\x00\x00".to_vec()
        );

        let val2 = TestMessage::from_json(&msg_json).unwrap();
        assert_eq!(val, val2);

        let val2 = TestMessage::from_base64(&msg_base64).unwrap();
        assert_eq!(val, val2);

        let val2 = TestMessage::from_binary(&msg_bin).unwrap();
        assert_eq!(val, val2);
    }

    #[test]
    fn fail_json() {
        let err = TestMessage::from_json("{\"key\":5}").unwrap_err();
        assert!(matches!(err, MessageFormatError::JSONError {}));
    }

    #[test]
    fn fail_base64() {
        let err = TestMessage::from_base64("aaaaa$aaaaa").unwrap_err();
        assert!(matches!(err, MessageFormatError::Base64DeserializeError {}));

        let err = TestMessage::from_base64("j6h0b21vcnJvdzKTpXRvZGF5ZMA=").unwrap_err();
        assert!(matches!(err, MessageFormatError::BinaryDeserializeError {}));
    }

    #[test]
    fn fail_binary() {
        let err = TestMessage::from_binary(b"").unwrap_err();
        assert!(matches!(err, MessageFormatError::BinaryDeserializeError {}));
    }

    #[test]
    fn binary_matches_bincode_serialize() {
        let mut val = TestMessage::new("tomorrow", 50);
        val.set_sub_message(TestMessage::new("today", 100));
        let msg = bincode::serialize(&val).unwrap();
        assert_eq!(val.to_binary().unwrap(), msg);
        assert_eq!(TestMessage::from_binary(&msg).unwrap(), val);
    }

    #[test]
    fn binary_rejects_trailing_bytes() {
        let mut msg = TestMessage::new("twenty", 20).to_binary().unwrap();
        msg.push(0);
        let err = TestMessage::from_binary(&msg).unwrap_err();
        assert!(matches!(err, MessageFormatError::BinaryDeserializeError {}));
    }

    #[test]
    fn binary_limit() {
        let val = vec![7u8; 100];
        let msg = val.to_binary().unwrap();
        assert_eq!(Vec::<u8>::from_binary_with_limit(&msg, 108).unwrap(), val);
        assert!(Vec::<u8>::from_binary_with_limit(&msg, 107).is_err());

        let too_big = vec![0u8; usize::try_from(MAX_BINARY_SIZE).unwrap()];
        let msg = too_big.to_binary().unwrap();
        assert!(Vec::<u8>::from_binary(&msg).is_err());
        assert_eq!(
            Vec::<u8>::from_binary_with_limit(&msg, MAX_BINARY_SIZE + 8).unwrap(),
            too_big
        );
    }

    #[test]
    fn base64_limit() {
        let max_len = usize::try_from(MAX_BINARY_SIZE.div_ceil(3) * 4).unwrap();
        let msg = "A".repeat(max_len + 4);
        let err = Vec::<u8>::from_base64(&msg).unwrap_err();
        assert!(matches!(err, MessageFormatError::Base64DeserializeError {}));
    }

    #[test]
    fn json_trailing_data() {
        let msg = TestMessage::new("twenty", 20).to_json().unwrap();
        assert!(TestMessage::from_json(&format!("{msg} \n\t")).is_ok());
        assert!(TestMessage::from_json(&format!("{msg}{msg}")).is_err());
        assert!(TestMessage::from_json(&format!("{msg} garbage")).is_err());
    }

    #[test]
    fn chrono_datetime_wire_compat() {
        use chrono::{DateTime, TimeZone, Utc};

        // Bytes produced by `bincode::serialize` (tari_utilities <= 0.10) for a stored DHT `OfflineTimestamp`
        let stored = b"\x14\x00\x00\x00\x00\x00\x00\x002024-01-02T03:04:05Z";
        let expected = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
        assert_eq!(bincode::serialize(&expected).unwrap(), stored.to_vec());
        assert_eq!(expected.to_binary().unwrap(), stored.to_vec());
        assert_eq!(DateTime::<Utc>::from_binary(stored).unwrap(), expected);
    }
}
