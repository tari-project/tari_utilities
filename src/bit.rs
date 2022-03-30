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

use std::mem;

/// Converts a single input byte to 8 bits (little-endian).
pub fn byte_to_bits(value: u8) -> [bool; 8] {
    let mut bits = [false; 8];
    for i in 0..8 {
        bits[i] = value & (1 << i) != 0;
    }
    bits
}

/// Converts a vector of input bytes to a vector of bits
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits: Vec<bool> = vec![false; bytes.len() * 8];
    for i in 0..bytes.len() {
        let bit_index = i * 8;
        bits[bit_index..(bit_index + 8)].copy_from_slice(&byte_to_bits(bytes[i]));
    }
    bits
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
        for i in 0..bits.len() {
            value |= usize::from(bits[i]) << i;
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
}
