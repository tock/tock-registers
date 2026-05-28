// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::block::{bus_doc_comment, interface_doc_comment, real_doc_comment};
use crate::{new_doc_comment, register_map, test_util::assert_tokens_eq, Env::External};
use quote::quote;

// This also doubles as the test for Env::External code generation.
#[test]
fn empty() {
    let input = quote! {
        ::tock_registers
        #[bus(Mmio32)]
        pub foo {}
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_comment = real_doc_comment();
    let new_comment = new_doc_comment();
    let expected = quote! {
        pub mod foo {
            #![allow(non_camel_case_types,dead_code,non_upper_case_globals)] use super::*;
            #interface_comment
            pub trait Interface: ::tock_registers::internal::core::marker::Copy {}
            pub mod lens {}
            #bus_comment #[allow(clippy::trait_duplication_in_bounds)]
            pub trait Bus: ::tock_registers::Address + sealed::Bus {
                const SIZE: usize;
            }
            impl Bus for Mmio32 { const SIZE: usize = 0; }
            impl sealed::Bus for Mmio32 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {
                const SIZE: usize = <B as Bus>::SIZE;
            }
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            const _: () = {};
            mod sealed { pub trait Bus {} }
            #real_comment #[derive(Clone)] pub struct Real<B: Bus = Mmio32> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B> where {}
            impl<B: Bus> ::tock_registers::Span for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
        }
    };
    assert_tokens_eq(register_map(input, External).unwrap(), expected);
}
