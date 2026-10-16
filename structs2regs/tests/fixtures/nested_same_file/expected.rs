// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/sam4l/src/gpio.rs -- a block built from sub-blocks declared in
// the same module. Both convert; the outer one resolves the inner sizes.
use kernel::utilities::registers::{ReadOnly, ReadWrite, WriteOnly, register_structs};

register_structs! {
    Register {
        (0x000 => val: ReadWrite<u32>),
        (0x004 => set: WriteOnly<u32>),
        (0x008 => clear: WriteOnly<u32>),
        (0x00C => toggle: WriteOnly<u32>),
        (0x010 => @END),
    }
}

register_structs! {
    GpioRegisters {
        (0x000 => gper: Register),
        (0x010 => pmr0: Register),
        (0x020 => pvr: ReadOnly<u32>),
        (0x024 => _reserved0),
        (0x030 => @END),
    }
}
