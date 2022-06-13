use crate::byte_array::ByteArray;
use crate::hex::{from_hex, Hex};
use std::{fmt, marker::PhantomData};
use serde::{de::{Error, Visitor}, Deserializer, Serializer};

pub fn serialize<S, T>(data: &T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ByteArray,
{
    if ser.is_human_readable() {
        ser.serialize_str(&data.to_hex())
    } else {
        ser.serialize_bytes(&data.as_bytes())
    }
}

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
