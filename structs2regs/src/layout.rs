// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Placement of a field list according to the `#[repr(C)]` layout rules.

use crate::num::Num;
use crate::sizes::{Resolver, round_up};

/// One entry in the generated block: either a source field or a run of
/// compiler-inserted alignment padding that the macro needs spelled out.
pub struct Placed<'a> {
    pub field: Option<&'a syn::Field>,
    /// Set only for compiler-inserted padding, which has no name in the source.
    pub pad_name: Option<String>,
    pub offset: Num,
    pub align: u64,
}

impl<'a> Placed<'a> {
    /// The field's name in the source, or the invented padding name.
    pub fn name(&self) -> String {
        match (self.field, &self.pad_name) {
            (Some(f), _) => f.ident.as_ref().expect("named field").to_string(),
            (None, Some(n)) => n.clone(),
            (None, None) => unreachable!("a placed entry is a field or padding"),
        }
    }
}

pub struct Block<'a> {
    pub placed: Vec<Placed<'a>>,
    /// The `@END` offset: the running sum of all entries, *not* rounded up to
    /// the struct's alignment. `test_fields!` does that rounding itself when
    /// it compares against `size_of`.
    pub end: Num,
    pub align: u64,
}

/// Lay out `fields` as `#[repr(C)]` does: advance to each field's alignment,
/// place it, and accumulate.
///
/// Where the running offset is symbolic the tool cannot round it, so it
/// assumes no padding is needed. That assumption is not silent: `test_fields!`
/// asserts `SUM & (ALIGN - 1) == 0` for every field and checks each field's
/// end offset, so a wrong guess is a compile error rather than a bad layout.
pub fn place<'a>(
    fields: &[&'a syn::Field],
    resolver: &Resolver,
    stack: &mut Vec<String>,
) -> Result<Block<'a>, String> {
    let mut placed = Vec::new();
    let mut offset = Num::constant(0);
    let mut struct_align = 1u64;
    let mut pads = 0;

    for field in fields {
        let layout = resolver.resolve_in(&field.ty, stack).map_err(|e| {
            let name = field.ident.as_ref().expect("named field");
            format!("field `{name}`: {e}")
        })?;

        if let Some(at) = offset.as_const() {
            let aligned = round_up(at, layout.align);
            if aligned != at {
                placed.push(Placed {
                    field: None,
                    pad_name: Some(format!("_implicit_padding{pads}")),
                    offset: Num::constant(at),
                    align: 1,
                });
                pads += 1;
                offset = Num::constant(aligned);
            }
        }

        placed.push(Placed {
            field: Some(field),
            pad_name: None,
            offset: offset.clone(),
            align: layout.align,
        });
        offset = offset.add(&layout.size);
        struct_align = struct_align.max(layout.align);
    }

    Ok(Block {
        placed,
        end: offset,
        align: struct_align,
    })
}
