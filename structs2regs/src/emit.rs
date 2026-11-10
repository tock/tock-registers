// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Rendering a `register_structs!` invocation, and the temporary module that
//! proves it is layout-identical to the struct it replaces.
//!
//! rustfmt does not reformat macro bodies, and the repository sets
//! `error_on_unformatted = true`, so what this module emits is final.

use crate::layout::{Block, Placed};
use crate::sizes::Resolver;

const STRUCT_INDENT: &str = "    ";
const ENTRY_INDENT: &str = "        ";

/// Attributes that are dropped rather than forwarded, with the reason:
///
/// - `repr`: `register_structs!` emits its own `#[repr(C)]`.
/// - `cfg`: the whole invocation is already gated on that predicate.
fn forwarded(attr: &syn::Attribute) -> bool {
    !attr.path().is_ident("repr") && !attr.path().is_ident("cfg")
}

fn attr_lines(attrs: &[syn::Attribute], resolver: &Resolver, indent: &str, out: &mut String) {
    for attr in attrs.iter().filter(|a| forwarded(a)) {
        out.push_str(indent);
        out.push_str(resolver.verbatim(syn::spanned::Spanned::span(attr)));
        out.push('\n');
    }
}

/// Render one `register_structs! { .. }` invocation.
pub fn invocation(
    item: &syn::ItemStruct,
    block: &Block,
    collapse: &[bool],
    cfg: Option<&str>,
    resolver: &Resolver,
) -> String {
    let mut out = String::new();
    if let Some(predicate) = cfg {
        out.push_str(&format!("#[cfg({predicate})]\n"));
    }
    out.push_str("register_structs! {\n");

    attr_lines(&item.attrs, resolver, STRUCT_INDENT, &mut out);

    let visibility = resolver.verbatim(syn::spanned::Spanned::span(&item.vis));
    out.push_str(STRUCT_INDENT);
    if !visibility.is_empty() {
        out.push_str(visibility);
        out.push(' ');
    }
    out.push_str(&format!("{} {{\n", item.ident));

    for (entry, &collapsed) in block.placed.iter().zip(collapse) {
        if let Some(field) = entry.field {
            attr_lines(&field.attrs, resolver, ENTRY_INDENT, &mut out);
        }
        out.push_str(ENTRY_INDENT);
        out.push_str(&entry_text(entry, collapsed, resolver));
        out.push('\n');
    }

    out.push_str(&format!(
        "{ENTRY_INDENT}({} => @END),\n",
        block.end.render_offset()
    ));
    out.push_str(STRUCT_INDENT);
    out.push_str("}\n}\n");
    out
}

fn entry_text(entry: &Placed, collapsed: bool, resolver: &Resolver) -> String {
    let offset = entry.offset.render_offset();
    let name = entry.name();
    if collapsed {
        return format!("({offset} => {name}),");
    }
    let field = entry.field.expect("only a source field is left typed");
    let visibility = resolver.verbatim(syn::spanned::Spanned::span(&field.vis));
    let ty = resolver.verbatim(syn::spanned::Spanned::span(&field.ty));
    if visibility.is_empty() {
        format!("({offset} => {name}: {ty}),")
    } else {
        format!("({offset} => {visibility} {name}: {ty}),")
    }
}

/// Render the temporary module that proves the conversion preserved the
/// layout.
///
/// `test_fields!` never calls `offset_of!`, so on its own it cannot see a
/// dropped field, two same-size fields swapped, a type replaced by one of the
/// same size, or an alignment change. This module closes all four by holding
/// the original struct alongside the converted one and comparing them.
pub fn layout_check(
    item: &syn::ItemStruct,
    block: &Block,
    cfg: Option<&str>,
    resolver: &Resolver,
) -> String {
    let name = &item.ident;
    let mut out = String::new();
    if let Some(predicate) = cfg {
        out.push_str(&format!("#[cfg({predicate})]\n"));
    }
    out.push_str(&format!(
        "// structs2regs: temporary layout check, removed automatically once `cargo check` passes.\n\
         #[allow(dead_code, non_snake_case)]\n\
         mod {CHECK_PREFIX}{name} {{\n    \
             use super::*;\n    \
             #[repr(C)]\n    \
             struct Original {{\n"
    ));

    for entry in &block.placed {
        // Compiler-inserted padding is not written in the source and so has
        // no counterpart in `Original`.
        let Some(field) = entry.field else { continue };
        let ident = entry.name();
        let ty = resolver.verbatim(syn::spanned::Spanned::span(&field.ty));
        out.push_str(&format!("        {ident}: {ty},\n"));
    }
    out.push_str("    }\n    const _: () = {\n");
    out.push_str(&format!(
        "        assert!(core::mem::size_of::<Original>() == core::mem::size_of::<super::{name}>());\n\
         \x20       assert!(core::mem::align_of::<Original>() == core::mem::align_of::<super::{name}>());\n"
    ));
    for entry in &block.placed {
        if entry.field.is_none() {
            continue;
        }
        let ident = entry.name();
        out.push_str(&format!(
            "        assert!(core::mem::offset_of!(Original, {ident}) == core::mem::offset_of!(super::{name}, {ident}));\n"
        ));
    }
    out.push_str("    };\n}\n");
    out
}

pub const CHECK_PREFIX: &str = "__structs2regs_check_";
