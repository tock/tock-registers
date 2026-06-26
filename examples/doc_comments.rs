// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! This example shows where you can use doc comments in a register_map! invocation.

use tock_registers::{mmio32_register_map, Read, Write};

mmio32_register_map! {
    /// Doc comment on a register definition.
    pub scalar_definition: u8 { Read },
}

mmio32_register_map! {
    /// Another doc comment on a register definition.
    pub array_definition: [u8; 3] { Read },
}

mmio32_register_map! {
    /// Doc comment on a register reference.
    pub scalar_reference: scalar_definition,
}

mmio32_register_map! {
    /// Another doc comment on a register reference.
    pub array_reference: [scalar_reference; 2],
}

mmio32_register_map! {
    /// Doc comment on a register block.
    pub foo {
        // Note: you cannot have an inner doc comment here.

        /// You can have doc comments on each field of a register block.
        0 => status: u8 { Read },

        // Note: You cannot have doc comments on a padding field.
        1 => _: 1,

        /// Another register block field doc comment.
        2 => control: u8 { Read, Write },
    }
}

mmio32_register_map! {
    //! Inner doc comment on the register_map! invocation itself. This doc comment will be copied
    //! onto both `buttons` and `pins`.

    /// Doc comment that only applies to buttons.
    pub buttons: u8 { Read },

    /// Another doc that only applies to pins.
    pub pins: [u8; 3] { Read },
}

fn main() {}
