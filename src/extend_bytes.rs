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

//! A trait allows us to call append_raw_bytes and get the raw bytes of the type.

use chrono::{DateTime, Utc};

/// A trait allows us to call append_raw_bytes and get the raw bytes of the type
pub trait ExtendBytes {
    fn append_raw_bytes(&self, buf: &mut Vec<u8>);
}

macro_rules! array_type_impl {
    ($($t:ty)*) => ($(
        impl<T> ExtendBytes for $t
        where T: ExtendBytes
        {
            fn append_raw_bytes(&self, buf: &mut Vec<u8>) {
                for t in self {
                    t.append_raw_bytes(buf);
                }
            }
        }
    )*)
}

array_type_impl! { Vec<T> [T] }

macro_rules! string_type_impl {
    ($($t:ty)*) => ($(
        impl ExtendBytes for $t {
            fn append_raw_bytes(&self, buf: &mut Vec<u8>) {
                buf.extend(self.as_bytes())
            }
        }
    )*)
}

string_type_impl! { str &str String }

macro_rules! int_type_impl {
    ($($t:ty)*) => ($(
        impl ExtendBytes for $t {
            fn append_raw_bytes(&self, buf: &mut Vec<u8>) {
                let bytes = self.to_le_bytes();
                buf.extend_from_slice(&bytes);
            }
        }
    )*)
}

int_type_impl! { i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 }

impl ExtendBytes for bool {
    fn append_raw_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(if *self { &[1u8] } else { &[0u8] });
    }
}

impl ExtendBytes for DateTime<Utc> {
    fn append_raw_bytes(&self, buf: &mut Vec<u8>) {
        let bytes = self.timestamp().to_le_bytes();
        buf.extend_from_slice(&bytes);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn integer_type_case() {
        let mut buf: Vec<u8> = vec![];

        macro_rules! int_test_impl {
            ($($t:ty)*) => ($(
                buf.clear();
                let sandbox = 1 as $t;
                sandbox.append_raw_bytes(buf.as_mut());
                assert_eq!(buf, sandbox.to_le_bytes());
            )*)
        }

        int_test_impl! { i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 }
    }

    #[test]
    fn string_type_case() {
        let mut buf: Vec<u8> = vec![];

        macro_rules! string_test_impl {
            ($($t:ty)*) => ($(
                buf.clear();
                let sandbox = <$t>::from("hello");
                sandbox.append_raw_bytes(buf.as_mut());
                assert_eq!(buf, sandbox.as_bytes());
            )*)
        }

        // &str and String case
        string_test_impl! { &str String }

        // str case
        buf.clear();
        (*String::from("hello")).append_raw_bytes(buf.as_mut());
        assert_eq!(buf, (*String::from("hello")).as_bytes());
    }

    #[test]
    fn bool_type_case() {
        let mut buf: Vec<u8> = vec![];

        // bool case of true
        buf.clear();
        let sandbox = true;
        sandbox.append_raw_bytes(buf.as_mut());
        assert_eq!(buf, &[1u8]);

        // bool case of false
        buf.clear();
        let sandbox = false;
        sandbox.append_raw_bytes(buf.as_mut());
        assert_eq!(buf, &[0u8]);
    }

    #[test]
    fn datetime_utc_case() {
        let mut buf: Vec<u8> = vec![];

        buf.clear();
        let sandbox = "2020-02-02T0:0:0.0Z".parse::<DateTime<Utc>>().unwrap();
        sandbox.append_raw_bytes(buf.as_mut());
        assert_eq!(buf, sandbox.timestamp().to_le_bytes());
    }

    #[test]
    fn array_type_case() {
        let mut buf: Vec<u8> = vec![];

        // Simple Vec<T> case where T is u8
        buf.clear();
        let sandbox = vec![1u8, 2u8, 3u8, 4u8];
        sandbox.append_raw_bytes(buf.as_mut());
        assert_eq!(buf, &[1u8, 2u8, 3u8, 4u8]);

        // Simple [T] case where T is u8
        buf.clear();
        let sandbox = [1u8, 2u8, 3u8, 4u8];
        sandbox.append_raw_bytes(buf.as_mut());
        assert_eq!(buf, &[1u8, 2u8, 3u8, 4u8]);
    }
}
