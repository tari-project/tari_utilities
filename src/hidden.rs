// Copyright 2022. The Tari Project
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

//! Sometimes you need to handle sensitive data, like a passphrase or key material.
//! There are pitfalls here that you want to avoid: the data should be zeroized when it goes out of scope, shouldn't be
//! displayed or output to debug logs, shouldn't be unintentionally copied or moved, and often should have strict type
//! differentiation to avoid it being misused in an unintended context. This library provides a generic type that can
//! help.

use std::{any::type_name, fmt};

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// This is a macro that produces a hidden type.
/// You can use it to generate multiple hidden types that can't be unintentionally mixed up in things like function
/// definitions.
///
/// ```edition2018
/// # #[macro_use] extern crate tari_utilities;
/// # use tari_utilities::Hidden;
/// # use zeroize::Zeroize;
/// # fn main() {
/// hidden_type!(MyHiddenType, [u8; 32]);
/// let example = MyHiddenType::default();
/// assert_eq!(example.reveal(), &[0u8; 32]);
/// # }
/// ```
#[macro_export]
macro_rules! hidden_type {
    ($name:ident, $type:ty) => {
        /// A hidden type
        #[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Zeroize)]
        pub struct $name
        where
            $type: Default + Zeroize,
        {
            data: Hidden<$type>,
        }

        impl $name {
            /// Get an immutable reference to the data
            #[allow(dead_code)]
            pub fn reveal(&self) -> &$type {
                self.data.reveal()
            }

            /// Get a mutable reference to the data
            #[allow(dead_code)]
            pub fn reveal_mut(&mut self) -> &mut $type {
                self.data.reveal_mut()
            }
        }
    };
}

/// A generic type for data that needs to be kept hidden and zeroized when out of scope
///
/// You can define a hidden type using any underlying type that supports `Default` and `Zeroize`.
/// This is the case for most basic types that you probably care about.
///
/// Hidden data can't be copied, which is an intentional design decision, and you can only access it as a reference.
/// However, it supports other functionality transparently: cloning, equality, ordering, serialization, and the like.
///
/// ```edition2018
/// # use rand::rngs::OsRng;
/// # use rand::RngCore;
/// # use tari_utilities::hidden::Hidden;
///
/// // In this example, we need to handle secret data of type `[u8; 32]`.
///
/// // We can create hidden data from existing data; in this case, it's the caller's responsibility to make sure the existing data is handled securely
/// let hidden_from_data = Hidden::<[u8; 32]>::hide([1u8; 32]);
///
/// // We can access the hidden data as a reference, but not take ownership of it
/// assert_eq!(hidden_from_data.reveal(), &[1u8; 32]);
///
/// // We can create default hidden data and then modify it as a mutable reference; this is common for functions that act on data in place
/// let mut rng = OsRng;
/// let mut hidden_in_place = Hidden::<[u8; 32]>::default();
/// rng.fill_bytes(hidden_in_place.reveal_mut());
///
/// // Cloning is safe to do
/// let clone = hidden_in_place.clone();
/// assert_eq!(hidden_in_place.reveal(), clone.reveal());
/// ```
#[derive(Clone, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Hidden<T>
where
    T: Zeroize,
{
    inner: Box<T>,
}

impl<T> Hidden<T>
where
    T: Zeroize,
{
    /// Create new hidden data from the underlying type
    pub fn hide(inner: T) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    /// Reveal the hidden data as an immutable reference
    pub fn reveal(&self) -> &T {
        &self.inner
    }

    /// Reveal the hidden data as a mutable reference
    pub fn reveal_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

/// Only output masked data for debugging, keeping the hidden data hidden
impl<T> fmt::Debug for Hidden<T>
where
    T: Zeroize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}>", type_name::<T>())
    }
}

/// Only display masked data, keeping the hidden data hidden
impl<T> fmt::Display for Hidden<T>
where
    T: Zeroize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}>", type_name::<T>())
    }
}

/// Zeroize the hidden data
impl<T> Zeroize for Hidden<T>
where
    T: Zeroize,
{
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

/// Zeroize the hidden data when dropped
impl<T> Drop for Hidden<T>
where
    T: Zeroize,
{
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality() {
        assert_eq!(Hidden::<u8>::hide(0u8), Hidden::<u8>::hide(0u8));
        assert_ne!(Hidden::<u8>::hide(0u8), Hidden::<u8>::hide(1u8));
    }

    #[test]
    fn reference() {
        assert_eq!(Hidden::<u8>::hide(0u8).reveal(), &0u8);
        assert_ne!(Hidden::<u8>::hide(0u8).reveal(), &1u8);
    }

    #[test]
    fn serialize() {
        let hidden = Hidden::<u8>::hide(1u8);
        let ser = serde_json::to_string(&hidden).unwrap();

        let deser: Hidden<u8> = serde_json::from_str(&ser).unwrap();
        assert_eq!(hidden, deser);
    }

    #[test]
    fn masking() {
        let hidden = Hidden::<u8>::hide(1u8);
        let formatted = format!("{}", hidden);
        let expected = format!("Hidden<{}>", type_name::<u8>());
        assert_eq!(formatted, expected);
    }

    #[test]
    fn types() {
        hidden_type!(TypeA, [u8; 32]);
        hidden_type!(TypeB, [u8; 32]);
        let a = TypeA::default();
        let b = TypeB::default();

        assert_eq!(a.reveal(), b.reveal());
    }
}
