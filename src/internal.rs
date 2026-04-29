// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.
// Copyright Better Bytes 2026.

//! Items used by macros that are not part of tock-registers' public interface. Not for use by
//! outside crates (the contents of this module are not stable).

#![doc(hidden)]

use core::marker::PhantomData;

/// It's possible for a crate that is not libcore to be named `core` in a calling crate. Re-export
/// so registers! can reliably find libcore.
pub use core;
#[cfg(feature = "proc_macros")]
pub use tock_registers_macros::registers;

/// Phantom type to make Real structs !Send and !Sync.
#[derive(Clone, Copy, Default)]
pub struct RealPhantom(PhantomData<*mut ()>);

impl RealPhantom {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}
