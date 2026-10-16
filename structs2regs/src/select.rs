// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Deciding which structs may be converted, and refusing the rest.
//!
//! Every refusal here is a structural property of the struct's own syntax,
//! checked before any offset is computed. The tool therefore never abandons a
//! half-converted struct.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{File, ItemStruct};

use crate::crate_scan::CrateScan;
use crate::sizes::is_mmio_register;

pub enum Plan<'a> {
    NoCfg(Vec<&'a syn::Field>),
    WithCfg {
        cfg: String,
        with: Vec<&'a syn::Field>,
        without: Vec<&'a syn::Field>,
    },
}
/// Exactly `#[repr(C)]`, with no other `repr` and nothing else inside it.
///
/// `#[repr(C, align(32))]` and a separate `#[repr(align(512))]` are both
/// rejected: `register_structs!` emits its own `#[repr(C)]` and an added
/// alignment interacts with erasing a reserved field's type.
pub fn has_plain_repr_c(attrs: &[syn::Attribute]) -> bool {
    let reprs: Vec<_> = attrs.iter().filter(|a| a.path().is_ident("repr")).collect();
    match reprs.as_slice() {
        [only] => match &only.meta {
            syn::Meta::List(list) => list.tokens.to_string() == "C",
            _ => false,
        },
        _ => false,
    }
}

/// The predicate of a `#[cfg(..)]` attribute, as written.
pub fn cfg_predicate(attrs: &[syn::Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|a| a.path().is_ident("cfg"))
        .and_then(|a| match &a.meta {
            syn::Meta::List(list) => Some(list.tokens.to_string()),
            _ => None,
        })
}

/// Returns a list of [syn::ItemStruct]s for conversion.
pub fn filter_target_items(file: &File) -> Vec<(&ItemStruct, Span)> {
    // Predict for determining if an [syn::ItemStruct] is a possible
    // target item for conversion: a named-field `repr(C)` struct with
    // at least // one memory-mapped register.
    fn is_candidate(item: &syn::ItemStruct) -> bool {
        if !has_plain_repr_c(&item.attrs) {
            return false;
        }
        let syn::Fields::Named(named) = &item.fields else {
            return false;
        };
        named.named.iter().any(|f| is_mmio_register(&f.ty))
    }

    file.items
        .iter()
        .filter_map(|item| {
            let syn::Item::Struct(item) = item else {
                return None;
            };
            if is_candidate(item) {
                Some((item, Spanned::span(item)))
            } else {
                None
            }
        })
        .collect()
}

pub fn plan<'a>(item: &'a syn::ItemStruct, scan: &CrateScan) -> Result<Plan<'a>, String> {
    let name = item.ident.to_string();

    if !has_plain_repr_c(&item.attrs) {
        return Err(
            "struct has a `repr` beyond plain `#[repr(C)]`; `register_structs!` emits its own              and an added alignment interacts with erasing a reserved field's type"
                .to_string(),
        );
    }
    if !item.generics.params.is_empty() {
        return Err(
            "struct is generic; `register_structs!` accepts at most a lifetime parameter"
                .to_string(),
        );
    }
    if item.attrs.iter().any(|a| a.path().is_ident("derive")) {
        return Err(
            "struct has a `#[derive]`; derived code reads fields without naming them, and \
             macro padding is `[u8; N]`"
                .to_string(),
        );
    }
    if scan.is_literal_constructed(&name) {
        return Err(format!(
            "`{name}` is built with a struct literal somewhere in this crate; macro padding \
             is `[u8; N]`, which such an initializer cannot name"
        ));
    }

    let syn::Fields::Named(named) = &item.fields else {
        return Err("struct has no named fields".to_string());
    };
    let all: Vec<&syn::Field> = named.named.iter().collect();

    let mut predicates: Vec<String> = Vec::new();
    for field in &all {
        if let Some(p) = cfg_predicate(&field.attrs)
            && !predicates.contains(&p)
        {
            predicates.push(p);
        }
    }

    match predicates.as_slice() {
        [] => Ok(Plan::NoCfg(all)),
        [predicate] => {
            let without: Vec<&syn::Field> = all
                .iter()
                .copied()
                .filter(|f| cfg_predicate(&f.attrs).is_none())
                .collect();
            Ok(Plan::WithCfg {
                cfg: predicate.clone(),
                with: all,
                without,
            })
        }
        many => Err(format!(
            "{} distinct cfg predicates on fields ({}); each configuration needs its own \
                 invocation, and their combinations are not mechanically separable",
            many.len(),
            many.join(", ")
        )),
    }
}
