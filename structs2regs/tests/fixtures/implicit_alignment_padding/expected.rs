// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/imxrt10xx/src/dma.rs -- mixed u16/u32 widths, so the compiler
// inserts padding the macro has to be told about explicitly.
use kernel::utilities::registers::ReadWrite;
use kernel::utilities::registers::register_structs;

register_structs! {
    TransferControl {
        (0x000 => soff: ReadWrite<u16>),
        (0x002 => _implicit_padding0),
        (0x004 => nbytes: ReadWrite<u32>),
        (0x008 => doff: ReadWrite<u16>),
        (0x00A => @END),
    }
}
