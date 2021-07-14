use std::mem;

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

use std::mem;

/// Converts a single input byte to 8 bits (little-endian)
pub fn byte_to_bits(value: u8) -> [bool; 8] {
    let mut bits = [false; 8];
    for i in 0..8 {
        bits[i] = value & (1 << i) != 0;
    }
    (bits)
}

/// Converts a single input integer to a vector of bits (little-endian)
/// Returns None if `bit_count` is higher than the number of bits in usize.
pub fn checked_uint_to_bits(value: usize, bit_count: usize) -> Option<Vec<bool>> {
    if bit_count > mem::size_of::<usize>() * 8 {
        return None;
    }

    let mut bits = vec![false; bit_count];
    for i in 0..bit_count {
        bits[i] = value & (1 << i) != 0;
    }
    Some(bits)
}

/// Converts a array of input bits (little-endian) to a single byte
pub fn bits_to_byte(bits: [bool; 8]) -> u8 {
    let mut value: u8 = 0;
    for i in 0..8 {
        value |= (bits[i] as u8) << i;
    }
    (value)
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
            value |= (bits[i] as usize) << i;
        }
        Some(value)
    }
}

/// Converts a vector of input bytes to a vector of bits
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits: Vec<bool> = vec![false; bytes.len() * 8];
    for i in 0..bytes.len() {
        let bit_index = i * 8;
        bits[bit_index..(bit_index + 8)].copy_from_slice(&byte_to_bits(bytes[i]));
    }
    (bits)
}

/// Converts a vector of bits to a vector of bytes
pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![0; bits.len() / 8];
    let mut curr_bits: [bool; 8] = [false; 8];
    for i in 0..bytes.len() {
        let byte_index = i * 8;
        curr_bits.copy_from_slice(&bits[byte_index..(byte_index + 8)]);
        bytes[i] = bits_to_byte(curr_bits) as u8;
    }
    (bytes)
}

#[cfg(test)]
mod test {
    use super::*;

    mod checked_uint_to_bits {
        use super::*;
        use std::convert;

        #[test]
        fn it_returns_empty_vec_if_bit_count_is_zero() {
            assert!(checked_uint_to_bits(std::usize::MAX, 0).unwrap().is_empty());
        }

        #[test]
        fn it_returns_none_given_too_many_bits() {
            assert!(checked_uint_to_bits(std::usize::MAX, 1337).is_none());
        }

        #[test]
        fn it_returns_bits_for_max_value() {
            let bits = checked_uint_to_bits(std::usize::MAX, mem::size_of::<usize>() * 8).unwrap();
            assert!(bits.into_iter().all(convert::identity));
        }
    }
}
