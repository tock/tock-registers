// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// -------------------------------------------------------------------------------------------------
// Safe chip crate
// -------------------------------------------------------------------------------------------------

//! Safe chip crate.
//!
//! Contains the implementation of the driver as well as unit tests for that driver.

#![forbid(unsafe_code)]

use crate::chip_unsafe::{rng, RngDma};

/// Driver for the RNG peripheral.
pub struct Rng<R: rng::Interface> {
    manager: RngDma<R>,
}

// A real driver would more complex than this (not just a wrapper around the DMA manager), because
// it would do things other than a pure DMA transfer.
impl<R: rng::Interface> Rng<R> {
    /// Constructs an instance of this driver with the given DMA manager.
    pub fn new(manager: RngDma<R>) -> Self {
        Rng { manager }
    }
}

impl<R: rng::Interface> Rng<R> {
    /// Starts filling the provided buffer with random data. The existing data in the buffer is
    /// used to seed the RNG as well.
    pub fn getrandom_start(&self, buffer: &'static mut [u8]) {
        self.manager.start(buffer)
    }

    /// Returns the filled buffer after the getrandom operation has finished.
    ///
    /// If no getrandom operation was started, or one is still ongoing, returns None.
    pub fn getrandom_finish(&self) -> Option<&'static mut [u8]> {
        self.manager.get_buffer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chip_fake::FakeRng, chip_unsafe::RngDma};
    use std::{rc::Rc, sync::Mutex};

    #[test]
    fn rng() {
        // This example safe DMA API only supports `&'static mut` buffers. For the unit test, we
        // intentionally leak a buffer to get that static lifetime without unsafe code. This hack
        // prevents Miri from flagging the leaked buffer.
        static BUFFER: Mutex<Option<&'static mut [u8]>> = Mutex::new(None);

        let (fake, manager) = RngDma::<Rc<FakeRng>>::new_fake();
        let driver = Rng::new(manager);
        assert_eq!(fake.get_state(), 0);
        let buffer = Box::leak(Box::new([1, 2, 3, 4]));
        driver.getrandom_start(buffer);
        assert_eq!(fake.get_state(), 14);
        let buffer = driver.getrandom_finish().unwrap();
        assert_eq!(fake.get_state(), 14);
        *BUFFER.lock().unwrap() = Some(buffer);
    }
}
