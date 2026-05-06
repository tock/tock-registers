// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use super::register_definition;
use crate::ast::{Layout, RegisterSpec};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident, Path};

/// Generates the module for a single register definition.
pub fn generate(tock_registers: &Path, layout: &Layout, register: &RegisterSpec) -> TokenStream {
    // At a high level, this function has:
    //
    // 1. A set of variable declarations, mostly of type TokenStream
    // 2. A series of loops/conditionals, each of which switch/iterate on a different aspect of the
    //    layout.
    // 3. A final quote! invocation that combines the variable declarations into the output code.
    //
    // This pattern avoids a lot of code duplication that would occur if we tried to generate each
    // piece of the output module in sequence.
    //
    // When you're trying to understand this code, I suggest:
    //
    // 1. Finding the portion of the generated code you're interested in within scalar_test_basic
    // 2. Tracing that portion of the generated code back through the final quote! invocation to
    //    the variables that produced it.
    // 3. Searching for uses of those variables to see how they are created.

    // Step 1: variable declarations
    let is_scalar = register.array_sizes.is_empty();
    let is_definition = register.operations.is_some();
    let element_type = &register.element_type;
    let docs = &layout.docs;
    let visibility = &layout.visibility;
    let name = &layout.name;
    let mut allows = TokenStream::new();
    let interface_comment = interface_doc_comment();
    let element_bound;
    let bus_comment = bus_doc_comment();
    let bus_bound;
    let buses = &layout.buses;
    let element_definition;
    let mut real;

    // Step 2: Work through different parts of the input to set the output variables.

    // If statement that handles differences between register definitions (which have operations)
    // and register references (which do not).
    if let Some(operations) = &register.operations {
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
        element_bound = quote![#element_type::Interface];
        bus_bound = quote_spanned![element_type.span()=>#element_type::Bus];
        element_definition = quote![];
        real = quote![#element_type::Real<B>];
    }
    let mut interface_bound = element_bound.clone();
    // match that handles the difference between scalar registers, non-nested array registers, and
    // nested array registers.
    let (mut len_definition, len_types_sizes) = match register.array_sizes.as_slice() {
        [] => (TokenStream::new(), vec![]),
        #[rustfmt::skip]
        [len] => (quote![pub enum Len {}], vec![(quote![Len], len)]),
        #[rustfmt::skip]
        nested => (
            quote![pub enum Len<const N: usize> {}],
            nested
                .iter()
                .enumerate()
                .map(|(n, s)| (quote![Len<#n>], s))
                .collect(),
        ),
    };
    // Loop that runs once for each level of array nesting.
    for (len_type, size) in len_types_sizes {
        interface_bound =
            quote![#tock_registers::RegisterArray<#len_type, Element: #interface_bound>];
        len_definition.extend(quote! {
            impl #tock_registers::array::Len for #len_type { const LEN: usize = #size; }
        });
        real = quote![#tock_registers::RealRegisterArray<#real, #len_type>];
    }
    // If statement that switches on whether this is a scalar register definition or not.
    let real_alias = if is_scalar && is_definition {
        quote![]
    } else {
        let real_alias_comment = real_alias_doc_comment();
        quote![#real_alias_comment pub type Real<B> = #real;]
    };
    // Match statement that switches on whether this is a scalar register, array definition, or
    // array reference.
    let impl_bound_type = match (is_scalar, is_definition) {
        (true, _) => quote![Self],
        (false, true) => quote![Element<B>],
        (false, false) => quote![#element_type::Real<B>],
    };

    // Step 3: the final quote! call that puts everything together.
    quote! {
        #(#docs)*
        #visibility mod #name {
            #allows
            use super::*;
            #interface_comment pub trait Interface: #interface_bound {}
            #len_definition
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
