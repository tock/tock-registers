// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/imxrt10xx/src/lpuart.rs:37 -- a field named with a raw identifier.
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::register_structs;

register_structs! {
    LpuartRegisters {
        (0x000 => baud: ReadWrite<u32, BAUD::Register>),
        (0x004 => r#match: ReadWrite<u32, MATCH::Register>),
        (0x008 => @END),
    }
}
