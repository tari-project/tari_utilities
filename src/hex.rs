// Copyright 2020. The Tari Project
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

//! Functions for conversion between binary and hex string.
//!
//! Encoding and decoding avoid data-dependent branches and lookups, so the codec is suitable for secret material.

use alloc::{string::String, vec::Vec};

#[cfg(feature = "serde")]
use serde::{Serializer, ser::Error};
use snafu::prelude::*;
use zeroize::Zeroize;

use crate::alloc::string::ToString;

/// Maximum number of bytes that can be encoded to hex.
const MAX_BYTES_SIZE: usize = 262_144; // 256kb

/// Any object implementing this trait has the ability to represent itself as a hexadecimal string and convert from it.
pub trait Hex {
    /// Try to convert the given hexadecimal string to the type.
    ///
    /// # Errors
    /// Any failures (incorrect  string length, non hex characters, etc.) return a [HexError](enum.HexError.html) with
    /// an explanatory note.
    fn from_hex(hex: &str) -> Result<Self, HexError>
    where Self: Sized;

    /// Return the lowercase hexadecimal string representation of the type.
    ///
    /// # Errors
    /// Returns [HexError::TooLarge] if the value is larger than 256 KiB.
    fn to_hex(&self) -> Result<String, HexError>;

    /// Return a hexadecimal representation for display or logging purposes. This never fails: values that are too large
    /// to encode are truncated and end with a marker that is not valid hex.
    fn to_hex_trimmed(&self) -> String {
        self.to_hex().unwrap_or_else(|_| "…(too large)".to_string())
    }
}

/// Errors for [Hex] trait.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum HexError {
    #[snafu(display(
        "Only hexadecimal characters (0-9, a-f, A-F) are permitted, with an optional `0x` prefix and surrounding \
         whitespace; canonical decoding permits only 0-9 and a-f"
    ))]
    InvalidCharacter {},
    #[snafu(display(
        "Hex string lengths (excluding any `0x` prefix and surrounding whitespace) must be a multiple of 2"
    ))]
    LengthError {},
    #[snafu(display("Invalid hex representation for the target type"))]
    HexConversionError {},
    #[snafu(display("Input is larger than the maximum of 262144 bytes that can be hex encoded"))]
    TooLarge {},
}

/// Encode the provided bytes into a lowercase hex string.
///
/// # Errors
/// Returns [HexError::TooLarge] if there are more than 256 KiB of bytes.
pub fn to_hex(bytes: &[u8]) -> Result<String, HexError> {
    if bytes.len() > MAX_BYTES_SIZE {
        return Err(HexError::TooLarge {});
    }
    Ok(encode(bytes))
}

/// Encode the provided bytes into a lowercase hex string for display or logging purposes. Input larger than 256 KiB
/// is truncated, and the output ends with a marker like `…(+123 bytes)` that is not valid hex.
pub fn to_hex_trimmed(bytes: &[u8]) -> String {
    if bytes.len() <= MAX_BYTES_SIZE {
        return encode(bytes);
    }
    let (head, _) = bytes.split_at(MAX_BYTES_SIZE);
    let mut s = encode(head);
    s.push_str(&format!("…(+{} bytes)", bytes.len() - MAX_BYTES_SIZE));
    s
}

/// Encode the provided vector of bytes into a hex string.
///
/// # Errors
/// Returns [HexError::TooLarge] if any entry has more than 256 KiB of bytes.
pub fn to_hex_multiple(bytearray: &[Vec<u8>]) -> Result<Vec<String>, HexError> {
    let mut result = Vec::with_capacity(bytearray.len());
    for bytes in bytearray {
        result.push(to_hex(bytes)?);
    }
    Ok(result)
}

/// Decode a hex string into bytes. Upper and lowercase digits are accepted, surrounding whitespace is trimmed and a
/// lowercase `0x` prefix is removed.
pub fn from_hex(hex_str: &str) -> Result<Vec<u8>, HexError> {
    let hex_trim = hex_str.trim();
    let hex_trim = hex_trim.strip_prefix("0x").unwrap_or(hex_trim);
    decode(hex_trim.as_bytes(), true)
}

/// Decode a canonical hex string into bytes. Only lowercase digits are accepted, with no whitespace or `0x` prefix, so
/// every byte string has exactly one accepted encoding.
pub fn from_hex_canonical(hex_str: &str) -> Result<Vec<u8>, HexError> {
    decode(hex_str.as_bytes(), false)
}

/// Use a serde serializer to serialize the hex string of the given object.
#[cfg(feature = "serde")]
pub fn serialize_to_hex<S, T>(t: &T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Hex,
{
    ser.serialize_str(&t.to_hex().map_err(S::Error::custom)?)
}

fn encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        s.push(encode_nibble(byte >> 4));
        s.push(encode_nibble(byte & 0x0f));
    }
    s
}

// Maps 0..=15 to '0'..='9', 'a'..='f'. `(9 - n) >> 8` is all ones only when n > 9, which adds the gap to 'a'.
fn encode_nibble(n: u8) -> char {
    let n = i16::from(n);
    let c = n + 0x30 + (((9 - n) >> 8) & 0x27);
    char::from(c as u8)
}

fn decode(hex: &[u8], allow_upper: bool) -> Result<Vec<u8>, HexError> {
    if !hex.len().is_multiple_of(2) {
        return Err(HexError::LengthError {});
    }
    let (pairs, _) = hex.as_chunks::<2>();
    let mut out = Vec::with_capacity(pairs.len());
    let mut invalid = 0u16;
    for [hi, lo] in pairs {
        let hi = decode_nibble(*hi, allow_upper);
        let lo = decode_nibble(*lo, allow_upper);
        invalid |= hi | lo;
        out.push(((hi << 4) | lo) as u8);
    }
    if invalid >> 8 != 0 {
        out.zeroize();
        return Err(HexError::InvalidCharacter {});
    }
    Ok(out)
}

// Returns the nibble value, or 0xffff for an invalid character. Each range check `(lo - 1 - c) & (c - hi - 1)` is
// negative only when c is inside the range, so `>> 8` yields an all-ones mask that selects the offset.
fn decode_nibble(c: u8, allow_upper: bool) -> u16 {
    let c = i16::from(c);
    let mut ret: i16 = -1;
    ret += (((0x2f - c) & (c - 0x3a)) >> 8) & (c - 47);
    ret += (((0x60 - c) & (c - 0x67)) >> 8) & (c - 86);
    if allow_upper {
        ret += (((0x40 - c) & (c - 0x47)) >> 8) & (c - 54);
    }
    ret as u16
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use super::*;
    #[test]
    fn test_to_hex() {
        assert_eq!(to_hex(&[0, 0, 0, 0]).unwrap(), "00000000");
        assert_eq!(to_hex(&[10, 11, 12, 13]).unwrap(), "0a0b0c0d");
        assert_eq!(to_hex(&[0, 0, 0, 255]).unwrap(), "000000ff");
    }

    #[test]
    fn test_from_hex() {
        assert_eq!(from_hex("00000000").unwrap(), vec![0, 0, 0, 0]);
        assert_eq!(from_hex("0a0b0c0d").unwrap(), vec![10, 11, 12, 13]);
        assert_eq!(from_hex(" 0a0b0c0d  ").unwrap(), vec![10, 11, 12, 13]);
        assert_eq!(from_hex("000000ff").unwrap(), vec![0, 0, 0, 255]);
        assert_eq!(from_hex("0x800000ff").unwrap(), vec![128, 0, 0, 255]);
        assert_eq!(from_hex("0A0b0C0D").unwrap(), vec![10, 11, 12, 13]);
        assert_eq!(from_hex("").unwrap(), Vec::<u8>::new());
        assert!(from_hex("800").is_err()); // Odd number of bytes
        assert!(from_hex("8080gf").is_err()); // Invalid hex character g
        // unicode strings have odd lengths and can cause panics
        assert!(from_hex("🖖🥴").is_err());
    }

    #[test]
    fn rejects_sign_characters() {
        // `u8::from_str_radix` accepts a leading '+', which made "+f" decode to 0x0f
        assert!(matches!(from_hex("+f"), Err(HexError::InvalidCharacter {})));
        assert!(matches!(from_hex("-f"), Err(HexError::InvalidCharacter {})));
        assert!(matches!(from_hex("00+f"), Err(HexError::InvalidCharacter {})));
    }

    #[test]
    fn rejects_every_non_hex_byte() {
        for c in 0u8..=255 {
            let s = [b'0', c];
            let result = decode(&s, true);
            if c.is_ascii_hexdigit() {
                assert_eq!(result.unwrap(), vec![
                    u8::from_str_radix(&(c as char).to_string(), 16).unwrap()
                ]);
            } else {
                assert!(
                    matches!(result, Err(HexError::InvalidCharacter {})),
                    "byte {c} accepted"
                );
            }
            let result = decode(&s, false);
            if c.is_ascii_digit() || (b'a'..=b'f').contains(&c) {
                assert!(result.is_ok());
            } else {
                assert!(
                    matches!(result, Err(HexError::InvalidCharacter {})),
                    "byte {c} accepted"
                );
            }
        }
    }

    #[test]
    fn round_trip_all_bytes() {
        let bytes: Vec<u8> = (0u8..=255).collect();
        let hex = to_hex(&bytes).unwrap();
        let expected: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        assert_eq!(hex, expected);
        assert_eq!(from_hex(&hex).unwrap(), bytes);
        assert_eq!(from_hex_canonical(&hex).unwrap(), bytes);
        assert_eq!(from_hex(&hex.to_uppercase()).unwrap(), bytes);
    }

    #[test]
    fn canonical_decoding() {
        assert_eq!(from_hex_canonical("0a0bff").unwrap(), vec![10, 11, 255]);
        assert!(matches!(from_hex_canonical("0A"), Err(HexError::InvalidCharacter {})));
        assert!(matches!(from_hex_canonical("0x0a"), Err(HexError::InvalidCharacter {})));
        assert!(matches!(from_hex_canonical(" 0a"), Err(HexError::LengthError {})));
        assert!(matches!(from_hex_canonical(" 0a "), Err(HexError::InvalidCharacter {})));
    }

    #[test]
    fn test_to_hex_multiple() {
        let ba = [vec![16u8, 32], vec![48, 64]];
        let hexed = to_hex_multiple(&ba).unwrap();
        assert_eq!(hexed, ["1020", "3040"]);

        let ba = [vec![1u8], vec![0u8; MAX_BYTES_SIZE + 1]];
        assert!(matches!(to_hex_multiple(&ba), Err(HexError::TooLarge {})));
    }

    #[test]
    fn length_error() {
        let result = from_hex("800");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HexError::LengthError {}));
        // Check that message is the doc message above
        assert_eq!(
            err.to_string(),
            "Hex string lengths (excluding any `0x` prefix and surrounding whitespace) must be a multiple of 2"
        );
    }

    #[test]
    fn max_length_error() {
        let bytes = [0; 262_144];
        let mut hex = "".to_string();
        for _ in 0..262_144 {
            hex.push_str("00");
        }
        assert_eq!(hex, bytes.to_hex().unwrap());
        assert_eq!(hex, bytes.to_hex_trimmed());

        let bytes = [0; 262_145];
        let err = bytes.to_hex().unwrap_err();
        assert!(matches!(err, HexError::TooLarge {}));
        assert_eq!(
            err.to_string(),
            "Input is larger than the maximum of 262144 bytes that can be hex encoded"
        );
    }

    #[test]
    fn trimmed_output_is_not_valid_hex() {
        let bytes = vec![0xabu8; MAX_BYTES_SIZE + 5];
        let trimmed = to_hex_trimmed(&bytes);
        assert!(trimmed.ends_with("…(+5 bytes)"));
        assert!(trimmed.starts_with("abab"));
        assert_eq!(trimmed.len(), MAX_BYTES_SIZE * 2 + "…(+5 bytes)".len());
        assert!(from_hex(&trimmed).is_err());
        assert_eq!(bytes.to_hex_trimmed(), trimmed);
        assert_eq!(to_hex_trimmed(&[1, 2]), "0102");
    }

    #[test]
    fn default_trimmed_marker() {
        struct Big;
        impl Hex for Big {
            fn from_hex(_: &str) -> Result<Self, HexError> {
                Ok(Big)
            }

            fn to_hex(&self) -> Result<String, HexError> {
                Err(HexError::TooLarge {})
            }
        }
        let marker = Big.to_hex_trimmed();
        assert_eq!(marker, "…(too large)");
        assert!(from_hex(&marker).is_err());
    }

    #[test]
    fn character_error() {
        let result = from_hex("1234567890ABCDEFG1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, HexError::InvalidCharacter { .. }));
        assert_eq!(
            err.to_string(),
            "Only hexadecimal characters (0-9, a-f, A-F) are permitted, with an optional `0x` prefix and surrounding \
             whitespace; canonical decoding permits only 0-9 and a-f"
        );
    }
}
