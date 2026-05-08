// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests a register block with each type of field.

#![no_std]

// Note that we don't use-import the macro because this file is also used for expand_macros_test,
// which builds the expanded code without the proc_macros feature.
use tock_registers::{Read, UnsafeRead, UnsafeWrite, Write};

tock_registers::mmio32_register_layouts! {
    // External registers to reference.
    a: u8 { Read, Write },
    b: u8 { UnsafeRead, Write },
    c {
        0 => scalar_definition: u8 { Read, UnsafeWrite },
        1 => array_definition: [[u8; 2]; 3] { UnsafeRead, UnsafeWrite },
        7 => _: 1,
        8 => scalar_reference: a,
        9 => array_reference: [[b; 2]; 3],
    }
}
