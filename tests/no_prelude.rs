// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests the generated code in a crate using the `#![no_implicit_prelude]` attribute. All of the
//! generated code should use #tock_registers to access items from tock_registers and
//! #tock_registers::internal::core to access items from core.

#![no_implicit_prelude]
#![no_std]

use ::tock_registers::{mmio32_register_map, Read, UnsafeRead, UnsafeWrite, Write};

mmio32_register_map! {
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

mmio32_register_map! {
    pub inner {
        [0] => ctrl: u8 { Read, Write },
    },

    pub outer {
        [0] => ctrl: u8 { Read },
        [1] => inner: inner,
    },
}
