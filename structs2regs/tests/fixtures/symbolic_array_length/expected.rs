// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/nrf52/src/usbd.rs -- a file-local const folds; chips/sifive/src/plic.rs
// -- `div_ceil` does not, so that offset stays a symbolic expression.
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::register_structs;

const NUM_ENDPOINTS: usize = 8;
const MAX_INTERRUPTS: usize = 1023;
const MAX_BIT_REGS: usize = MAX_INTERRUPTS.div_ceil(32);

register_structs! {
    UsbdRegisters {
        (0x000 => epin: [ReadWrite<u32, Task::Register>; NUM_ENDPOINTS]),
        (0x020 => enable: [ReadWrite<u32, Enable::Register>; MAX_BIT_REGS]),
        (0x20 + 4 * MAX_BIT_REGS => claim: ReadWrite<u32, Claim::Register>),
        (0x24 + 4 * MAX_BIT_REGS => @END),
    }
}
