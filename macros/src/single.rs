// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::register_definition;
use crate::ast::{Definition, RegisterSpec};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident, Path};

pub fn generate(
    tock_registers: &Path,
    definition: &Definition,
    register: &RegisterSpec,
) -> TokenStream {
    let is_scalar = register.array_sizes.is_empty();
    let is_definition = register.operations.is_some();
    let element_type = &register.element_type;
    let docs = &definition.docs;
    let visibility = &definition.visibility;
    let name = &definition.name;
    let mut allows = TokenStream::new();
    let interface_comment = interface_doc_comment();
    let element_bound;
    let bus_comment = bus_doc_comment();
    let bus_bound;
    let buses = &definition.buses;
    let element_definition;
    let mut real;
    if let Some(operations) = &register.operations {
        // This RegisterSpec is a register definition.
        allows = quote![#![allow(clippy::expl_impl_clone_on_copy)]];
        element_bound =
            quote![#tock_registers::Register<DataType = #element_type> #(+ #operations)*];
        bus_bound =
            quote_spanned![element_type.span()=>#tock_registers::DataTypeBus<#element_type>];
        let struct_name = match is_scalar {
            true => "Real",
            false => "Element",
        };
        element_definition = register_definition(
            tock_registers,
            struct_doc_comment(is_scalar),
            &Ident::new(struct_name, Span::call_site()),
            register,
            operations,
        );
        real = quote![Element<B>];
    } else {
        // This RegisterSpec is a register reference.
        element_bound = quote![#element_type::Interface];
        bus_bound = quote_spanned![element_type.span()=>#element_type::Bus];
        element_definition = quote![];
        real = quote![#element_type::Real<B>];
    }
    let mut interface_bound = element_bound.clone();
    for size in &register.array_sizes {
        interface_bound = quote![#tock_registers::RegisterArray<Element: #interface_bound>];
        real = quote![#tock_registers::RealRegisterArray<#real, #size>];
    }
    let real_alias = if is_scalar && is_definition {
        quote![]
    } else {
        let real_alias_comment = real_alias_doc_comment();
        quote![#real_alias_comment pub type Real<B> = #real;]
    };
    let impl_bound_type = match (is_scalar, is_definition) {
        (true, _) => quote![Self],
        (false, true) => quote![Element<B>],
        (false, false) => quote![#element_type::Real<B>],
    };
    quote! {
        #(#docs)*
        #visibility mod #name {
            #allows
            use super::*;
            #interface_comment pub trait Interface: #interface_bound {}
            #bus_comment pub trait Bus: #bus_bound + sealed::Bus {}
            #(impl Bus for #buses {})*
            #(impl sealed::Bus for #buses {})*
            impl<B: Bus> Bus for #tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for #tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            #element_definition
            #real_alias
            impl<B: Bus> Interface for Real<B> where #impl_bound_type: #element_bound {}
        }
    }
}

pub fn interface_doc_comment() -> TokenStream {
    quote! {
        /// Trait representing this register's operations. Driver code can use this trait to work
        /// with both real hardware and fake implementations of the register (for unit testing).
    }
}

pub fn bus_doc_comment() -> TokenStream {
    quote! {
        /// Buses supported by this register.
    }
}

pub fn struct_doc_comment(is_scalar: bool) -> TokenStream {
    match is_scalar {
        true => quote! {
            /// Struct that implements [Interface] for use with the real hardware.
        },
        false => quote! {
            /// Implementation of an element of this register array for use with real hardware.
            /// This implements the tock_registers::Register trait as well as any operation traits
            /// specified in the register definition.
        },
    }
}

pub fn real_alias_doc_comment() -> TokenStream {
    quote! {
        /// Implementation of [Interface] for use with real hardware.
    }
}
