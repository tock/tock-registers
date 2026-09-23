// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Fake chip crate
//!
//! Contains fake versions of peripherals. Used by the safe chip crate's unit tests as well as test
//! infrastructure in external crates (e.g. integration tests).

use crate::chip_unsafe::rng;
use crate::safe_dma::{DmaEnable, FakeUnsafeWrite};
use core::{cell::Cell, ptr::null_mut, slice::from_raw_parts_mut};
use tock_registers::Register;

pub struct FakeRng {
    address: Cell<*mut u8>,
    len: Cell<usize>,
    state: Cell<u8>,
}

impl FakeRng {
    /// Gets the current state, for unit tests.
    pub fn get_state(&self) -> u8 {
        self.state.get()
    }
}

impl Default for FakeRng {
    fn default() -> Self {
        FakeRng {
            address: Cell::new(null_mut()),
            len: Cell::new(0),
            state: Cell::new(0),
        }
    }
}

impl rng::Interface for FakeRng {
    type address<'s> = FakeUnsafeWrite<'s, *mut u8>;
    fn address(&self) -> Self::address<'_> {
        FakeUnsafeWrite::new(&self.address)
    }
    type len<'s> = FakeUnsafeWrite<'s, usize>;
    fn len(&self) -> Self::len<'_> {
        FakeUnsafeWrite::new(&self.len)
    }
    type enable<'s> = FakeEnable<'s>;
    fn enable(&self) -> FakeEnable<'_> {
        FakeEnable(self)
    }
}

#[derive(Clone, Copy)]
pub struct FakeEnable<'f>(&'f FakeRng);

impl DmaEnable for FakeEnable<'_> {
    // This fake RNG always finishes its operation instantly, so the driver can never observe it
    // running.
    fn is_running(self) -> bool {
        false
    }

    unsafe fn start(self) {
        // Safety: The caller has guaranteed that `address` and `len` point to a fully-initialized
        // buffer that the hardware (`self`, in this case) can read and write. `u8`'s alignment is
        // 1.
        let buffer: &mut [u8] =
            unsafe { from_raw_parts_mut(self.0.address.get(), self.0.len.get()) };
        let mut state = self.0.state.get();
        // Seeding: sum the buffer contents as well as the state to get the new state.
        state = buffer
            .iter()
            .copied()
            .fold(0, u8::wrapping_add)
            .wrapping_add(state);
        // PRNG algorithm: increment for each output.
        for out in buffer {
            state = state.wrapping_add(1);
            *out = state;
        }
        self.0.state.set(state);
    }
}

impl Register for FakeEnable<'_> {
    type DataType = u8;
}
