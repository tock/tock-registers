// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests a register block with each type of field.
use tock_registers::{mmio32_registers, Read, Write};

mmio32_registers! {
    // External registers to reference.
    a: u8 { Read, Write },
    b: u8 { Read, Write },
    c {
        0 => scalar_definition: u8 { Read, Write },
        1 => array_definition: [[u8; 2]; 3] { Read, Write },
        7 => _: 1,
        8 => scalar_reference: a,
        9 => array_reference: [[b; 2]; 3],
    }
}
