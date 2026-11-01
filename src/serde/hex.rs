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

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{fmt, marker::PhantomData};

use serde::{
    Deserializer,
    Serializer,
    de::{Error, Visitor},
    ser::Error as _,
};
use zeroize::Zeroizing;

use crate::{
    byte_array::ByteArray,
    hex::{Hex, from_hex},
};

/// Serializes a [`ByteArray`] to a hex string or a binary array.
pub fn serialize<S, T>(data: &T, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ByteArray,
{
    if ser.is_human_readable() {
        let hex = Zeroizing::new(data.to_hex().map_err(S::Error::custom)?);
        ser.serialize_str(&hex)
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
        let bytes = Zeroizing::new(from_hex(v).map_err(|e| E::custom(e.to_string()))?);
        self.visit_bytes(&bytes)
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where E: Error {
        let v = Zeroizing::new(v);
        self.visit_str(&v)
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where E: Error {
        let v = Zeroizing::new(v);
        self.visit_bytes(&v)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where E: Error {
        T::from_canonical_bytes(v).map_err(|e| E::custom(e.to_string()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where E: Error {
        self.visit_bytes(v)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec::Vec};

    use serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
        de::{Error as _, Visitor, value::Error as ValueError},
    };

    struct BinarySerializer;

    impl Serializer for BinarySerializer {
        type Error = ValueError;
        type Ok = Vec<u8>;
        type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn is_human_readable(&self) -> bool {
            false
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Ok(v.to_vec())
        }

        fn serialize_newtype_struct<T: ?Sized + Serialize>(
            self,
            _name: &'static str,
            value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            value.serialize(self)
        }

        fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_newtype_variant<T: ?Sized + Serialize>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error> {
            unimplemented!()
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            unimplemented!()
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            unimplemented!()
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            unimplemented!()
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            unimplemented!()
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            unimplemented!()
        }

        fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
            unimplemented!()
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            unimplemented!()
        }
    }

    struct BinaryDeserializer<'de>(&'de [u8]);

    impl<'de> Deserializer<'de> for BinaryDeserializer<'de> {
        type Error = ValueError;

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            byte_buf option unit unit_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }

        fn is_human_readable(&self) -> bool {
            false
        }

        fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
            Err(ValueError::custom("unexpected deserialize_any"))
        }

        fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            visitor.visit_borrowed_bytes(self.0)
        }

        fn deserialize_newtype_struct<V: Visitor<'de>>(
            self,
            _name: &'static str,
            visitor: V,
        ) -> Result<V::Value, Self::Error> {
            visitor.visit_newtype_struct(self)
        }
    }

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

    #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
    struct HexOrBytesVec(#[serde(with = "super")] Vec<u8>);

    #[test]
    fn serialize_too_large_is_an_error() {
        let big = HexOrBytesVec(vec![0u8; 262_145]);
        let err = serde_json::to_string(&big).unwrap_err();
        assert!(err.to_string().contains("262144"));
        // Binary formats are not limited
        assert!(big.serialize(BinarySerializer).is_ok());
    }

    #[test]
    fn deserialize_uppercase_and_reject_sign() {
        let restored: HexOrBytes = serde_json::from_str("\"010203FF\"").unwrap();
        assert_eq!(restored, HexOrBytes([1, 2, 3, 255]));
        assert!(serde_json::from_str::<HexOrBytes>("\"+1020304\"").is_err());
    }

    #[test]
    fn check_serde_hex_binary() {
        let hex_or_bytes = HexOrBytes([1, 2, 3, 255]);
        let expected: Vec<u8> = vec![1, 2, 3, 255];
        assert_eq!(hex_or_bytes.serialize(BinarySerializer).unwrap(), expected);
        let restored = HexOrBytes::deserialize(BinaryDeserializer(&expected)).unwrap();
        assert_eq!(hex_or_bytes, restored);
    }
}
