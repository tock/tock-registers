// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/sam4l/src/crccu.rs:154 -- a second repr attribute. The macro emits its
// own #[repr(C)] and an added alignment interacts with padding type erasure.
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
#[repr(align(512))]
struct Descriptor {
    addr: ReadWrite<u32>,
    ctrl: ReadWrite<u32>,
}
