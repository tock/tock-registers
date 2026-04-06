// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// TODO: We should have separate types for non-null pointers and nullable pointers. Implement that.
//       Can we make the nullability a generic argument so we don't have to write everything 2x?
// TODO: Make it easier to construct Mmio32/Mmio64.
// TODO: Decide whether Mmio32 should have a Bus<u64> impl.
// TODO: Implement ExactIndexArray. Do we move LEN to be a trait parameter, or do we have to update
//       parsing code?
// TODO: Implement FakeArrayRegister.
// TODO: Implement UnimplementedRegister, add to operation documentation.
// TODO: Implement macro that automatically provides the type defs for Interface trait impls.
// TODO: Implement UnsafeRead/UnsafeWrite. Should they be supertraits of Read/Write or independent?
// TODO: Add --all-targets or --all to the clippy run in the Makefile (requires fixing existing
//       code).
// TODO: Implement a "clippy test" -> a crate that uses tock-registers with as many lints as
//       possible enabled (to verify we don't trip any of them).
// TODO: Are doc comments on padding desirable? If not -> remove support (AST update?).
// TODO: Improve parse error handling. There might be three classes of errors:
//       1. Errors which immediately terminate parsing (e.g. unexpected token)
//       2. Errors which prevent generating code, but allow parsing to continue (e.g. register
//          reference must be to a module)
//       3. Errors where we can still generate code (e.g. multiple #[aliased] attributes)
// TODO: Implement a arm64_secure_vm feature (see the TODO in src/mmio.rs).
// TODO: Investigate adding typestates into the API.
// TODO: Investigate dependency reductions (both syn/quote/proc-macro2 and
//       prettyplease/pretty-assertions).
// TODO: Re-evaluate which `syn` features we need (is full necessary?).

// Questions to ask the group:
// TODO: Do we want to rename something? We have the Register trait, registers module, and
//       registers! macro. Maybe we need to rename registers! ?
// TODO: Do we want to remove the trailing commas after declarations? Easy to do, but a bit harder
//       to revert (have some Punctuated iterator code that I don't want to rewrite).

mod ast;
mod block;
#[cfg(test)]
mod block_test_all_fields;
#[cfg(test)]
mod block_test_docs;
#[cfg(test)]
mod block_test_empty;
#[cfg(test)]
mod block_test_offsets;
mod parse;
#[cfg(test)]
mod parse_tests;
mod single;
#[cfg(test)]
mod single_test_basic;
#[cfg(test)]
mod single_test_docs;
#[cfg(test)]
mod test_util;

use ast::{Input, RegisterSpec, Value};
use proc_macro2::TokenStream;
use quote::quote;
use std::mem::replace;
use syn::{parse_macro_input, Ident, Path, PathArguments};

#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate(parse_macro_input!(input as Input)).into()
}

/// Returns the generated code for a registers! invocation.
fn generate(input: Input) -> TokenStream {
    let mut out = TokenStream::new();
    for definition in input.definitions {
        out.extend(match &definition.value {
            Value::Single(register) => {
                single::generate(&input.tock_registers, &definition, register)
            }
            Value::Block(fields) => block::generate(&input.tock_registers, &definition, fields),
        });
    }
    out
}

/// Generates the Real struct for a register definition (one that has an operations list).
/// `struct_name` is the name of the struct to generate, which may not match the name of the
/// register.
fn register_definition(
    tock_registers: &Path,
    docs: TokenStream,
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
        #docs pub struct #struct_name<B: Bus> {
            address: B,
            _phantom: #tock_registers::internal::RealPhantom,
        }
        impl<B: Bus> #struct_name<B> {
            #new_comment pub const unsafe fn new(address: B) -> Self {
                Self { address, _phantom: #tock_registers::internal::RealPhantom::new() }
            }
        }
        impl<B: Bus> #tock_registers::internal::core::clone::Clone for #struct_name<B> {
            #[inline] fn clone(&self) -> Self { *self }
        }
        impl<B: Bus> #tock_registers::internal::core::marker::Copy for #struct_name<B> {}
        impl<B: Bus> #tock_registers::Block for #struct_name<B> {
            type Address = B;
            const SIZE: usize = <B as #tock_registers::DataTypeBus<#element_type>>::PADDED_SIZE;
            unsafe fn new(address: B) -> Self {
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
        ///    `tock_registers::registers!` macro) must correctly describe the
        ///    pointed-to register(s).
        /// 3. The returned register accessor must not be used in a way that
        ///    causes data races. The exact requirements depend on the hardware,
        ///    but it's usually best to access a register from only one thread
        ///    at a time.
    }
}
