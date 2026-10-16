// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/stm32f303xc/src/syscfg.rs -- doc comments, one reserved gap.
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields};

/// System configuration controller
#[repr(C)]
struct SyscfgRegisters {
    /// memory remap register
    memrm: ReadWrite<u32, MEMRM::Register>,
    /// peripheral mode configuration register
    pmc: ReadWrite<u32, PMC::Register>,
    _reserved0: [u8; 8],
    /// Compensation cell control register
    cmpcr: ReadOnly<u32, CMPCR::Register>,
}
