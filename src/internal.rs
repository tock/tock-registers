// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2024.
// Copyright Google LLC 2024.
// Copyright Better Bytes 2026.

//! Items used by macros that are not part of tock-registers' public interface. Not for use by
//! outside crates (the contents of this module are not stable).

#![doc(hidden)]

/// It's possible for a crate that is not libcore to be named `core` in a calling crate. Re-export
/// so registers! can reliably find libcore.
pub use core;
pub use tock_registers_macros::registers;

/// Information about a register block.
///
/// BlockInfo is embedded inside the generated Bus implementation for a register block. NUM_FIELDS
/// includes padding fields as well as fields that are #[cfg]-disabled. Note that a #[cfg]-disabled
/// field may have an incorrect offset.
pub struct BlockInfo<const NUM_FIELDS: usize> {
    pub offsets: [usize; NUM_FIELDS],
    pub block_size: usize,
}

impl<const N: usize> BlockInfo<N> {
    /// Computes the block info for a register.
    pub const fn new(
        cfgs: [bool; N],
        element_sizes: [usize; N],
        array_lens: [&[usize]; N],
    ) -> Self {
        let mut i = 0;
        let mut offsets = [0; N];
        let mut block_size = 0;
        while i < N {
            offsets[i] = block_size;
            if cfgs[i] {
                block_size += size_calc(element_sizes[i], array_lens[i]);
            }
            i += 1;
        }
        BlockInfo {
            offsets,
            block_size,
        }
    }
}

/// Returns the block size of a field with the given element size and array lengths.
const fn size_calc(mut element_size: usize, array_lens: &[usize]) -> usize {
    let mut i = 0;
    while i < array_lens.len() {
        element_size *= array_lens[i];
        i += 1;
    }
    element_size
}
