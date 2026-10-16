// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/stm32f303xc/src/syscfg.rs -- doc comments, one reserved gap.
use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, register_structs};

register_structs! {
    /// System configuration controller
    SyscfgRegisters {
        /// memory remap register
        (0x000 => memrm: ReadWrite<u32, MEMRM::Register>),
        /// peripheral mode configuration register
        (0x004 => pmc: ReadWrite<u32, PMC::Register>),
        (0x008 => _reserved0),
        /// Compensation cell control register
        (0x010 => cmpcr: ReadOnly<u32, CMPCR::Register>),
        (0x014 => @END),
    }
}
