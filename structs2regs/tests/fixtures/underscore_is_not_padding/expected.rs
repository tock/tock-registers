// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/sam4l/src/pm.rs:56 -- `_protctrl` is underscore-prefixed to silence
// dead_code, but it is a real register and must keep its type.
// chips/sam4l/src/gpio.rs:25 -- `reserved0` has no underscore but is padding.
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::register_structs;

register_structs! {
    PmRegisters {
        (0x000 => ctrl: ReadWrite<u32, Control::Register>),
        (0x004 => _protctrl: u32),
        (0x008 => reserved0),
        (0x00C => version: u32),
        (0x010 => unlock: ReadWrite<u32, Unlock::Register>),
        (0x014 => @END),
    }
}
