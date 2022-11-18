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
//! differentiation to avoid it being misused in an unintended context. This library provides a generic type and macro
//! that can help.

use std::{
    any::type_name,
    fmt,
    ops::{Deref, DerefMut},
};

use serde::Deserialize;
use zeroize::Zeroize;

/// This is a macro that produces a hidden type from an underlying data type.
///
/// It is a thin wrapper around `Hidden` and retains its useful properties:
/// - The data is not subject to `Debug` or `Display` output, which are masked.
/// - The data can only be accessed by (immutable or mutable) reference.
/// - The data zeroizes when dropped, and can also be manually zeroized.
/// - Cloning is safe.
///
/// The macro is a useful way to generate a hidden type that is subject to the compiler's type guarantees.
/// This can be useful if you need multiple hidden types that use the same underlying data type, but shouldn't be
/// confused for each other.
///
/// Note that it may not be safe to dereference the hidden data if its type implements `Copy`.
/// If the type does not implement `Copy`, you should be fine.
/// If it does, avoid dereferencing.
///
/// ```edition2018
/// # #[macro_use] extern crate tari_utilities;
/// # use tari_utilities::Hidden;
/// # use zeroize::Zeroize;
/// # fn main() {
/// // Define a hidden type with a `[u8; 32]` data type
/// hidden_type!(MyHiddenType, [u8; 32]);
///
/// // Instantiate using existing data
/// let mut example = MyHiddenType::from([1u8; 32]);
///
/// // Access the data by immutable reference
/// assert_eq!(example.reveal(), &[1u8; 32]);
///
/// // Access the data by mutable reference
/// let example_mut_ref = example.reveal_mut();
/// *example_mut_ref = [42u8; 32];
/// assert_eq!(example.reveal(), &[42u8; 32]);
///
/// // Clone the data safely
/// let mut example_clone = example.clone();
///
/// // Zeroize the data manually
/// example_clone.zeroize();
/// assert_eq!(example_clone.reveal(), &[0u8; 32]);
/// # }
/// ```
#[macro_export]
macro_rules! hidden_type {
    ($name:ident, $type:ty) => {
        /// A hidden type
        #[derive(Clone, Debug, Zeroize)]
        pub struct $name {
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

        impl From<$type> for $name {
            /// Hide existing data
            fn from(t: $type) -> Self {
                Self {
                    data: Hidden::hide(t),
                }
            }
        }
    };
}

/// A generic type for data that needs to be kept hidden and zeroized when out of scope, and is accessible only by
/// reference.
///
/// You can define a hidden type using any underlying data type that implements `Zeroize`.
/// This is the case for most basic types that you probably care about.
///
/// Hidden data has useful properties:
/// - The data is not subject to `Debug` or `Display` output, which are masked.
/// - The data can only be accessed by (immutable or mutable) reference.
/// - The data zeroizes when dropped, and can also be manually zeroized.
/// - Cloning is safe.
///
/// Note that it may not be safe to dereference the hidden data if its type implements `Copy`.
/// If the type does not implement `Copy`, you should be fine.
/// If it does, avoid dereferencing.
///
/// Hidden data supports transparent deserialization, but you'll need to implement serialization yourself if you need
/// it.
///
/// ```edition2018
/// # use tari_utilities::hidden::Hidden;
/// # use zeroize::Zeroize;
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
/// let mut hidden_in_place = Hidden::<[u8; 32]>::hide([0u8; 32]);
/// let hidden_in_place_mut_ref = hidden_in_place.reveal_mut();
/// *hidden_in_place_mut_ref = [42u8; 32];
/// assert_eq!(hidden_in_place.reveal(), &[42u8; 32]);
///
/// // Cloning is safe to do
/// let mut clone = hidden_in_place.clone();
/// assert_eq!(hidden_in_place.reveal(), clone.reveal());
///
/// // You can manually zeroize the data if you need to
/// clone.zeroize();
/// assert_eq!(clone.reveal(), &[0u8; 32]);
/// ```
#[derive(Clone, Deserialize)]
#[serde(transparent)]
pub struct Hidden<T>
where T: Zeroize
{
    inner: Box<T>,
}

impl<T> Hidden<T>
where T: Zeroize
{
    /// Create new hidden data from the underlying type
    pub fn hide(inner: T) -> Self {
        Self { inner: Box::new(inner) }
    }

    /// Reveal the hidden data as an immutable reference
    pub fn reveal(&self) -> &T {
        self.inner.deref()
    }

    /// Reveal the hidden data as a mutable reference
    pub fn reveal_mut(&mut self) -> &mut T {
        self.inner.deref_mut()
    }
}

/// Only output masked data for debugging, keeping the hidden data hidden
impl<T> fmt::Debug for Hidden<T>
where T: Zeroize
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}>", type_name::<T>())
    }
}

/// Only display masked data, keeping the hidden data hidden
impl<T> fmt::Display for Hidden<T>
where T: Zeroize
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}>", type_name::<T>())
    }
}

/// Zeroize the hidden data
impl<T> Zeroize for Hidden<T>
where T: Zeroize
{
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

/// Zeroize the hidden data when dropped
impl<T> Drop for Hidden<T>
where T: Zeroize
{
    fn drop(&mut self) {
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references() {
        // Check immutable reference
        assert_eq!(Hidden::hide(0u8).reveal(), &0u8);

        // Check mutable reference
        let mut hidden = Hidden::hide([0u8; 32]);
        let hidden_mut_ref = hidden.reveal_mut();

        *hidden_mut_ref = [42u8; 32];
        assert_eq!(hidden.reveal(), &[42u8; 32]);
    }

    #[test]
    fn deserialize() {
        let value = 1u8;
        let hidden = Hidden::<u8>::hide(value);

        let ser = value.to_string();

        let deser: Hidden<u8> = serde_json::from_str(&ser).unwrap();
        assert_eq!(hidden.reveal(), deser.reveal());
    }

    #[test]
    fn masking() {
        let hidden = Hidden::<u8>::hide(1u8);
        let formatted = format!("{}", hidden);
        let expected = format!("Hidden<{}>", type_name::<u8>());
        assert_eq!(formatted, expected);
    }

    #[test]
    fn macro_types() {
        hidden_type!(TypeA, [u8; 32]);
        hidden_type!(TypeB, [u8; 32]);
        let a = TypeA::from([1u8; 32]);
        let b = TypeB::from([1u8; 32]);

        assert_eq!(a.reveal(), &[1u8; 32]);
        assert_eq!(b.reveal(), &[1u8; 32]);
    }
}
