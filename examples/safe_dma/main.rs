// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

#![forbid(unsafe_op_in_unsafe_fn)]

pub mod chip_fake;
pub mod chip_safe;
pub mod chip_unsafe;
pub mod safe_dma;

// Publicly re-export the operations at the crate root, as required by tock-registers.
pub use safe_dma::{DmaEnable, UnsafeWrite};

// -------------------------------------------------------------------------------------------------
// main(): Represents a board crate (instantiating the Real instance of the driver, albeit with a
// on-stack fake peripheral).
// -------------------------------------------------------------------------------------------------

#[cfg(target_pointer_width = "64")]
fn main() {
    use chip_safe::Rng;
    use chip_unsafe::RngDma;
    use core::cell::UnsafeCell;
    use core::mem::{offset_of, replace};
    use core::ptr::{null_mut, NonNull};
    use tock_registers::Mmio64;

    // Replica of the of the peripheral registers' MMIO interface; allows us to demonstrate how
    // chip_unsafe::RngDma will be used for real (in a board crate).
    #[repr(C)]
    struct SimulatedMmio {
        address: *mut u8,
        len: usize,
        enable: u8,
    }
    assert_eq!(offset_of!(SimulatedMmio, address), 0);
    assert_eq!(offset_of!(SimulatedMmio, len), 8);
    assert_eq!(offset_of!(SimulatedMmio, enable), 16);

    let fake = UnsafeCell::new(SimulatedMmio {
        address: null_mut(),
        len: 0,
        enable: 0,
    });
    let mmio = Mmio64::new(NonNull::new(fake.get()).unwrap().cast());
    // Safety: `Fake` correctly matches the register map, and this handle (and everything derived
    // from it) is dropped by this function before `fake` is dropped.
    let registers = unsafe { chip_unsafe::rng::Real::new(mmio) };
    // Safety: Nothing other than this DMA manager mutates `fake` (this is a safety invariant the
    // board crate can assert but no other crate can). Slight exception: we do set `enable` to 0 to
    // simulate the DMA operation completing, but we don't simulate any DMA operations after that.
    let manager = unsafe { RngDma::new(registers) };
    let driver = Rng::new(manager);

    let buffer = Box::leak(Box::new([0; 4]));
    driver.getrandom_start(buffer);

    // Verify the peripheral was configured correctly, then simulate DMA operations.
    let fake: &mut SimulatedMmio = unsafe { &mut *fake.get() };
    assert_eq!(fake.enable, 1);
    assert_eq!(fake.len, 4);
    let buffer: &mut [u8; 4] = unsafe { &mut *fake.address.cast() };
    assert_eq!(replace(buffer, [1, 2, 3, 4]), [0; 4]);
    // Simulate the DMA operation ending.
    // Safety: No other thread is accessing fake.
    fake.enable = 0;

    let buffer = driver.getrandom_finish().unwrap();
    assert_eq!(buffer, [1, 2, 3, 4]);

    drop(unsafe { Box::from_raw(buffer) });
}

#[cfg(not(target_pointer_width = "64"))]
fn main() {}
