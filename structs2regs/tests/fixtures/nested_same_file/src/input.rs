// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/sam4l/src/gpio.rs -- a block built from sub-blocks declared in
// the same module. Both convert; the outer one resolves the inner sizes.
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly};

#[repr(C)]
struct Register {
    val: ReadWrite<u32>,
    set: WriteOnly<u32>,
    clear: WriteOnly<u32>,
    toggle: WriteOnly<u32>,
}

#[repr(C)]
struct GpioRegisters {
    gper: Register,
    pmr0: Register,
    pvr: ReadOnly<u32>,
    _reserved0: [u32; 3],
}
