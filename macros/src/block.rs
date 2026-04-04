// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::register_definition;
use crate::ast::{Definition, Field, FieldDef, PerBusInt};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident, Path};

/// Generates the module for a register block.
pub fn generate(tock_registers: &Path, definition: &Definition, fields: &[Field]) -> TokenStream {
    let docs = &definition.docs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let interface_comment = interface_doc_comment();
    let mut interface_fields = TokenStream::new();
    let bus_comment = bus_doc_comment();
    let mut bus_bounds = TokenStream::new();
    let mut bus_offset_decls = TokenStream::new();
    let buses = &definition.buses;
    let mut block_sizes: Vec<_> = (0..buses.len()).map(|_| quote![0]).collect();
    let mut bus_offset_defs: Vec<_> = (0..buses.len()).map(|_| TokenStream::new()).collect();
    let mut offset_tests = TokenStream::new();
    let real_comment = real_doc_comment();
    let mut interface_bounds = TokenStream::new();
    let mut interface_impl_items = TokenStream::new();
    let mut real_structs = TokenStream::new();
    for field in fields.iter() {
        let docs = &field.docs;
        let (aliased, name, register) = match &field.field_def {
            FieldDef::Padding(sizes) => {
                for bus_idx in 0..buses.len() {
                    let offset = &field.offsets[bus_idx];
                    let block_size = &block_sizes[bus_idx];
                    offset_tests.extend(quote_spanned![offset.span()=>
                        assert!(#offset == #block_size, "offset mismatch");]);
                    let size = &sizes[bus_idx];
                    block_sizes[bus_idx].extend(quote![+ #size]);
                }
                continue;
            }
            FieldDef::Register {
                aliased,
                name,
                definition,
            } => (*aliased, name, definition),
        };
        let element_type = &register.element_type;
        let mut interface_bound;
        let mut real;
        if let Some(operations) = &register.operations {
            interface_bound =
                quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
            let real_name = Ident::new(&format!("real_{name}"), name.span());
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
        for array_size in &register.array_sizes {
            interface_bound = quote![#tock_registers::RegisterArray<Element: #interface_bound>];
            real = quote![#tock_registers::RealRegisterArray<#real, #array_size>];
        }
        interface_fields.extend(quote! {
            type #name: #interface_bound;
            #(#docs)* fn #name(self) -> Self::#name;
        });
        let name_offset = Ident::new(&format!("{name}_offset"), name.span());
        bus_offset_decls.extend(match &field.offsets {
            PerBusInt::Array(offsets) => {
                for (bus_idx, offset) in offsets.iter().enumerate() {
                    bus_offset_defs[bus_idx].extend(quote![const #name_offset: usize = #offset;]);
                }
                quote![const #name_offset: usize;]
            }
            PerBusInt::Single(offset) => quote![const #name_offset: usize = #offset;],
        });
        if !aliased {
            for (bus_idx, bus) in buses.iter().enumerate() {
                let offset = &field.offsets[bus_idx];
                let block_size = &block_sizes[bus_idx];
                offset_tests.extend(quote_spanned![offset.span()=>
                    assert!(#offset == #block_size, "offset mismatch");]);
                block_sizes[bus_idx] = quote![#offset +
                    <<Real<#bus> as Interface>::#name as #tock_registers::Block>::SIZE];
            }
        }
        interface_impl_items.extend(quote! {
            type #name = #real;
            fn #name(self) -> Self::#name {
                unsafe {
                    Self::#name::new(self.0.byte_add(<B as Bus>::#name_offset))
                }
            }
        });
    }
    quote! {
        #(#docs)*
        #visibility mod #name {
            #![allow(clippy::expl_impl_clone_on_copy)]
            #![allow(non_camel_case_types)]
            use super::*;
            #interface_comment pub trait Interface: #tock_registers::internal::core::marker::Copy {
                #interface_fields
            }
            #[allow(non_upper_case_globals)]
            #bus_comment pub trait Bus: #tock_registers::Address #bus_bounds + sealed::Bus {
                const BLOCK_SIZE: usize;
                #bus_offset_decls
            }
            #(
                impl Bus for #buses {
                    const BLOCK_SIZE: usize = #block_sizes;
                    #bus_offset_defs
                }
                impl sealed::Bus for #buses {}
            )*
            const _: () = { #offset_tests };
            mod sealed { pub trait Bus {} }
            #real_comment pub struct Real<B: Bus>(B);
            impl<B: Bus> Real<B> {
                pub const unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> #tock_registers::internal::core::clone::Clone for Real<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> #tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B> where #interface_bounds {
                #interface_impl_items
            }
            impl<B: Bus> #tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_SIZE;
                unsafe fn new(address: B) -> Self { Self(address) }
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
