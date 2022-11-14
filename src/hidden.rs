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

use std::{any::type_name, fmt, marker::PhantomData};

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A marker trait for labeling different hidden types
pub trait HiddenLabel {}

/// This is a macro that produces a label for a hidden type.
/// You can use it to generate multiple hidden types that can't be unintentionally mixed up in things like function
/// definitions. You shouldn't ever need to access it directly once you define your hidden type, since it only exists to
/// keep types in order.
///
/// Its use is optional; as shown elsewhere in this documentation, you can define a hidden type without the use of a
/// label, in which a default one will be applied for you. But if you do that, you don't get the type enforcement
/// benefits.
///
/// ```edition2018
/// # #[macro_use] extern crate tari_utilities;
/// # use tari_utilities::hidden::HiddenLabel;
/// # use tari_utilities::hidden_label;
/// # fn main() {
/// // This makes `MyHiddenTypeLabel` available for later use when defining a hidden type
/// hidden_label!(MyHiddenTypeLabel);
/// # }
/// ```
#[macro_export]
macro_rules! hidden_label {
    ($name:ident) => {
        /// A hidden type label
        #[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name;

        impl HiddenLabel for $name {}
    };
}

// This is the hidden type label that is used internally if you define a hidden type without proving your own label
hidden_label!(DefaultHiddenLabel);

/// A generic type for data that needs to be kept hidden, needs to be zeroized when out of scope, and is resistant to
/// misuse.
///
/// You can define a hidden type using any underlying type that supports `Default` and `Zeroize`.
/// This is the case for most basic types that you probably care about.
///
/// You can optionally supply a label to the hidden type, which lets you define multiple hidden types that can't be
/// interchanged. To do this, use the `hidden_label!` macro to define a label, and then use it when defining your hidden
/// type. If you don't include a label, a default one is used for you.
///
/// Hidden data can't be copied, which is an intentional design decision, and you can only access it as a reference.
/// However, it supports other functionality transparently: cloning, equality, ordering, serialization, and the like.
/// Note an important fact about serialization: it's transparent, so the type label isn't serialized.
/// This means you should be sure to know the label you intend during deserialization!
///
/// ```edition2018
/// # use rand::rngs::OsRng;
/// # use rand::RngCore;
/// # use tari_utilities::hidden::{Hidden, HiddenLabel};
/// # use tari_utilities::hidden_label;
///
/// // In this example, we need to handle keys for a cipher that are `[u8; 32]` byte arrays
///
/// // Define a label for the new hidden type
/// // This ensures that any other `[u8; 32]` hidden types can't get confused with this one
/// hidden_label!(CipherKeyLabel);
///
/// // Alias a new hidden type using the data type and label
/// type CipherKey = Hidden<[u8; 32], CipherKeyLabel>;
///
/// // We can create hidden data from existing data; in this case, it's the caller's responsibility to make sure the existing data is handled securely
/// let key_from_data = CipherKey::hide([1u8; 32]);
///
/// // We can access the hidden data as a reference, but not take ownership of it
/// assert_eq!(key_from_data.reveal(), &[1u8; 32]);
///
/// // We can create default hidden data and then modify it as a mutable reference; this is common for functions that act on data in place
/// let mut rng = OsRng;
/// let mut key_in_place = CipherKey::default();
/// rng.fill_bytes(key_in_place.reveal_mut());
///
/// // Cloning is safe to do
/// let clone = key_in_place.clone();
/// assert_eq!(key_in_place.reveal(), clone.reveal());
/// ```
///
/// Type enforcement means the compiler won't let you mix and match hidden types with different labels.
/// This can be important when dealing with different types of key material, for example.
///
/// ```compile_fail
/// # use tari_utilities::hidden::{Hidden, HiddenLabel};
/// # use tari_utilities::hidden_label;
///
/// // These are labels for different hidden types we want to create; in this case, keys for two different ciphers
/// hidden_label!(KeyForCipherALabel);
/// hidden_label!(KeyForCipherBLabel);
///
/// // Both ciphers have keys that are `[u8; 32]` under the hood
/// type KeyForCipherA = Hidden<[u8; 32], KeyForCipherALabel>;
/// type KeyForCipherB = Hidden<[u8; 32], KeyForCipherBLabel>;
///
/// // Create a key for each cipher; notice the underlying data is the same for both
/// let key_a = KeyForCipherA::hide([1u8; 32]);
/// let key_b = KeyForCipherB::hide([1u8; 32]);
///
/// // The compiler won't let us treat them the same; this won't build
/// assert_eq!(key_a, key_b);
/// ```
///
/// But if you choose not to use a label, you can mix and match hidden types. Be careful if you do this!
///
/// ```edition2018
/// # use tari_utilities::hidden::{Hidden, HiddenLabel};
/// # use tari_utilities::hidden_label;
///
/// // Define two types with no labels; you probably don't want to actually do this!
/// type TypeA = Hidden<[u8; 32]>;
/// type TypeB = Hidden<[u8; 32]>;
///
/// // Create an instance of both
/// let a = TypeA::default();
/// let b = TypeB::default();
///
/// // You can mix and match these! Be sure that's what you actually intended.
/// assert_eq!(a, b);
/// ```
#[derive(Clone, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Hidden<T, L = DefaultHiddenLabel>
where
    T: Zeroize,
    L: HiddenLabel,
{
    inner: Box<T>,
    #[serde(skip)]
    _type: PhantomData<L>,
}

impl<T, L> Hidden<T, L>
where
    T: Zeroize,
    L: HiddenLabel,
{
    /// Create new hidden data from the underlying type
    pub fn hide(inner: T) -> Self {
        Self {
            inner: Box::new(inner),
            _type: PhantomData,
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
impl<T, L> fmt::Debug for Hidden<T, L>
where
    T: Zeroize,
    L: HiddenLabel,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}, {}>", type_name::<T>(), type_name::<L>(),)
    }
}

/// Only display masked data, keeping the hidden data hidden
impl<T, L> fmt::Display for Hidden<T, L>
where
    T: Zeroize,
    L: HiddenLabel,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}, {}>", type_name::<T>(), type_name::<L>(),)
    }
}

/// Zeroize the hidden data
impl<T, L> Zeroize for Hidden<T, L>
where
    T: Zeroize,
    L: HiddenLabel,
{
    fn zeroize(&mut self) {
        self.inner.zeroize();
    }
}

/// Zeroize the hidden data when dropped
impl<T, L> Drop for Hidden<T, L>
where
    T: Zeroize,
    L: HiddenLabel,
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

        // You can deserialize with any label
        hidden_label!(TestLabel);
        let deser_label: Hidden<u8, TestLabel> = serde_json::from_str(&ser).unwrap();
        assert_eq!(hidden.reveal(), deser_label.reveal());
    }

    #[test]
    fn masking() {
        let hidden = Hidden::<u8>::hide(1u8);
        let formatted = format!("{}", hidden);
        let expected = format!("Hidden<{}, {}>", type_name::<u8>(), type_name::<DefaultHiddenLabel>());
        assert_eq!(formatted, expected);
    }
}
