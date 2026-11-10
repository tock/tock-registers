// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// chips/nrf5x/src/gpio.rs:85 -- nrf51 and nrf52 are layout-equivalent, but the
// "neither feature" build is real (it is what `make ci-job-chips` compiles)
// and has a different layout again.
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
pub struct GpioRegisters {
    out: ReadWrite<u32, Out::Register>,
    #[cfg(feature = "nrf51")]
    _reserved2: [u32; 120],
    #[cfg(feature = "nrf52")]
    latch: ReadWrite<u32, Latch::Register>,
    pin_cnf: [ReadWrite<u32, PinConfig::Register>; 32],
}
