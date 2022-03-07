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

use chrono::{DateTime, NaiveDateTime, Utc};
use newtype_ops::newtype_ops;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    ops::Div,
};
use thiserror::Error;
use time::OffsetDateTime;

/// The timestamp, defined as the amount of seconds past from UNIX epoch.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Deserialize, Serialize)]
pub struct EpochTime(u64);

impl EpochTime {
    /// return UTC current as EpochTime
    pub fn now() -> EpochTime {
        EpochTime(Utc::now().timestamp() as u64)
    }

    /// Return the EpochTime as a u64
    pub fn as_u64(self) -> u64 {
        self.0
    }

    /// Return a new EpochTime increased by the amount of seconds given.
    /// It will panic if combined EpochTime and seconds are larger than U64::MAX
    pub fn increase(self, seconds: u64) -> EpochTime {
        let num = seconds.checked_add(self.0);
        let value = match num {
            Some(v) => v,
            _ => panic!("u64 overflow in timestamp"),
        };
        EpochTime(value)
    }

    /// Checked EpochTime subtraction. Computes self - other, returning None if overflow occurred.
    pub fn checked_sub(self, other: EpochTime) -> Option<EpochTime> {
        match self.0.checked_sub(other.0) {
            None => None,
            Some(v) => Some(EpochTime(v)),
        }
    }
}

impl Default for EpochTime {
    fn default() -> Self {
        EpochTime(Utc::now().timestamp() as u64)
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

impl From<DateTime<Utc>> for EpochTime {
    fn from(value: DateTime<Utc>) -> Self {
        EpochTime(value.timestamp() as u64)
    }
}

impl From<EpochTime> for DateTime<Utc> {
    fn from(value: EpochTime) -> Self {
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(value.0 as i64, 0), Utc)
    }
}

#[derive(Error, Debug)]
pub enum TimeConvertError {
    #[error("Can't convert time range: {0}")]
    InvalidRange(#[from] time::error::ComponentRange),
    #[error("Can't convert a number into a time value: {0}")]
    NumError(#[from] std::num::TryFromIntError),
}

impl TryFrom<OffsetDateTime> for EpochTime {
    type Error = TimeConvertError;

    fn try_from(value: OffsetDateTime) -> Result<Self, TimeConvertError> {
        let sec = value.unix_timestamp().try_into()?;
        Ok(EpochTime(sec))
    }
}

impl TryFrom<EpochTime> for OffsetDateTime {
    type Error = TimeConvertError;

    fn try_from(value: EpochTime) -> Result<Self, TimeConvertError> {
        let sec = value.0.try_into()?;
        OffsetDateTime::from_unix_timestamp(sec).map_err(TimeConvertError::from)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_epoch_time() {
        assert_eq!(EpochTime::from(1_000) + EpochTime::from(8_000), EpochTime::from(9_000));
        assert_eq!(&EpochTime::from(15) + &EpochTime::from(5), EpochTime::from(20));
    }
}
