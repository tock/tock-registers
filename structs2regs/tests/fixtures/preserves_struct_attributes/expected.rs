// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/sam4l/src/dma.rs -- #[allow(dead_code)] must survive: CI builds chip
// crates with -D warnings.
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::register_structs;

register_structs! {
    /// DMA channel registers.
    #[allow(dead_code)]
    DmaRegisters {
        (0x000 => memaddr: ReadWrite<u32, MemoryAddress::Register>),
        (0x004 => _reserved0),
        (0x008 => ctrl: ReadWrite<u32, Control::Register>),
        (0x00C => @END),
    }
}
