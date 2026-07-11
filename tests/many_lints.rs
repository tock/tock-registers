// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests `register_map!` with many lints enabled to ensure the generated code does not
//! unnecessarily trigger warnings or errors in user code compiled with strict lints.

#![no_std]
// -------------------------------------------------------------------------------------------------
// rustc allowed-by-default lints. For ease of future updates, this is every lint from
// https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html that is recognized by MSRV
// toolchain.
// -------------------------------------------------------------------------------------------------
#![forbid(absolute_paths_not_starting_with_crate)]
#![forbid(ambiguous_negative_literals)]
// async_idents was renamed to keyword_idents_2018
#![forbid(deprecated_safe_2024)]
#![forbid(deref_into_dyn_supertrait)]
// disjoint_capture_migration was renamed to rust_2021_incompatible_closure_captures
#![forbid(edition_2024_expr_fragment_specifier)]
// elided_lifetime_in_path was renamed to elided_lifetimes_in_paths
#![forbid(elided_lifetimes_in_paths)]
#![forbid(explicit_outlives_requirements)]
#![forbid(ffi_unwind_calls)]
#![forbid(impl_trait_overcaptures)]
#![forbid(impl_trait_redundant_captures)]
#![forbid(keyword_idents)]
#![forbid(keyword_idents_2018)]
#![forbid(keyword_idents_2024)]
#![forbid(let_underscore_drop)]
#![forbid(macro_use_extern_crate)]
#![forbid(meta_variable_misuse)]
#![forbid(missing_copy_implementations)]
#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![forbid(missing_unsafe_on_extern)]
#![forbid(non_ascii_idents)]
// or_patterns_back_compat was renamed to rust_2021_incompatible_or_patterns
#![forbid(redundant_imports)]
#![forbid(redundant_lifetimes)]
#![forbid(rust_2021_incompatible_closure_captures)]
#![forbid(rust_2021_incompatible_or_patterns)]
#![forbid(rust_2021_prefixes_incompatible_syntax)]
#![forbid(rust_2021_prelude_collisions)]
#![forbid(rust_2024_prelude_collisions)]
// single_use_lifetime was renamed to single_use_lifetimes
#![forbid(single_use_lifetimes)]
#![forbid(tail_expr_drop_order)]
#![forbid(trivial_casts)]
#![forbid(trivial_numeric_casts)]
#![forbid(unit_bindings)]
#![forbid(unnameable_types)]
#![forbid(unreachable_pub)]
#![forbid(unsafe_attr_outside_unsafe)]
#![forbid(unsafe_code)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![forbid(unstable_features)]
// unused_crate_dependencies cannot be satisfied (macros cannot control Cargo.toml)
#![forbid(unused_extern_crates)]
#![forbid(unused_import_braces)]
#![forbid(unused_lifetimes)]
#![forbid(unused_macro_rules)]
#![forbid(unused_qualifications)]
#![forbid(unused_results)]
#![forbid(variant_size_differences)]
// -------------------------------------------------------------------------------------------------
// Clippy lints
// -------------------------------------------------------------------------------------------------
// Enable all Clippy lint groups. We forbid every group possible. For the ones that cannot be
// forbidden, we have a comment explaining why.
//
// blanket_clippy_restriction_lints is allowed
#![deny(clippy::all)]
// clippy::cargo skipped because the generated code does not control Cargo.toml
#![forbid(clippy::complexity)]
#![forbid(clippy::correctness)]
#![deny(clippy::nursery)] // trait_duplication_in_bounds is allowed in the generated code
#![forbid(clippy::pedantic)]
#![forbid(clippy::perf)]
#![deny(clippy::restriction)] // unseparated_literal_suffix allowed in this file
#![forbid(clippy::style)]
#![deny(clippy::suspicious)] // blanket_clippy_restriction_lints allowed in this file
//
// Lints allowed out of necessity
#![allow(
    clippy::blanket_clippy_restriction_lints,
    reason = "purpose of this test"
)]
#![allow(
    clippy::unseparated_literal_suffix,
    reason = "incompatible with separated_literal_suffix"
)]
// -------------------------------------------------------------------------------------------------
// Some lints are automatically suppressed in proc macro outputs. tock-registers-codegen emits
// allow() attributes for those lints when run in code generation mode; this verifies that those
// allow() attributes do not exist in the proc macro output.
// -------------------------------------------------------------------------------------------------
#![forbid(non_upper_case_globals)]
#![forbid(unused)]

use tock_registers::{register_map, Mmio32, Mmio64, Read, Write};

register_map! {
    #![buses(Mmio32, Mmio64)]
    aa: u8 { Read, Write },
    bb: u8 { Write },
    cc {
        0usize => scalar_definition: u8 { Read },
        1usize => array_definition: [[u8; 2]; 3] { Read, Write },
        7usize => _: [1usize, 1usize],
        [8usize, 8usize] => scalar_reference: aa,
        9usize => array_reference: [[bb; 2]; 3],
    }
}
