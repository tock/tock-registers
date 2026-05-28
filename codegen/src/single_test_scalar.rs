// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::single::{
    bus_doc_comment, interface_doc_comment, real_alias_doc_comment, struct_doc_comment,
};
use crate::{new_doc_comment, register_map, test_util::assert_tokens_eq, Env::External};
use quote::quote;

/// This serves two purposes: it tests the code generation of single scalar register definitions,
/// and also documents (via comments) some of the trickier parts of the generated code.
#[test]
fn scalar_definition_example() {
    let input = quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo: u8 { Read, Write },
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let struct_comment = struct_doc_comment(true);
    let new_comment = new_doc_comment();
    let expected = quote! {
        pub mod foo {
            #![allow(dead_code)] use super::*;
            #interface_comment
            pub trait Interface: ::tock_registers::Register<DataType = u8> + Read + Write {}
            #bus_comment pub trait Bus:
                // Bus needs this bound so that the Span impl for Real<B> can access PADDED_SIZE.
                // It would be ideal to be able to bound BusRead/BusWrite as well, as that would
                // allow us to remove the `where` clause from the `Interface for Real<B>` impl.
                // However, we have to get those trait names from the operations' macros, and you
                // cannot invoke macros from bounds list, so it is difficult to implement that
                // bound. So we only generate the Bus<> bound, as it is the only necessary bound.
                ::tock_registers::DataTypeBus<u8>
                + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            #struct_comment #[derive(Clone)] pub struct Real<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            // We could just use `#[derive(Copy)]` on `Real`, but that generates more-complex code
            // than necessary (it has unnecessary trait bounds). Instead, this macro emits the Copy
            // impl directly.
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> ::tock_registers::Span for Real<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for Real<B> { type DataType = u8; }
            Read!(real_impl, Real, u8,,);
            Write!(real_impl, Real, u8,,);
            impl<B: Bus> Interface for Real<B> where
                // Rust understands that Real<B> always implements Register, but it does not
                // understand that Register::DataType is always u8. Therefore we have to add this
                // bound, to match the bounds on the `Interface` trait definition.
                Self: ::tock_registers::Register<DataType = u8>
                // We have the same issue as Bus here -- Rust doesn't know that every Bus impl is
                // BusRead/BusWrite so it doesn't know that every Real<> is Read/Write. Therefore
                // we have to bound Read + Write here to match Interface's definition.
                + Read + Write {}
        }
    };
    assert_tokens_eq(register_map(input, External).unwrap(), expected);
}

/// This serves two purposes: it tests the code generation of single scalar register references,
/// and also documents (via comments) some of the trickier parts of the generated code.
#[test]
fn scalar_reference_example() {
    let input = quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo: status,
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_alias_comment = real_alias_doc_comment();
    let expected = quote! {
        pub mod foo {
            #![allow(dead_code)] use super::*;
            #interface_comment pub trait Interface: status::Interface {}
            #bus_comment pub trait Bus: status::Bus + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            #real_alias_comment pub type Real<B> = status::Real<B>;
            impl<B: Bus> Interface for Real<B> where
                // Similar to definitions, without this bound Rust does not understand that every
                // Bus implements BusRead/BusWrite as needed.
                Self: status::Interface {}
        }
    };
    assert_tokens_eq(register_map(input, External).unwrap(), expected);
}

/// Verifies that generic arguments on operations are correctly copied to the output (they need to
/// be split off the operation path for the operation macro invocation).
///
/// This also doubles as the test for `#[bus]`.
#[test]
fn generic_operation() {
    let input = quote! {
        ::tock_registers
        #[bus(Mmio32)]
        foo: u8 { Dance<Waltz> },
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let struct_comment = struct_doc_comment(true);
    let new_comment = new_doc_comment();
    let expected = quote! {
        mod foo {
            #![allow(dead_code)] use super::*;
            #interface_comment
            pub trait Interface: ::tock_registers::Register<DataType = u8> + Dance<Waltz> {}
            #bus_comment pub trait Bus: ::tock_registers::DataTypeBus<u8> + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl sealed::Bus for Mmio32 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            #struct_comment #[derive(Clone)] pub struct Real<B: Bus = Mmio32> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> ::tock_registers::Span for Real<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for Real<B> { type DataType = u8; }
            // Since macros cannot accept generic arguments, the generics are instead detached
            // from the operation path and moved into an argument of the macro invocation.
            Dance!(real_impl, Real, u8, <Waltz>,);
            impl<B: Bus> Interface for Real<B> where
                Self: ::tock_registers::Register<DataType = u8> + Dance<Waltz> {}
        }
    };
    assert_tokens_eq(register_map(input, External).unwrap(), expected);
}
