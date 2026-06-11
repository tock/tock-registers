// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// If you're new to this macro's codebase, look at:
// 1. The `ast` module. The rustdoc contains an explanation of the input grammar, and understanding
//    the AST is important for understanding both the parsing module and the code generation
//    modules.
// 2. The generated code in `single_test_scalar`, then `single_test_array`, then
//    `block_test_all_fields`. Nonobvious parts of the generated code are documented in those test
//    cases.

#[cfg(all(test, not(miri)))]
mod test_util;

use proc_macro2::TokenStream;
use quote::quote;

/// Returns the generated code for a `tock_registers_macro::register_map!` invocation.
///
/// # Input
/// `tock_registers::register_map!` prepends `$crate` to the tokens passed to
/// `tock_registers_macro::register_map!`, so that the generated code knows how to find the
/// `tock_registers` crate. Therefore, this function needs `input` to start with the path to the
/// `tock_registers` crate.
///
/// # Return value
/// If an error is encountered, Err() is returned and the contained TokenStream produces a compiler
/// error.
pub fn register_map(input: TokenStream, env: Env) -> Result<TokenStream, TokenStream> {
    let _ = (input, env);
    todo!()
}

/// register_map generates slightly different code (different `#![allow()]` attributes) depending
/// on whether it is run as part of a procedural macro or run externally to rustc. This enum is
/// used to tell register_map which mode to use.
#[derive(Clone, Copy)]
pub enum Env {
    /// Generate code suitable to feed into a separate rustc invocation run.
    External,
    /// Generate code tuned for procedural macros.
    ProcMacro,
}

/// Returns the block comment for the `new` function for a register or register block.
#[allow(unused)] // TODO: Remove once register_definition has been added (as part of the AST PR)
fn new_doc_comment() -> TokenStream {
    quote! {
        /// Constructs an accessor for this register or register block.
        /// # Safety
        /// 1. `address` must point to register(s) on the bus corresponding to
        ///    `B`.
        /// 2. The register(s)' definition (as provided to the
        ///    `tock_registers::register_map!` macro) must correctly
        ///    describe the pointed-to register(s).
        /// 3. The returned register accessor must not be used in a way that
        ///    causes data races. The exact requirements depend on the hardware,
        ///    but it's usually best to access a register from only one thread
        ///    at a time.
    }
}
