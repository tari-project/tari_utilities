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
use tari_utilities::bit::checked_bits_to_uint;

#[test]
fn shift_left_overflow_bits_to_uint() {
    const PTR_SIZE_BITS: usize = mem::size_of::<usize>() * 8;

    let bits = [true];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, Some(1));

    let bits = [false, true];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, Some(2));

    let bits = [true, true];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, Some(3));

    let bits = [false; PTR_SIZE_BITS];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, Some(0));

    // lengths greater than the machine ptr size (typically 64bit) would overflow
    let bits = [false; PTR_SIZE_BITS + 1];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, None);

    let bits = [false; 128];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, None);

    let bits = [
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
    ];
    let result = checked_bits_to_uint(&bits);
    assert_eq!(result, None);
}
