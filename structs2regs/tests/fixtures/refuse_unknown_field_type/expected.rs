// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/litex/src/uart.rs -- an associated type has no statically known size.
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
pub struct Mixed {
    known: ReadWrite<u32, Ctrl::Register>,
    unknown: SomeTypeFromAnotherModule,
}
