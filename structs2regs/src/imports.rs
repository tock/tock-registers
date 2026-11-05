// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Bringing `register_structs` into scope.
//!
//! Chip modules reach the macro through the kernel re-export
//! (`kernel/src/utilities/mod.rs`), so a converted module needs
//! `register_structs` in its `kernel::utilities::registers` import.

use std::ops::Range;

const MACRO: &str = "register_structs";

/// The edit that puts `register_structs` in scope, or `None` if it already is.
pub fn ensure_import(source: &str, file: &syn::File) -> Option<(Range<usize>, String)> {
    let uses: Vec<&syn::ItemUse> = file
        .items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Use(u) => Some(u),
            _ => None,
        })
        .collect();

    if uses.iter().any(|u| imports_macro(&u.tree)) {
        return None;
    }

    // Preferred: widen the existing registers import.
    for item in &uses {
        if !is_registers_group(&item.tree) {
            continue;
        }
        let span = syn::spanned::Spanned::span(item).byte_range();
        if let Some(replacement) = widen_group(&source[span.clone()]) {
            return Some((span, replacement));
        }
    }

    // Otherwise add a fresh import after the last `use`.
    let anchor = uses
        .last()
        .map(|u| syn::spanned::Spanned::span(u).byte_range().end)?;
    Some((
        anchor..anchor,
        format!("\nuse kernel::utilities::registers::{MACRO};"),
    ))
}

/// Whether this is `use kernel::utilities::registers::{ .. };` exactly.
///
/// Matching on the tree rather than on a text prefix matters: a sibling
/// import such as `kernel::utilities::registers::interfaces::{..}` shares the
/// prefix but must not absorb the macro.
fn is_registers_group(tree: &syn::UseTree) -> bool {
    let syn::UseTree::Path(kernel) = tree else {
        return false;
    };
    let syn::UseTree::Path(utilities) = &*kernel.tree else {
        return false;
    };
    let syn::UseTree::Path(registers) = &*utilities.tree else {
        return false;
    };
    kernel.ident == "kernel"
        && utilities.ident == "utilities"
        && registers.ident == "registers"
        && matches!(&*registers.tree, syn::UseTree::Group(_))
}

fn imports_macro(tree: &syn::UseTree) -> bool {
    match tree {
        syn::UseTree::Path(p) => imports_macro(&p.tree),
        syn::UseTree::Name(n) => n.ident == MACRO,
        syn::UseTree::Rename(r) => r.ident == MACRO,
        syn::UseTree::Group(g) => g.items.iter().any(imports_macro),
        syn::UseTree::Glob(_) => false,
    }
}

/// Insert `register_structs` into a braced import list, keeping the ASCII
/// ordering rustfmt produces.
fn widen_group(text: &str) -> Option<String> {
    let open = text.rfind('{')?;
    let close = text.rfind('}')?;
    if close < open {
        return None;
    }
    let mut names = split_top_level(&text[open + 1..close]);
    names.push(MACRO.to_string());
    names.sort();
    names.dedup();
    Some(format!("{}{}}};", &text[..=open], names.join(", ")))
}

/// Split a brace group on its top-level commas, so that a nested group such
/// as `interfaces::{Readable, Writeable}` stays in one piece.
fn split_top_level(inner: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    for ch in inner.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    parts.push(current);
    parts
        .into_iter()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{split_top_level, widen_group};

    #[test]
    fn inserts_in_ascii_order() {
        let text = "use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields};";
        assert_eq!(
            widen_group(text).unwrap(),
            "use kernel::utilities::registers::{ReadOnly, ReadWrite, register_bitfields, \
             register_structs};"
        );
    }

    #[test]
    fn nested_group_is_not_split() {
        let parts = split_top_level("ReadOnly, interfaces::{Readable, Writeable}, ReadWrite");
        assert_eq!(
            parts,
            vec!["ReadOnly", "interfaces::{Readable, Writeable}", "ReadWrite"]
        );
    }
}
