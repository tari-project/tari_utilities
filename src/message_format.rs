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

//! A `MessageFormat` trait that handles conversion from and to json or base64.
//!
//! Base64 decoding rejects decoded input over [MAX_BINARY_SIZE] bytes, and JSON decoding rejects trailing
//! non-whitespace bytes.

use alloc::string::String;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use snafu::prelude::*;
use zeroize::Zeroizing;

/// The maximum decoded input size in bytes `from_base64` accepts (4 MiB). This limits input size, not memory.
pub const MAX_BINARY_SIZE: u64 = 4 * 1024 * 1024;

/// Errors for [MessageFormat] trait.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum MessageFormatError {
    #[snafu(display("An error occurred de-/serialising an object from/into JSON"))]
    JSONError {},
    #[snafu(display("An error occurred deserialising an object from Base64"))]
    Base64DeserializeError {},
}

/// Trait for converting to/from json/base64.
pub trait MessageFormat: Sized {
    /// Convert to json.
    fn to_json(&self) -> Result<String, MessageFormatError>;
    /// Convert to base64.
    fn to_base64(&self) -> Result<String, MessageFormatError>;

    /// Convert from json. Trailing whitespace is allowed, any other trailing data is rejected.
    fn from_json(msg: &str) -> Result<Self, MessageFormatError>;
    /// Convert from base64. The decoded data is limited to [MAX_BINARY_SIZE] bytes.
    fn from_base64(msg: &str) -> Result<Self, MessageFormatError>;
}

impl<T> MessageFormat for T
where T: DeserializeOwned + Serialize
{
    fn to_json(&self) -> Result<String, MessageFormatError> {
        serde_json::to_string(self).map_err(|_| MessageFormatError::JSONError {})
    }

    fn to_base64(&self) -> Result<String, MessageFormatError> {
        let val = Zeroizing::new(self.to_json()?);
        Ok(base64::encode(val.as_bytes()))
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
        if buf.len() as u64 > MAX_BINARY_SIZE {
            return Err(MessageFormatError::Base64DeserializeError {});
        }
        let mut de = serde_json::Deserializer::from_reader(buf.as_slice());
        let value = Deserialize::deserialize(&mut de).map_err(|_| MessageFormatError::JSONError {})?;
        de.end().map_err(|_| MessageFormatError::JSONError {})?;
        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, string::ToString, vec::Vec};

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
    fn base64_simple() {
        let val = TestMessage::new("twenty", 20);
        let msg = val.to_base64().unwrap();
        assert_eq!(msg, "eyJrZXkiOiJ0d2VudHkiLCJ2YWx1ZSI6MjAsInN1Yl9tZXNzYWdlIjpudWxsfQ==");
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
            "eyJrZXkiOiJ0b21vcnJvdyIsInZhbHVlIjo1MCwic3ViX21lc3NhZ2UiOnsia2V5IjoidG9kYXkiLCJ2YWx1ZSI6MTAwLCJzdWJfbWVzc2FnZSI6bnVsbH19"
        );

        let val2 = TestMessage::from_json(&msg_json).unwrap();
        assert_eq!(val, val2);

        let val2 = TestMessage::from_base64(&msg_base64).unwrap();
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
        assert!(matches!(err, MessageFormatError::JSONError {}));
    }

    #[test]
    fn base64_limit() {
        let max_len = usize::try_from(MAX_BINARY_SIZE.div_ceil(3) * 4).unwrap();
        let msg = "A".repeat(max_len + 4);
        let err = Vec::<u8>::from_base64(&msg).unwrap_err();
        assert!(matches!(err, MessageFormatError::Base64DeserializeError {}));

        // 4_194_305 decoded bytes still fits within max_len base64 chars (5_592_408) because of 3-byte rounding
        let raw = vec![b' '; usize::try_from(MAX_BINARY_SIZE).unwrap() + 1];
        let msg = base64::encode(&raw);
        assert_eq!(msg.len(), max_len);
        let err = Vec::<u8>::from_base64(&msg).unwrap_err();
        assert!(matches!(err, MessageFormatError::Base64DeserializeError {}));
    }

    #[test]
    fn json_trailing_data() {
        let msg = TestMessage::new("twenty", 20).to_json().unwrap();
        assert!(TestMessage::from_json(&format!("{msg} \n\t")).is_ok());
        assert!(TestMessage::from_json(&format!("{msg}{msg}")).is_err());
        assert!(TestMessage::from_json(&format!("{msg} garbage")).is_err());

        let msg_base64 = base64::encode(format!("{msg} garbage").as_bytes());
        assert!(matches!(
            TestMessage::from_base64(&msg_base64),
            Err(MessageFormatError::JSONError {})
        ));
    }
}
