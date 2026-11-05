// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/nrf5x-unsafe/src/aes.rs -- pub and private fields in one block.
use kernel::utilities::registers::{ReadWrite, WriteOnly};

#[repr(C)]
pub struct AesEcbRegisters {
    task_startecb: WriteOnly<u32, Task::Register>,
    _reserved1: [u32; 62],
    pub event_endecb: ReadWrite<u32, Event::Register>,
    pub intenset: ReadWrite<u32, Intenset::Register>,
}
