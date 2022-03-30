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

//! A trait that handles [base58](https://crates.io/crates/base58-monero) encoding and decoding.

use thiserror::Error;

use crate::{ByteArray, ByteArrayError};

/// Trait for encoding/decoding to base58.
pub trait Base58 {
    /// Convert from base58 string.
    fn from_base58(hex: &str) -> Result<Self, Base58Error>
    where Self: Sized;

    /// Convert to base58 string.
    fn to_base58(&self) -> String;
}

/// Errors for trait Base58.
#[derive(Debug, Error)]
pub enum Base58Error {
    #[error("Byte array error: {0}")]
    ByteArrayError(#[from] ByteArrayError),
    #[error("Decode error: {0}")]
    DecodeError(#[from] base58_monero::Error),
}

impl<T: ByteArray> Base58 for T {
    fn from_base58(data: &str) -> Result<Self, Base58Error>
    where Self: Sized {
        let bytes = base58_monero::decode(data)?;
        Self::from_bytes(&bytes).map_err(Into::into)
    }

    fn to_base58(&self) -> String {
        base58_monero::encode(self.as_bytes()).expect("base58_monero::encode is infallible")
    }
}

#[cfg(test)]
mod test {
    use rand::{rngs::OsRng, RngCore};

    use super::*;

    #[test]
    fn decoding() {
        assert_eq!(Vec::from_base58("111111").unwrap(), vec![0; 4]);
        assert_eq!(Vec::from_base58("11115Q").unwrap(), vec![0, 0, 0, 255]);
        assert!(Vec::from_base58("11111O").is_err());
        assert!(Vec::from_base58("🖖🥴").is_err());
    }

    #[test]
    fn encoding() {
        assert_eq!(vec![0; 4].to_base58(), "111111");
        assert_eq!(vec![0, 2, 250, 39].to_base58(), "111zzz");
    }

    #[test]
    fn inverse_operations() {
        let mut bytes = vec![0; 10];
        OsRng.fill_bytes(&mut bytes);
        assert_eq!(Vec::from_base58(&bytes.to_base58()).unwrap(), bytes);
    }
}
