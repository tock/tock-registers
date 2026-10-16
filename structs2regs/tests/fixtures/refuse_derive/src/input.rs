// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Derived code reads fields without naming them, and macro padding is
// [u8; N], which has no Default beyond 32 elements.
use kernel::utilities::registers::ReadWrite;

#[derive(Default)]
#[repr(C)]
struct StoredState {
    regs: ReadWrite<u32>,
    _reserved0: [u32; 2],
}
