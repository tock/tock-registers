// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::register_definition;
use crate::ast::{Definition, Field, FieldDef, PerBusInt};
use crate::new_doc_comment;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Ident, Path, TypePath};

/// Generates the module for a register block.
pub fn generate(tock_registers: &Path, definition: &Definition, fields: &[Field]) -> TokenStream {
    // At a high level, this function has:
    //
    // 1. A set of variable declarations (of type TokenStream or Vec<TokenStream>)
    // 2. A loop that loops through each field, adding to the variable declarations.
    // 3. A final quote! invocation that combines the variable declarations into the output code.
    //
    // This pattern avoids the need to have separate loops for each portion of the generated code,
    // which removes a lot of code duplication.
    //
    // This is generally more complex than single::generate, so if you need to understand/modify
    // both functions, I suggest working on single::generate first. That knowledge will transfer
    // over to this function.
    //
    // The advice from single::generate's comment about looking at the generated code first, then
    // tracing backwards through the final quote! invocation to find the relevant variables works
    // in this function as well. I suggest starting from block_test_all_fields.

    // Step 1: variable declarations
    let docs = &definition.docs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let interface_comment = interface_doc_comment();
    let mut interface_fields = TokenStream::new();
    let mut len_definitions = TokenStream::new();
    let bus_comment = bus_doc_comment();
    let mut bus_bounds = TokenStream::new();
    let mut bus_offset_decls = TokenStream::new();
    let buses = &definition.buses;
    // cumulative_sizes is empty if the current cumulative size is unknown (due to a padding field
    // with no specified size).
    let mut cumulative_sizes: Vec<_> = (0..buses.len()).map(|_| quote![0]).collect();
    let mut bus_offset_defs: Vec<_> = (0..buses.len()).map(|_| TokenStream::new()).collect();
    let mut borrowed_bus_defs = TokenStream::new();
    let mut offset_tests = TokenStream::new();
    let real_comment = real_doc_comment();
    let new_comment = new_doc_comment();
    let mut interface_bounds = TokenStream::new();
    let mut interface_impl_items = TokenStream::new();
    let mut real_structs = TokenStream::new();

    // Step 2: Loop through each field, update the variables.
    for field in fields.iter() {
        // This `match` statement handles padding and `continue`s to the next iteration on padding
        // fields, so the rest of the body of this loop does not need to special-case for padding.
        let (docs, aliased, name, register) = match &field.field_def {
            FieldDef::Padding(sizes) => {
                add_offset_tests(&mut offset_tests, buses, &cumulative_sizes, &field.offsets);
                cumulative_sizes.clear();
                if let Some(sizes) = sizes {
                    for bus_idx in 0..buses.len() {
                        let offset = &field.offsets[bus_idx];
                        let size = &sizes[bus_idx];
                        cumulative_sizes.push(quote![#offset + #size]);
                    }
                }
                continue;
            }
            FieldDef::Register {
                docs,
                aliased,
                name,
                definition,
            } => (docs, *aliased, name, definition),
        };
        // The rest of this loop body is for register fields. It consists of a series of
        // conditionals and loops that all switch/iterate on a different aspect of the field.
        let element_type = &register.element_type;
        let mut interface_bound;
        let mut real;
        // If statement that handles differences between register definitions (which have
        // operations) and register references (which do not).
        if let Some(operations) = &register.operations {
            interface_bound =
                quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
            let real_name = format_ident!("real_{name}");
            real = quote![#real_name<B>];
            let bus_trait = quote![#tock_registers::DataTypeBus<#element_type>];
            bus_bounds.extend(quote![+ #bus_trait]);
            real_structs.extend(register_definition(
                tock_registers,
                field_struct_doc_comment(name),
                &real_name,
                register,
                operations,
            ));
        } else {
            interface_bound = quote![#element_type::Interface];
            real = quote![#element_type::Real<B>];
            bus_bounds.extend(quote_spanned![element_type.span()=>+ #element_type::Bus]);
        };
        interface_bounds.extend(quote![#real: #interface_bound,]);
        // match that handles the difference between scalar registers, non-nested array registers,
        // and nested array registers.
        let len_types_sizes = match register.array_sizes.as_slice() {
            [] => vec![],
            [len] => {
                len_definitions.extend(quote![pub enum #name {}]);
                vec![(quote![#name], len)]
            }
            nested => {
                len_definitions.extend(quote![pub enum #name<const N: usize> {}]);
                nested
                    .iter()
                    .enumerate()
                    .map(|(n, s)| (quote![#name<#n>], s))
                    .collect()
            }
        };
        // Loop that runs once for each level of array nesting.
        for (len_type, size) in len_types_sizes {
            interface_bound =
                quote![#tock_registers::RegisterArray<lens::#len_type, Element: #interface_bound>];
            len_definitions.extend(quote! {
                impl #tock_registers::array::Len for #len_type { const LEN: usize = #size; }
            });
            real = quote![#tock_registers::RealRegisterArray<#real, lens::#len_type>];
        }
        interface_fields.extend(quote! {
            type #name: #interface_bound;
            #(#docs)* fn #name(self) -> Self::#name;
        });
        let name_offset = format_ident!("{name}_offset");
        // match that handles the difference between scalar offset definitions and array offset
        // definitions (moves the value of the name_offset fields between the Bus trait definition
        // and impls).
        bus_offset_decls.extend(match &field.offsets {
            PerBusInt::Array(offsets) => {
                for (bus_idx, offset) in offsets.iter().enumerate() {
                    bus_offset_defs[bus_idx].extend(quote![const #name_offset: usize = #offset;]);
                }
                borrowed_bus_defs
                    .extend(quote![const #name_offset: usize = <B as Bus>::#name_offset;]);
                quote![const #name_offset: usize;]
            }
            PerBusInt::Single(offset) => quote![const #name_offset: usize = #offset;],
        });
        // if that handles aliased vs. non-aliased fields.
        if !aliased {
            add_offset_tests(&mut offset_tests, buses, &cumulative_sizes, &field.offsets);
            cumulative_sizes.clear();
            for (bus_idx, bus) in buses.iter().enumerate() {
                let offset = &field.offsets[bus_idx];
                cumulative_sizes.push(quote![#offset + <<Real<#bus> as Interface>::#name as #tock_registers::Span>::SIZE]);
            }
        }
        interface_impl_items.extend(quote! {
            type #name = #real;
            fn #name(self) -> Self::#name {
                // Safety (see crate::new_doc_comment() for requirements):
                // 1. When Self::new was called to construct `self`, the caller guaranteed that the
                //    passed address points to registers on the bus of type B.
                // 2. The definition of this register block is correct (guaranteed by the caller of
                //    Self::new), which guarantees that a register corresponding to the #real type
                //    exists at B::name_offset. This also guarantees that the byte_add does not
                //    leave the register block.
                // 3. The user of this struct is responsible for using the entire register block in
                //    a way that avoids data races, which includes the responsibility to avoid data
                //    races on individual fields of the register block.
                unsafe {
                    Self::#name::new(self.address.byte_add(<B as Bus>::#name_offset))
                }
            }
        });
    }

    // Step 3: the final quote! call that puts everything together.
    quote! {
        #(#docs)*
        #visibility mod #name {
            #![allow(clippy::expl_impl_clone_on_copy)]
            #![allow(nonstandard_style)]
            use super::*;
            #interface_comment pub trait Interface: #tock_registers::internal::core::marker::Copy {
                #interface_fields
            }
            pub mod lens { #len_definitions }
            #bus_comment pub trait Bus: #tock_registers::Address #bus_bounds + sealed::Bus {
                const SIZE: usize;
                #bus_offset_decls
            }
            #(
                impl Bus for #buses {
                    const SIZE: usize = #cumulative_sizes;
                    #bus_offset_defs
                }
                impl sealed::Bus for #buses {}
            )*
            impl<B: Bus> Bus for #tock_registers::BorrowedBus<'_, B> {
                const SIZE: usize = <B as Bus>::SIZE;
                #borrowed_bus_defs
            }
            impl<B: Bus> sealed::Bus for #tock_registers::BorrowedBus<'_, B> {}
            #[allow(clippy::eq_op)] const _: () = { #offset_tests };
            mod sealed { pub trait Bus {} }
            #real_comment pub struct Real<B: Bus> {
                address: B,
                _phantom: #tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: #tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> #tock_registers::internal::core::clone::Clone for Real<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> #tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B> where #interface_bounds {
                #interface_impl_items
            }
            impl<B: Bus> #tock_registers::Span for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::SIZE;
                unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: #tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<#tock_registers::BorrowedBus<'b, B>>;
            }
            #real_structs
        }
    }
}

pub fn interface_doc_comment() -> TokenStream {
    quote! {
        /// Trait representing this register block. Driver code can use this trait to work with
        /// both real hardware and fake implementations of the peripheral (for unit testing).
    }
}

pub fn bus_doc_comment() -> TokenStream {
    quote! {
        /// Buses supported by this register block.
    }
}

pub fn real_doc_comment() -> TokenStream {
    quote! {
        /// Struct implementing [Interface] for use with the real hardware.
    }
}

pub fn field_struct_doc_comment(name: &Ident) -> TokenStream {
    let msg = format!("Struct that provides access to the `{name}` register on real hardware.");
    quote![#[doc = #msg]]
}

/// Adds offset tests for a field with the given offsets. If the current cumulative size is unknown
/// (because of a padding field with no specified size), this does nothing.
fn add_offset_tests(
    offset_tests: &mut TokenStream,
    buses: &[TypePath],
    cumulative_sizes: &[TokenStream],
    offsets: &PerBusInt,
) {
    for (bus_idx, cumulative_size) in cumulative_sizes.iter().enumerate() {
        let offset = &offsets[bus_idx];
        let bus = &buses[bus_idx].path.segments.last().expect("empty bus path");
        let msg = format!("offset mismatch for bus {}", bus.ident);
        offset_tests
            .extend(quote_spanned![offset.span()=>assert!(#offset == #cumulative_size, #msg);]);
    }
}
