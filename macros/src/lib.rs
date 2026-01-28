// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// TODO: Implement #[aliased] (which makes a register *not* participate in offset and block size
//       calculations). Remove my new Aliased and BlockInfo. Question: should we only emit fields
//       with variable offsets, or all fields?
// TODO: Figure out multi-core support. How do we make the real types !Send (by making the Buses
//       not Send?). Do we add some sort of RegisterSender? Can the lifetime be on the bus or does
//       it have to be on Real<> (perhaps a BorrowedBus<'s, B: Bus>?)?
// TODO: Make operations with generic arguments work, and update the documentation for adding new
//       operations (as this will probably add an argument to the operation macro interface). It
//       looks like ast should still use syn::Path to represent operations, but parsing should
//       error if any non-last path segment has arguments.
// TODO: Add Read::debug().
// TODO: Should FakeRegister use the Value type or LocalRegisterCopy?
// TODO: We should have separate types for non-null pointers and nullable pointers. Implement that.
//       Can we make the nullability a generic argument so we don't have to write everything 2x?
// TODO: Make it easier to construct Mmio32/Mmio64.
// TODO: Decide whether Mmio32 should have a Bus<u64> impl.
// TODO: Implement FakeArrayRegister.
// TODO: Implement UnimplementedRegister, add to operation documentation.
// TODO: Implement macro that automatically provides the type defs for Interface trait impls.
// TODO: Implement UnsafeRead/UnsafeWrite. Should they be supertraits of Read/Write or independent?
// TODO: Implement ExactIndexArray. Do we move LEN to be a trait parameter, or do we have to update
//       parsing code?
// TODO: Add --all-targets or --all to the clippy run in the Makefile (requires fixing existing
//       code).
// TODO: Implement a "clippy test" -> a crate that uses tock-registers with as many lints as
//       possible enabled (to verify we don't trip any of them).
// TODO: Are doc comments on padding desirable? If not -> remove support (AST update?).
// TODO: Update parsing logic to support outputting multiple errors at once (for errors that do not
//       need to stop parsing).
// TODO: Re-evaluate which `syn` features we need (is full necessary?).
// TODO: Implement a arm64_secure_vm feature (see the TODO in src/mmio.rs).

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
use syn::{parse_macro_input, Ident, Path};

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
    let element_type = &register.element_type;
    quote! {
        #docs
        pub struct #struct_name<B: Bus>(B);
        impl<B: Bus> #struct_name<B> {
            pub unsafe fn new(address: B) -> Self { Self(address) }
        }
        impl<B: Bus> #tock_registers::internal::core::clone::Clone for #struct_name<B> {
            #[inline] fn clone(&self) -> Self { *self }
        }
        impl<B: Bus> #tock_registers::internal::core::marker::Copy for #struct_name<B> {}
        impl<B: Bus> #tock_registers::Block for #struct_name<B> {
            type Address = B;
            const SIZE: usize = <B as #tock_registers::DataTypeBus<#element_type>>::PADDED_SIZE;
            unsafe fn new(address: B) -> Self {
                Self(address)
            }
        }
        impl<B: Bus> #tock_registers::Register for #struct_name<B> {
            type DataType = #element_type;
        }
        #(#operations!(real_impl, #struct_name, #element_type,);)*
    }
}
