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

mod ast;
mod block;
#[cfg(all(test, not(miri)))]
mod block_test_all_fields;
#[cfg(all(test, not(miri)))]
mod block_test_docs;
#[cfg(all(test, not(miri)))]
mod block_test_empty;
#[cfg(all(test, not(miri)))]
mod block_test_offsets;
mod parse;
#[cfg(all(test, not(miri)))]
mod parse_tests;
mod single;
#[cfg(all(test, not(miri)))]
mod single_test_array;
#[cfg(all(test, not(miri)))]
mod single_test_docs;
#[cfg(all(test, not(miri)))]
mod single_test_scalar;
#[cfg(all(test, not(miri)))]
mod test_util;

use ast::{Input, RegisterSpec, Value};
use proc_macro2::TokenStream;
use quote::quote;
use std::mem::replace;
use syn::{parse2, Ident, Path, PathArguments};

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
    use Value::{Block, Single};
    let input: Input = parse2(input).map_err(|e| e.to_compile_error())?;
    let mut out = TokenStream::new();
    for layout in input.layouts {
        out.extend(match &layout.value {
            Block(fields) => block::generate(env, &input.tock_registers, &layout, fields),
            Single(register) => single::generate(env, &input.tock_registers, &layout, register),
        });
    }
    Ok(out)
}

/// register_map generates slightly different code (in particular, different `#![allow()]`
/// attributes) depending on whether it is run as part of a procedural macro or run externally to
/// rustc. This enum is used to tell register_map which mode to use.
#[derive(Clone, Copy)]
pub enum Env {
    /// Generate code suitable to feed into a separate rustc invocation run.
    External,
    /// Generate code tuned for procedural macros.
    ProcMacro,
}

/// Generates the Real struct for a register definition (one that has an operations list).
/// `struct_name` is the name of the struct to generate, which does not need to match the name of
/// the register.
fn register_definition(
    tock_registers: &Path,
    docs: TokenStream,
    bus_default: &TokenStream,
    struct_name: &Ident,
    register: &RegisterSpec,
    operations: &[Path],
) -> TokenStream {
    let new_comment = new_doc_comment();
    let element_type = &register.element_type;
    let mut op_macros = Vec::with_capacity(operations.len());
    let mut op_generics = Vec::with_capacity(operations.len());
    for mut path in operations.iter().cloned() {
        let last = path.segments.last_mut().expect("empty operation path");
        op_generics.push(replace(&mut last.arguments, PathArguments::None));
        op_macros.push(path);
    }
    quote! {
        #docs #[derive(Clone)] pub struct #struct_name<B: Bus #bus_default> {
            address: B,
            _phantom: #tock_registers::internal::RealPhantom,
        }
        impl<B: Bus> #struct_name<B> {
            #new_comment pub const unsafe fn new(address: B) -> Self {
                Self { address, _phantom: #tock_registers::internal::RealPhantom::new() }
            }
        }
        impl<B: Bus> #tock_registers::internal::core::marker::Copy for #struct_name<B> {}
        impl<B: Bus> #tock_registers::Span for #struct_name<B> {
            type Address = B;
            const SIZE: usize = <B as #tock_registers::DataTypeBus<#element_type>>::PADDED_SIZE;
            unsafe fn with_addr(address: B) -> Self {
                Self { address, _phantom: #tock_registers::internal::RealPhantom::new()  }
            }
            type Borrowed<'b> = #struct_name<#tock_registers::BorrowedBus<'b, B>>;
        }
        impl<B: Bus> #tock_registers::Register for #struct_name<B> {
            type DataType = #element_type;
        }
        #(#op_macros!(real_impl, #struct_name, #element_type, #op_generics,);)*
    }
}

/// Returns the block comment for the `new` function for a register or register block.
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
