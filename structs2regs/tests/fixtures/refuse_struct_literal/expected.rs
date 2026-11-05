// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/lowrisc/src/usbdev.rs:322 -- an initializer names `_reserved`, which
// macro padding would retype to [u8; N].
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
struct Endpoint {
    buf: ReadWrite<u32>,
    _reserved: u32,
}

impl Endpoint {
    pub const fn new() -> Endpoint {
        Endpoint {
            buf: ReadWrite::new(0),
            _reserved: 0,
        }
    }
}
