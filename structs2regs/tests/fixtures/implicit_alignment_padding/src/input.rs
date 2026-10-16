// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/imxrt10xx/src/dma.rs -- mixed u16/u32 widths, so the compiler
// inserts padding the macro has to be told about explicitly.
use kernel::utilities::registers::ReadWrite;

#[repr(C)]
struct TransferControl {
    soff: ReadWrite<u16>,
    nbytes: ReadWrite<u32>,
    doff: ReadWrite<u16>,
}
