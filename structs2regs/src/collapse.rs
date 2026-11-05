// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Deciding which fields give up their type to become implicit padding.
//!
//! Collapsing never renames and never merges: `(0x014 => _protctrl)` still
//! generates a field called `_protctrl`, just typed `[u8; 4]`. Every name in
//! the source therefore survives into the converted struct at the same
//! offset, which is what lets the generated equivalence module assert an
//! offset for *every* field, collapsed ones included.

use crate::crate_scan::CrateScan;
use crate::layout::Placed;
use crate::sizes::is_plain_integer_storage;

/// Name stems that mark a gap in the memory map rather than a register.
///
/// A name test is needed on top of the type test because some real registers
/// are declared as bare integers: `parameter: u32` and `version: u32` in
/// `chips/sam4l/src/gpio.rs` are registers, not padding. Erring here only
/// leaves a field typed, which is always valid output.
const RESERVED_STEMS: &[&str] = &[
    "reserved", "reserve", "res", "rsvd", "padding", "pad", "unused", "spare",
];

fn looks_reserved(name: &str) -> bool {
    let stem = name.strip_prefix('_').unwrap_or(name).to_ascii_lowercase();
    RESERVED_STEMS.iter().any(|s| {
        stem.strip_prefix(s).is_some_and(|rest| {
            rest.is_empty() || rest.starts_with(|c: char| c.is_ascii_digit() || c == '_')
        })
    })
}

/// Returns, for each placed entry, whether to emit it as a bare padding
/// marker rather than a typed field.
pub fn decide(placed: &[Placed], scan: &CrateScan, struct_align: u64) -> Vec<bool> {
    let mut collapse: Vec<bool> = placed
        .iter()
        .map(|entry| match entry.field {
            // Compiler-inserted padding has no name and no type in the
            // source; a marker is the only faithful rendering.
            None => true,
            Some(field) => {
                let name = entry.name();
                // Three independent signals must agree before a field gives
                // up its type. Measured over `chips/`, the type test alone
                // rescues 121 real registers that the other tests miss
                // (`keyr: WriteOnly<u32>`, `events_ch: [AdcEventChRegisters; 8]`),
                // and the name test rescues a further handful declared as
                // bare integers.
                matches!(field.vis, syn::Visibility::Inherited)
                    && is_plain_integer_storage(&field.ty)
                    && looks_reserved(&name)
                    && scan.is_unused(&name)
            }
        })
        .collect();

    // Macro padding is `[u8; N]`, alignment 1. If the only field carrying the
    // struct's alignment is one being collapsed, the converted struct would
    // be less aligned than the original. Keep such a field typed.
    //
    // `test_fields!` computes its MAX_ALIGN from typed entries only, so this
    // mirrors exactly what the generated assertions will see.
    loop {
        let kept = placed
            .iter()
            .zip(&collapse)
            .filter(|(_, collapsed)| !**collapsed)
            .map(|(entry, _)| entry.align)
            .max()
            .unwrap_or(1);
        if kept >= struct_align {
            break;
        }
        let restore = placed.iter().zip(&collapse).position(|(entry, collapsed)| {
            *collapsed && entry.field.is_some() && entry.align == struct_align
        });
        match restore {
            Some(index) => collapse[index] = false,
            None => break,
        }
    }

    collapse
}

#[cfg(test)]
mod tests {
    use super::looks_reserved;

    #[test]
    fn recognizes_the_conventional_gap_names() {
        for name in [
            "_reserved0",
            "reserved0",
            "_reserved",
            "_res",
            "_res2",
            "_pad",
            "_padding",
            "RESERVED1",
            "_unused",
            "_spare",
        ] {
            assert!(looks_reserved(name), "{name} should read as padding");
        }
    }

    #[test]
    fn leaves_registers_alone() {
        // Real registers from chips/: an undocumented one, two declared as
        // bare integers, and two whose names merely begin with the same
        // letters as a stem.
        for name in [
            "_protctrl",
            "parameter",
            "version",
            "keyr",
            "resume",
            "result",
            "padr",
        ] {
            assert!(!looks_reserved(name), "{name} should read as a register");
        }
    }
}
