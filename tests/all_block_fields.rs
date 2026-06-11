// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests a register block with each type of field.

#![no_std]

use tock_registers::{mmio32_register_map, Read, UnsafeRead, UnsafeWrite, Write};

mmio32_register_map! {
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
