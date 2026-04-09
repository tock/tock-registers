// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::block::{
    bus_doc_comment, field_struct_doc_comment, interface_doc_comment, real_doc_comment,
};
use crate::{generate, new_doc_comment, test_util::assert_tokens_eq};
use quote::quote;
use syn::parse_quote;

/// This serves two purposes: it tests the generated code, and also serves as a walkthrough of the
/// generated code.
#[test]
fn all_field_types_example() {
    let input = parse_quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {
            0 => scalar_definition: u8 { Read, Dance<Waltz> },
            1 => array_definition: [[u8; 2]; 3] { Read, Write },
            7 => _: 1, // Padding
            8 => scalar_reference: a,
            9 => array_reference: [[b; 2]; 3],
        }
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_comment = real_doc_comment();
    let new_comment = new_doc_comment();
    let scalar_definition_comment = field_struct_doc_comment(&parse_quote![scalar_definition]);
    let array_definition_comment = field_struct_doc_comment(&parse_quote![array_definition]);
    let expected = quote! {
        pub mod foo {
            #![allow(clippy::expl_impl_clone_on_copy)]
            #![allow(non_camel_case_types)]
            use super::*;
            #interface_comment pub trait Interface: ::tock_registers::internal::core::marker::Copy {
                type scalar_definition:
                    ::tock_registers::Register<DataType = u8> + Read + Dance<Waltz>;
                fn scalar_definition(self) -> Self::scalar_definition;
                type array_definition: ::tock_registers::RegisterArray<3, Element:
                    ::tock_registers::RegisterArray<2, Element:
                        ::tock_registers::Register<DataType = u8> + Read + Write> >;
                fn array_definition(self) -> Self::array_definition;
                type scalar_reference: a::Interface;
                fn scalar_reference(self) -> Self::scalar_reference;
                type array_reference: ::tock_registers::RegisterArray<3, Element:
                    ::tock_registers::RegisterArray<2, Element: b::Interface> >;
                fn array_reference(self) -> Self::array_reference;
            }
            #[allow(non_upper_case_globals)]
            #bus_comment
            pub trait Bus: ::tock_registers::Address + ::tock_registers::DataTypeBus<u8> +
                ::tock_registers::DataTypeBus<u8> + a::Bus + b::Bus + sealed::Bus
            {
                const BLOCK_SIZE: usize;
                const scalar_definition_offset: usize = 0;
                const array_definition_offset: usize = 1;
                const scalar_reference_offset: usize = 8;
                const array_reference_offset: usize = 9;
            }
            impl Bus for Mmio32 {
                const BLOCK_SIZE: usize = 9 +
                    <<Real<Mmio32> as Interface>::array_reference as ::tock_registers::Block>::SIZE;
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const BLOCK_SIZE: usize = 9 +
                    <<Real<Mmio64> as Interface>::array_reference as ::tock_registers::Block>::SIZE;
            }
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {
                const BLOCK_SIZE: usize = <B as Bus>::BLOCK_SIZE;
            }
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            const _: () = {
                assert!(0 == 0, "offset mismatch for bus Mmio32");
                assert!(0 == 0, "offset mismatch for bus Mmio64");
                assert!(1 == 0 + <<Real<Mmio32> as Interface>::scalar_definition as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio32");
                assert!(1 == 0 + <<Real<Mmio64> as Interface>::scalar_definition as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio64");
                assert!(7 == 1 + <<Real<Mmio32> as Interface>::array_definition as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio32");
                assert!(7 == 1 + <<Real<Mmio64> as Interface>::array_definition as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio64");
                assert!(8 == 7 + 1, "offset mismatch for bus Mmio32");
                assert!(8 == 7 + 1, "offset mismatch for bus Mmio64");
                assert!(9 == 8 + <<Real<Mmio32> as Interface>::scalar_reference as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio32");
                assert!(9 == 8 + <<Real<Mmio64> as Interface>::scalar_reference as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio64");
            };
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
            impl<B: Bus> Interface for Real<B>
            where
                real_scalar_definition<B>:
                    ::tock_registers::Register<DataType = u8> + Read + Dance<Waltz>,
                real_array_definition<B>: ::tock_registers::Register<DataType = u8> + Read + Write,
                a::Real<B>: a::Interface,
                b::Real<B>: b::Interface,
            {
                type scalar_definition = real_scalar_definition<B>;
                fn scalar_definition(self) -> Self::scalar_definition {
                    unsafe {
                        Self::scalar_definition::new(
                            self.address.byte_add(<B as Bus>::scalar_definition_offset))
                    }
                }
                type array_definition = ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<real_array_definition<B>, 2>, 3>;
                fn array_definition(self) -> Self::array_definition {
                    unsafe {
                        Self::array_definition::new(
                            self.address.byte_add(<B as Bus>::array_definition_offset))
                    }
                }
                type scalar_reference = a::Real<B>;
                fn scalar_reference(self) -> Self::scalar_reference {
                    unsafe {
                        Self::scalar_reference::new(
                            self.address.byte_add(<B as Bus>::scalar_reference_offset))
                    }
                }
                type array_reference = ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<b::Real<B>, 2>, 3>;
                fn array_reference(self) -> Self::array_reference {
                    unsafe {
                        Self::array_reference::new(
                            self.address.byte_add(<B as Bus>::array_reference_offset))
                    }
                }
            }
            impl<B: Bus> ::tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
            #scalar_definition_comment pub struct real_scalar_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_scalar_definition<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_scalar_definition<B> { #[inline] fn clone(&self) -> Self { *self } }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_scalar_definition<B> {}
            impl<B: Bus> ::tock_registers::Block for real_scalar_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_scalar_definition<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_scalar_definition<B> {
                type DataType = u8;
            }
            Read!(real_impl, real_scalar_definition, u8,,);
            // Since macros cannot accept generic arguments, the generics are instead detached from
            // the operation path and moved into an argument of the macro invocation.
            Dance!(real_impl, real_scalar_definition, u8, <Waltz>,);
            #array_definition_comment pub struct real_array_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_array_definition<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_array_definition<B> { #[inline] fn clone(&self) -> Self { *self } }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_array_definition<B> {}
            impl<B: Bus> ::tock_registers::Block for real_array_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_array_definition<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_array_definition<B> {
                type DataType = u8;
            }
            Read!(real_impl, real_array_definition, u8,,);
            Write!(real_impl, real_array_definition, u8,,);
        }
    };
    assert_tokens_eq(generate(input), expected);
}
