// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use tock_registers::{mmio32_registers, Read};

mmio32_registers! {
    pub scalar_definition: u8 { Read },
    pub array_definition: [u8; 2] { Read },
    pub block_definition {
        0 => scalar_field: u8 { Read },
    }
}

mmio32_registers! {
    pub scalar_reference: scalar_definition,
    pub array_reference: array_definition,
    pub block_reference: block_definition,
}
