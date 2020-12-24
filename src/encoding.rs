use crate::{ByteArray, ByteArrayError};
use thiserror::Error;

pub trait Base58 {
    fn from_base58(hex: &str) -> Result<Self, Base58Error>
    where Self: Sized;

    fn to_base58(&self) -> String;
}

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
    use super::*;
    use rand::{rngs::OsRng, RngCore};

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
