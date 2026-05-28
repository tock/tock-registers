// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#![no_std]

use tock_registers::{Mmio32, Mmio64, Read, Write};

// We intentionally test a couple different macro paths: different macro names, and with or without
// a leading tock_registers.

register_map! {
    #![buses(Mmio32, Mmio64)]
    a: u8 { Read, Write },
    b: u8 { Read, Write },
}

::tock_registers::mmio32_register_map! {
    c {
        0 => scalar_definition: u8 { Read, Write },
        1 => array_definition: [[u8; 2]; 3] { Read, Write },
        7 => _: 1,
        8 => scalar_reference: a,
        9 => array_reference: [[b; 2]; 3],
    }
}

tock_registers::mmio64_register_map! {
    d: a,
}

// Uncomment this to verify that tock_registers is being used without the proc_macros feature (this
// should fail to build with an "unresolved import" error).
//use tock_registers::register_map;
