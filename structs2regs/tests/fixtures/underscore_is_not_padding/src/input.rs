// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/sam4l/src/pm.rs:56 -- `_protctrl` is underscore-prefixed to silence
// dead_code, but it is a real register and must keep its type.
// chips/sam4l/src/gpio.rs:25 -- `reserved0` has no underscore but is padding.
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
struct PmRegisters {
    ctrl: ReadWrite<u32, Control::Register>,
    _protctrl: u32,
    reserved0: u32,
    version: u32,
    unlock: ReadWrite<u32, Unlock::Register>,
}
