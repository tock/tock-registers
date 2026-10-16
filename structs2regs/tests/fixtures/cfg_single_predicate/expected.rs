// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/stm32f4xx/src/flash.rs -- one cfg predicate. test_fields! discards cfg
// attributes, so each configuration needs its own invocation.
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_structs};

#[cfg(feature = "stm32f429")]
register_structs! {
    FlashRegisters {
        (0x000 => acr: ReadWrite<u32, ACR::Register>),
        (0x004 => keyr: WriteOnly<u32>),
        (0x008 => optcr1: ReadWrite<u32, OPTCR1::Register>),
        (0x00C => @END),
    }
}

#[cfg(not(feature = "stm32f429"))]
register_structs! {
    FlashRegisters {
        (0x000 => acr: ReadWrite<u32, ACR::Register>),
        (0x004 => keyr: WriteOnly<u32>),
        (0x008 => @END),
    }
}
