// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/nrf5x-unsafe/src/aes.rs -- pub and private fields in one block.
use kernel::utilities::registers::{ReadWrite, WriteOnly, register_structs};

register_structs! {
    pub AesEcbRegisters {
        (0x000 => task_startecb: WriteOnly<u32, Task::Register>),
        (0x004 => _reserved1),
        (0x0FC => pub event_endecb: ReadWrite<u32, Event::Register>),
        (0x100 => pub intenset: ReadWrite<u32, Intenset::Register>),
        (0x104 => @END),
    }
}
