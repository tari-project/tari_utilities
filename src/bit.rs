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
#![allow(clippy::needless_range_loop)]
//! Functions for conversion between integer and bit array.

use alloc::vec::Vec;
use core::{convert::TryInto, mem};

/// Converts a single input byte to 8 bits (little-endian).
pub fn byte_to_bits(value: u8) -> [bool; 8] {
    let mut bits = [false; 8];
    for (i, bit) in bits.iter_mut().enumerate() {
        *bit = value & (1 << i) != 0;
    }
    bits
}

/// Converts 8 bits (little-endian) back into a single byte.
pub fn bits_to_byte(bits: &[bool; 8]) -> u8 {
    let mut value = 0u8;
    for (i, bit) in bits.iter().enumerate() {
        value |= u8::from(*bit) << i;
    }
    value
}

/// Converts a vector of input bytes to a vector of bits
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    bytes.iter().flat_map(|byte| byte_to_bits(*byte)).collect()
}

/// Converts a slice of little-endian bits back into bytes, returning `None` if `bits.len()` is not a multiple of 8.
pub fn checked_bits_to_bytes(bits: &[bool]) -> Option<Vec<u8>> {
    if (bits.len() & 7) != 0 {
        return None;
    }
    bits.chunks(8)
        .map(|chunk| chunk.try_into().ok().map(|arr: [bool; 8]| bits_to_byte(&arr)))
        .collect()
}

/// Converts a vector of input bits (little-endian) to its integer representation
/// Returns None if the length of `bits` is greater than the number of bits in a `usize`, which would cause an attempt
/// to shift left with overflow
pub fn checked_bits_to_uint(bits: &[bool]) -> Option<usize> {
    const PTR_SIZE_BITS: usize = mem::size_of::<usize>() * 8;

    if bits.len() > PTR_SIZE_BITS {
        None
    } else {
        let mut value: usize = 0;
        for (i, bit) in bits.iter().enumerate() {
            value |= usize::from(*bit) << i;
        }
        Some(value)
    }
}

/// Converts a slice of input bits (little-endian) to a `u64`, returning `None` if `bits.len() > 64`.
pub fn checked_bits_to_u64(bits: &[bool]) -> Option<u64> {
    const U64_BITS: usize = u64::BITS as usize;

    if bits.len() > U64_BITS {
        None
    } else {
        let mut value: u64 = 0;
        for (i, bit) in bits.iter().enumerate() {
            value |= u64::from(*bit) << i;
        }
        Some(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_bytes_to_bits() {
        let bytes = [1u8, 127];
        let bits = bytes_to_bits(&bytes);
        assert_eq!(bits, [
            true, false, false, false, false, false, false, false, true, true, true, true, true, true, true, false
        ]);
    }

    #[test]
    pub fn test_checked_bits_to_uint() {
        let bits = [true, false, false, false, false, false, false, false];
        let uint = checked_bits_to_uint(&bits).unwrap();
        assert_eq!(uint, 1);
        let bits = [
            true, false, false, false, false, false, false, false, true, false, false, false, false, false, false,
            false,
        ];
        let uint = checked_bits_to_uint(&bits).unwrap();
        assert_eq!(uint, 257);
        let bits = [false; mem::size_of::<usize>() * 8 + 1];
        assert!(checked_bits_to_uint(&bits).is_none());
    }

    #[test]
    pub fn test_bits_to_byte_and_bytes_roundtrip() {
        for b in 0u8..=255 {
            let bits = byte_to_bits(b);
            assert_eq!(bits_to_byte(&bits), b);
        }

        let payload = [0u8, 1, 0x55, 0xaa, 127, 255];
        let bits = bytes_to_bits(&payload);
        assert_eq!(checked_bits_to_bytes(&bits).unwrap(), payload);
        assert_eq!(checked_bits_to_bytes(&[]).unwrap(), Vec::<u8>::new());
        assert!(checked_bits_to_bytes(&bits[..bits.len() - 1]).is_none());
    }

    #[test]
    pub fn test_checked_bits_to_u64() {
        assert_eq!(checked_bits_to_u64(&[]), Some(0));
        let max_bits = [true; 64];
        assert_eq!(checked_bits_to_u64(&max_bits), Some(u64::MAX));
        let overflow_bits = [false; 65];
        assert_eq!(checked_bits_to_u64(&overflow_bits), None);
    }
}
