// Copyright 2019 The Tari Project
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

//! A trait that offers representation of data types as a byte array or hex string.

use snafu::prelude::*;

use crate::hex::{from_hex, to_hex, Hex, HexError};

/// Errors for [ByteArray] trait.
#[derive(Debug, Snafu, PartialEq, Eq)]
pub enum ByteArrayError {
    /// An array can't be parsed.
    #[snafu(display("Could not create a ByteArray when converting from a different format: `{reason}'"))]
    ConversionError { reason: String },
    /// The lenght doesn't fit to the array.
    #[snafu(display("The input data was the incorrect length to perform the desired conversion"))]
    IncorrectLength {},
}

/// Trait the allows converting to/from [array][[u8]]/[vec][[u8]].
#[allow(clippy::ptr_arg)]
pub trait ByteArray: Sized {
    /// Return the type as a byte vector.
    fn to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    /// Try and convert the given byte vector to the implemented type.
    ///
    /// # Errors
    ///
    /// Any failures (incorrect string length, etc) return an [ByteArrayError](enum.ByteArrayError.html) with an
    /// explanatory note.
    fn from_vec(v: &Vec<u8>) -> Result<Self, ByteArrayError> {
        Self::from_bytes(v.as_slice())
    }

    /// Try and convert the given byte array to the implemented type. Any failures (incorrect array length,
    /// implementation-specific checks, etc.) return a [ByteArrayError](enum.ByteArrayError.html) with an explanatory
    /// note.
    fn from_bytes(bytes: &[u8]) -> Result<Self, ByteArrayError>;

    /// Return the type as a byte array.
    fn as_bytes(&self) -> &[u8];
}

impl ByteArray for Vec<u8> {
    fn to_vec(&self) -> Vec<u8> {
        self.clone()
    }

    fn from_vec(v: &Vec<u8>) -> Result<Self, ByteArrayError> {
        Ok(v.clone())
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, ByteArrayError> {
        Ok(bytes.to_vec())
    }

    fn as_bytes(&self) -> &[u8] {
        Vec::as_slice(self)
    }
}

impl<const I: usize> ByteArray for [u8; I] {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ByteArrayError> {
        if bytes.len() != I {
            return Err(ByteArrayError::IncorrectLength {});
        }
        let mut a = [0u8; I];
        a.copy_from_slice(bytes);
        Ok(a)
    }

    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl<T: ByteArray> Hex for T {
    fn from_hex(hex: &str) -> Result<Self, HexError> {
        let v = from_hex(hex)?;
        Self::from_bytes(&v).map_err(|_| HexError::HexConversionError {})
    }

    fn to_hex(&self) -> String {
        to_hex(self.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_to_vec() {
        let v = vec![0u8, 1, 128, 255];
        let ba = <Vec<u8>>::from_vec(&v).unwrap();
        assert_eq!(ba.to_vec(), v);
    }

    #[test]
    fn from_to_array() {
        let v = vec![
            0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31,
        ];
        let ba = <[u8; 32]>::from_vec(&v).unwrap();
        assert_eq!(ba.to_vec(), v);
    }

    #[test]
    fn from_to_different_sizes() {
        let v4 = vec![0u8, 1, 2, 3];
        let a4 = <[u8; 4]>::from_vec(&v4).unwrap();
        assert_eq!(a4.to_vec(), v4);
        let v10 = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let a10 = <[u8; 10]>::from_vec(&v10).unwrap();
        assert_eq!(a10.to_vec(), v10);
        fn check(_: impl ByteArray) {}
        check([0; 32]);
        check([0; 64]);
        check([0; 1000]);
    }

    #[test]
    fn from_to_hex() {
        let v = <Vec<u8>>::from_hex("deadbeef").unwrap();
        assert_eq!(v.to_hex(), "deadbeef");
    }

    #[test]
    fn test_error_handling() {
        let err = <[u8; 32]>::from_bytes(&[1, 2, 3, 4]).unwrap_err();
        assert_eq!(err, ByteArrayError::IncorrectLength{});

        let err = <[u8; 32]>::from_hex("abcd").unwrap_err();
        assert!(matches!(err, HexError::HexConversionError{}));
    }
}
