// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// usize is target-dependent: `make ci-job-chips` evaluates the generated
// assertions on a 64-bit host, but boards build for 32-bit targets.
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
struct Sized {
    ctrl: ReadWrite<u32, Ctrl::Register>,
    count: usize,
}
