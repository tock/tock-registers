// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::block::{bus_doc_comment, interface_doc_comment, real_doc_comment};
use crate::{generate, new_doc_comment, test_util::assert_tokens_eq};
use quote::quote;
use syn::parse_quote;

#[test]
fn empty() {
    let input = parse_quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {}
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_comment = real_doc_comment();
    let new_comment = new_doc_comment();
    let expected = quote! {
        pub mod foo {
            #![allow(clippy::expl_impl_clone_on_copy)]
            #![allow(non_camel_case_types)]
            use super::*;
            #interface_comment
            pub trait Interface: ::tock_registers::internal::core::marker::Copy {}
            #[allow(non_upper_case_globals)]
            #bus_comment pub trait Bus: ::tock_registers::Address + sealed::Bus {
                const BLOCK_SIZE: usize;
            }
            impl Bus for Mmio32 { const BLOCK_SIZE: usize = 0; }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 { const BLOCK_SIZE: usize = 0; }
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {
                const BLOCK_SIZE: usize = <B as Bus>::BLOCK_SIZE;
            }
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            const _: () = {};
            mod sealed { pub trait Bus {} }
            #real_comment pub struct Real<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Real<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B> where {}
            impl<B: Bus> ::tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
        }
    };
    assert_tokens_eq(generate(input), expected);
}
