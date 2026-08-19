// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

#![cfg(test)]

use core::ptr::{null_mut, NonNull};

#[cfg(not(target_pointer_width = "32"))]
use crate::{Mmio32, Mmio32Nullable};

#[cfg(not(target_pointer_width = "64"))]
use crate::{Mmio64, Mmio64Nullable};

#[cfg(not(target_pointer_width = "32"))]
#[test]
#[should_panic = "Mmio32 requires a 32-bit system"]
fn mmio32_from_addr_width_check() {
    Mmio32::from_addr(1);
}

#[cfg(not(target_pointer_width = "32"))]
#[test]
#[should_panic = "Mmio32 requires a 32-bit system"]
fn mmio32_new_width_check() {
    Mmio32::new(NonNull::dangling());
}

#[cfg(not(target_pointer_width = "32"))]
#[test]
#[should_panic = "Mmio32Nullable requires a 32-bit system"]
fn mmio32nullable_from_addr_width_check() {
    Mmio32Nullable::from_addr(1);
}

#[cfg(not(target_pointer_width = "32"))]
#[test]
#[should_panic = "Mmio32Nullable requires a 32-bit system"]
fn mmio32nullable_new_width_check() {
    Mmio32Nullable::new(null_mut());
}

#[cfg(not(target_pointer_width = "64"))]
#[test]
#[should_panic = "Mmio64 requires a 64-bit system"]
fn mmio64_from_addr_width_check() {
    Mmio64::from_addr(1);
}

#[cfg(not(target_pointer_width = "64"))]
#[test]
#[should_panic = "Mmio64 requires a 64-bit system"]
fn mmio64_new_width_check() {
    Mmio64::new(NonNull::dangling());
}

#[cfg(not(target_pointer_width = "64"))]
#[test]
#[should_panic = "Mmio64Nullable requires a 64-bit system"]
fn mmio64nullable_from_addr_width_check() {
    Mmio64Nullable::from_addr(1);
}

#[cfg(not(target_pointer_width = "64"))]
#[test]
#[should_panic = "Mmio64Nullable requires a 64-bit system"]
fn mmio64nullable_new_width_check() {
    Mmio64Nullable::new(null_mut());
}
