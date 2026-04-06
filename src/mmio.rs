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

// Safety: Mmio32 does not expose safe operations to access registers on its own. Instead, it is
// used through two types:
// 1. The Real<> structs, which are always !Send + !Sync
// 2. RegisterSender<>, which is Send if the bus is Send but which guarantees that only one thread
//    can access the registers at a time. Making Mmio32 Send enables RegisterSender<> to work with
//    it.
unsafe impl Send for Mmio32 {}

// TODO: Implement a arm64_secure_vm feature that uses confidential-vm-safe instructions:
// https://github.com/google/safe-mmio/blob/main/src/aarch64_mmio.rs
// https://github.com/rust-lang/rust/issues/131894
macro_rules! bus_impls {
    ($name:ident, $value:ty, $size:literal) => {
        /// Safety: All the bus_impls! invocations have the correct size.
        unsafe impl Bus<$value> for $name {
            const PADDED_SIZE: usize = $size;
        }
        impl BusRead<$value> for $name {
            unsafe fn read(self) -> $value {
                // BusRead::read's preconditions guarantee that a readable register with value type
                // $value exists at address self.0, and the caller is responsible for avoiding data
                // races and satisfying any other unsafe invariants of the register.
                unsafe { read_volatile(self.0.cast_const().cast()) }
            }
        }
        impl BusWrite<$value> for $name {
            unsafe fn write(self, value: $value) {
                // BusRead::write's preconditions guarantee that a writable register with value
                // type $value exists at address self.0, and the caller is responsible for avoiding
                // data races and satisfying any other unsafe invariants of the register.
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

// Safety: Mmio64 does not expose safe operations to access registers on its own. Instead, it is
// used through two types:
// 1. The Real<> structs, which are always !Send + !Sync
// 2. RegisterSender<>, which is Send if the bus is Send but which guarantees that only one thread
//    can access the registers at a time. Making Mmio64 Send enables RegisterSender<> to work with
//    it.
unsafe impl Send for Mmio64 {}

bus_impls!(Mmio64, u8, 1);
bus_impls!(Mmio64, u16, 2);
bus_impls!(Mmio64, u32, 4);
bus_impls!(Mmio64, u64, 8);
bus_impls!(Mmio64, usize, 8);
