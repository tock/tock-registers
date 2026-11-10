// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/sam4l/src/dma.rs -- #[allow(dead_code)] must survive: CI builds chip
// crates with -D warnings.
use kernel::utilities::registers::ReadWrite;

/// DMA channel registers.
#[allow(dead_code)]
#[repr(C)]
struct DmaRegisters {
    memaddr: ReadWrite<u32, MemoryAddress::Register>,
    _reserved0: [u8; 4],
    ctrl: ReadWrite<u32, Control::Register>,
}
