// Copyright 2019. The Tari Project
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

//! Data structure representing time as a `u64`.

use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSerialize};

/// The timestamp, defined as the amount of seconds past from UNIX epoch.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize,))]
pub struct EpochTime(u64);

impl EpochTime {
    /// Return UTC current as EpochTime.
    ///
    /// Returns `EpochTime(0)` if the system clock is set before the UNIX epoch, and panics in debug builds in that
    /// case. Use [EpochTime::try_now] to handle that case explicitly.
    pub fn now() -> EpochTime {
        let now = Self::try_now();
        debug_assert!(now.is_some(), "System clock is set before the UNIX epoch");
        now.unwrap_or(EpochTime(0))
    }

    /// Return UTC current as EpochTime, or `None` if the system clock is set before the UNIX epoch.
    pub fn try_now() -> Option<EpochTime> {
        let elapsed = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
        Some(EpochTime(elapsed.as_secs()))
    }

    /// Creates a new EpochTime representing the number of seconds since the unix epoch (1970-01-01 00:00:00 UTC).
    pub fn from_secs_since_epoch(secs: u64) -> EpochTime {
        EpochTime(secs)
    }

    /// Return the EpochTime as a u64.
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Checked EpochTime addition. Computes self + other, returning None if overflow occurred.
    pub fn checked_add(self, other: EpochTime) -> Option<EpochTime> {
        self.0.checked_add(other.0).map(EpochTime)
    }

    /// Checked EpochTime subtraction. Computes self - other, returning None if overflow occurred.
    pub fn checked_sub(self, other: EpochTime) -> Option<EpochTime> {
        self.0.checked_sub(other.0).map(EpochTime)
    }
}
impl fmt::Display for EpochTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for EpochTime {
    fn from(value: u64) -> Self {
        EpochTime(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn now() {
        let a = EpochTime::now();
        let b = EpochTime::now();
        assert!(a <= b);
        let c = EpochTime::try_now().unwrap();
        assert!(b <= c);
        assert!(c.as_u64() > 0);
    }

    #[test]
    fn as_u64() {
        let time = EpochTime::from(1234567);
        assert_eq!(time.as_u64(), 1234567);
    }

    #[test]
    fn checked_add() {
        let a = EpochTime::from(1111);
        let b = EpochTime::from_secs_since_epoch(123);
        assert_eq!(a.checked_add(b), Some(EpochTime::from(1234)));
        let b = EpochTime::from(u64::MAX);
        assert_eq!(a.checked_add(b), None);
    }

    #[test]
    fn checked_sub() {
        let a = EpochTime::from(1234);
        let b = EpochTime::from(123);
        assert_eq!(a.checked_sub(b), Some(EpochTime::from(1111)));
        assert_eq!(b.checked_sub(a), None);
    }

    #[test]
    fn display() {
        let time = EpochTime::from(1234567);
        assert_eq!("1234567", format!("{}", time));
    }
}
