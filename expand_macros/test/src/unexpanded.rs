// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// Rust inhibits many `unused` warnings on code emitted by proc macros. However, this test bypasses
// the proc macro mechanism, so we have to do that allow ourselves.
#![allow(unused)]

use tock_registers::{Mmio32, Mmio64, Read, Write};

// We intentionally test a couple different macro paths: different macro names, and with or without
// a leading tock_registers.

register_layouts! {
    #![buses(Mmio32, Mmio64)]
    a: u8 { Read, Write },
    b: u8 { Read, Write },
}

::tock_registers::mmio32_register_layouts! {
    c {
        0 => scalar_definition: u8 { Read, Write },
        1 => array_definition: [[u8; 2]; 3] { Read, Write },
        7 => _: 1,
        8 => scalar_reference: a,
        9 => array_reference: [[b; 2]; 3],
    }
}

// Uncomment this to verify that tock_registers is being used without the proc_macros feature (this
// should fail to build with an "unresolved import" error).
//use tock_registers::register_layouts;
