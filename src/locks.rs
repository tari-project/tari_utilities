// Copyright 2019, The Tari Project
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

//! Macros for RwLock.

/// Recovers a poisoned lock by returning the guard anyway.
///
/// The protected value is left as it was when the panic occurred, which may be partially updated. It is not rolled
/// back.
#[macro_export]
macro_rules! recover_lock {
    ($e:expr) => {
        match $e {
            Ok(lock) => lock,
            Err(poisoned) => {
                $crate::log::warn!(target: "tari_util", "Lock has been POISONED and will be silently recovered");
                poisoned.into_inner()
            },
        }
    };
}

/// This macro returns a Mutex or RwLock guard without returning a `PoisonError`.
/// If the lock is poisoned (i.e. a panic before a MutexGuard / RwLockGuard is dropped), a warning is logged and the
/// guard is returned anyway. The value is whatever state the panicking thread left it in, which may be partially
/// updated; this is NOT a rollback, so callers must not rely on the value being consistent after a poisoning panic.
#[macro_export]
macro_rules! acquire_lock {
    ($e:expr, $m:ident) => {
        $crate::recover_lock!($e.$m())
    };
    ($e:expr) => {
        $crate::acquire_lock!($e, lock)
    };
}

/// Acquire a write lock on a RwLock, silently recovering the lock if it is poisoned
#[macro_export]
macro_rules! acquire_write_lock {
    ($e:expr) => {
        $crate::acquire_lock!($e, write)
    };
}

/// Acquire a read lock on a RwLock, silently recovering the lock if it is poisoned
#[macro_export]
macro_rules! acquire_read_lock {
    ($e:expr) => {
        $crate::acquire_lock!($e, read)
    };
}

#[cfg(all(test, feature = "std"))]
mod test {
    use std::{
        sync::{Mutex, RwLock},
        thread,
    };

    #[test]
    fn poisoned_lock_keeps_partial_update() {
        let lock = Mutex::new(vec![1, 2]);
        thread::scope(|s| {
            let result = s
                .spawn(|| {
                    let mut guard = lock.lock().unwrap();
                    guard.push(3);
                    panic!("poison the lock");
                })
                .join();
            assert!(result.is_err());
        });
        assert!(lock.is_poisoned());
        // The partial update survives; nothing is rolled back
        assert_eq!(*acquire_lock!(lock), vec![1, 2, 3]);

        let rw = RwLock::new(1);
        assert_eq!(*acquire_read_lock!(rw), 1);
        *acquire_write_lock!(rw) = 2;
        assert_eq!(*acquire_read_lock!(rw), 2);
    }
}
