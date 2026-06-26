// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use multi_crate_parent::array_definition;
use multi_crate_parent::array_reference;
use multi_crate_parent::block_definition;
use multi_crate_parent::block_reference;
use multi_crate_parent::scalar_definition;
use multi_crate_parent::scalar_reference;

tock_registers::mmio32_register_map! {
    scalar_definition_reference: scalar_definition,
    array_definition_reference: array_definition,
    block_definition_reference: block_definition,
    scalar_reference_reference: scalar_reference,
    array_reference_reference: array_reference,
    block_reference_reference: block_reference,
}
