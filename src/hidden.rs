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

//! A generic type that hides data from being displayed or written, keeps it on the heap, zeroizes it when it goes away,
//! limits access, and allows for type enforcement.

use std::{any::type_name, fmt, marker::PhantomData};

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// A marker trait for labeling different hidden types
pub trait HiddenLabel {}

/// Create a hidden type label
#[macro_export]
macro_rules! hidden_label {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
        pub struct $name;

        impl HiddenLabel for $name {}
    };
}

// A default hidden type label; only use this if you're absolutely sure you don't care about type enforcement
hidden_label!(DefaultHiddenLabel);

/// Data that needs to be kept hidden, needs to be zeroized when it goes away, and is resistant to misuse
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
    /// Hide the data
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

/// Only output masked data for debugging
impl<T, L> fmt::Debug for Hidden<T, L>
where
    T: Zeroize,
    L: HiddenLabel,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hidden<{}, {}>", type_name::<T>(), type_name::<L>(),)
    }
}

/// Only display masked data
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
