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

//! A set of useful and commonly used utilities that are used in several places in the Tari project.

#![no_std]
// This is to allow no_std use
#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[cfg(any(feature = "epoch", feature = "test"))]
#[macro_use]
extern crate std;

pub mod bit;
pub mod byte_array;
pub mod convert;
#[cfg(feature = "monero")]
pub mod encoding;
#[cfg(feature = "epoch")]
pub mod epoch_time;
pub mod fixed_set;
pub mod hash;
pub mod hex;
#[cfg(feature = "zeroize")]
pub mod hidden;
pub mod locks;
#[cfg(feature = "serde")]
pub mod message_format;
#[cfg(feature = "zeroize")]
pub mod password;
#[cfg(feature = "subtle")]
pub mod safe_array;
#[cfg(feature = "serde")]
pub mod serde;
pub use self::{
    byte_array::{ByteArray, ByteArrayError},
    hash::Hashable,
};
#[cfg(feature = "zeroize")]
pub use self::{hidden::Hidden, password::SafePassword};
