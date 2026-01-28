// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{Address, Bus, BusRead, BusWrite};
use core::ptr::{read_volatile, write_volatile};

/// MMIO register bus for 32-bit systems.
#[derive(Clone, Copy)]
pub struct Mmio32(pub *mut ());

impl Address for Mmio32 {
    unsafe fn byte_add(self, offset: usize) -> Mmio32 {
        // Safety: The safety requirements of Address::byte_add require self + offset to remain
        // within this register span and not wrap.
        Mmio32(unsafe { self.0.byte_add(offset) })
    }
}

// TODO: Implement a arm64_secure_vm feature that uses confidential-vm-safe instructions:
// https://github.com/google/safe-mmio/blob/main/src/aarch64_mmio.rs
// https://github.com/rust-lang/rust/issues/131894
macro_rules! bus_impls {
    ($name:ident, $data_type:ty, $size:literal) => {
        impl Bus<$data_type> for $name {
            const PADDED_SIZE: usize = $size;
        }
        impl BusRead<$data_type> for $name {
            unsafe fn read(self) -> $data_type {
                unsafe { read_volatile(self.0.cast_const().cast()) }
            }
        }
        impl BusWrite<$data_type> for $name {
            unsafe fn write(self, value: $data_type) {
                unsafe { write_volatile(self.0.cast(), value) }
            }
        }
    };
}

// TODO: Should this have a u64 impl or not?
bus_impls!(Mmio32, u8, 1);
bus_impls!(Mmio32, u16, 2);
bus_impls!(Mmio32, u32, 4);
bus_impls!(Mmio32, u64, 8);
bus_impls!(Mmio32, usize, 4);

/// MMIO register bus for 64-bit systems.
#[derive(Clone, Copy)]
pub struct Mmio64(pub *mut ());

impl Address for Mmio64 {
    unsafe fn byte_add(self, offset: usize) -> Mmio64 {
        // Safety: The safety requirements of Address::byte_add require self + offset to remain
        // within this register span and not wrap.
        Mmio64(unsafe { self.0.byte_add(offset) })
    }
}

bus_impls!(Mmio64, u8, 1);
bus_impls!(Mmio64, u16, 2);
bus_impls!(Mmio64, u32, 4);
bus_impls!(Mmio64, u64, 8);
bus_impls!(Mmio64, usize, 8);
