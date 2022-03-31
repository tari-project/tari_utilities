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
    ops::Div,
    time::{SystemTime, UNIX_EPOCH},
};

use newtype_ops::newtype_ops;
use serde::{Deserialize, Serialize};

/// The timestamp, defined as the amount of seconds past from UNIX epoch.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct EpochTime(u64);

impl EpochTime {
    /// Return UTC current as EpochTime.
    pub fn now() -> EpochTime {
        EpochTime(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("SystemTime before UNIX EPOCH!")
                .as_secs(),
        )
    }

    /// Creates a new EpochTime representing the number of seconds since the unix epoch (1970-01-01 00:00:00 UTC).
    pub fn from_secs_since_epoch(secs: u64) -> EpochTime {
        EpochTime(secs)
    }

    /// Return the EpochTime as a u64.
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Return a new EpochTime increased by the amount of seconds given.
    ///
    /// # Panics
    ///
    /// It will panic if combined EpochTime and seconds are larger than U64::MAX.
    #[must_use]
    pub fn increase(self, seconds: u64) -> EpochTime {
        let value = seconds.checked_add(self.0).expect("u64 overflow in timestamp");
        EpochTime(value)
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

// You can only add or subtract EpochTime from EpochTime
newtype_ops! { [EpochTime] {add sub} {:=} Self Self }
newtype_ops! { [EpochTime] {add sub} {:=} &Self &Self }
newtype_ops! { [EpochTime] {add sub} {:=} Self &Self }

// Multiplication and division of EpochTime by scalar is EpochTime
newtype_ops! { [EpochTime] {mul div rem} {:=} Self u64 }

// Division of EpochTime by EpochTime is a EpochTime ratio (scalar) (newtype_ops doesn't handle this case)
impl Div for EpochTime {
    type Output = u64;

    fn div(self, rhs: Self) -> Self::Output {
        self.0 / rhs.0
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
    }

    #[test]
    fn as_u64() {
        let time = EpochTime::from(1234567);
        assert_eq!(time.as_u64(), 1234567);
    }

    #[test]
    fn increase() {
        let a = EpochTime::from(1111);
        assert_eq!(a.increase(123), EpochTime::from(1234));
    }

    #[test]
    #[should_panic]
    fn increase_overflow() {
        let a = EpochTime::from(1234);
        let _ = a.increase(u64::MAX);
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
    fn test_div() {
        let a = EpochTime::from(3639);
        let b = EpochTime::from(3);
        assert_eq!(1213, a / b);
    }

    #[test]
    fn display() {
        let time = EpochTime::from(1234567);
        assert_eq!("1234567", format!("{}", time));
    }

    #[test]
    fn add_epoch_time() {
        assert_eq!(EpochTime::from(1_000) + EpochTime::from(8_000), EpochTime::from(9_000));
        assert_eq!(EpochTime::from(15) + EpochTime::from(5), EpochTime::from(20));
    }
}
