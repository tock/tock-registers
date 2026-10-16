// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

// Shape: chips/sam4l/src/usart.rs -- padding declared with register types.
// It keeps its type: erasing it would drop the struct's alignment to 1.
use kernel::utilities::registers::{ReadOnly, ReadWrite};

#[repr(C)]
struct UsartRegisters {
    fidi: ReadWrite<u32, FidiRatio::Register>,
    _reserved0: [ReadOnly<u32>; 5],
    ner: ReadOnly<u32, NumErrors::Register>,
}
