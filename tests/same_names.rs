// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests register_layouts! with nested blocks that have the same variable name (this can trigger
//! "multiple items in scope" errors if the generated code isn't careful about which traits are in
//! scope).

#![no_std]

use tock_registers::{mmio32_register_layouts, Read, Write};

mmio32_register_layouts! {
    pub inner {
        // We use an offset array to make the #name_offset entries exist in both the Bus
        // declaration and impls, as the name collision error tends to show up in the impls.
        [0] => ctrl: u8 { Read, Write },
    },

    pub outer {
        [0] => ctrl: u8 { Read },
        [1] => inner: inner,
    },
}
