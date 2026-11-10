// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/imxrt10xx/src/ccm_analog.rs:33 -- register_structs! accepts at most a
// lifetime parameter.
use kernel::utilities::registers::{ReadWrite, WriteOnly};

#[repr(C)]
struct RegisterSCT<R: RegisterLongName = ()> {
    reg: ReadWrite<u32, R>,
    set: WriteOnly<u32, R>,
}
