// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::block::{
    bus_doc_comment, field_struct_doc_comment, interface_doc_comment, real_doc_comment,
};
use crate::{new_doc_comment, register_map, test_util::assert_tokens_eq, Env::ProcMacro};
use quote::quote;
use syn::parse_quote;

/// This serves two purposes: it tests the generated code, and also serves as a walkthrough of the
/// generated code.
#[test]
fn all_field_types_example() {
    let input = quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {
            0 => scalar_definition: u8 { Read, Dance<Waltz> },
            1 => array_definition: [[u8; 2]; 3] { Read, Write },
            7 => _: 1, // Padding
            8 => scalar_reference: a,
            9 => array_reference: [[b; 2]; 3],
            15 => flat_array_definition: [u8; 2] { Read },
            17 => flat_array_reference: [c; 2],
        }
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_comment = real_doc_comment();
    let new_comment = new_doc_comment();
    let scalar_definition_comment = field_struct_doc_comment(&parse_quote![scalar_definition]);
    let array_definition_comment = field_struct_doc_comment(&parse_quote![array_definition]);
    let flat_array_definition_comment =
        field_struct_doc_comment(&parse_quote![flat_array_definition]);
    let expected = quote! {
        pub mod foo {
            #![allow(non_camel_case_types)]
            use super::*;
            #interface_comment pub trait Interface: ::tock_registers::internal::core::marker::Copy {
                type scalar_definition:
                    ::tock_registers::Register<DataType = u8> + Read + Dance<Waltz>;
                fn scalar_definition(self) -> Self::scalar_definition;
                type array_definition: ::tock_registers::RegisterArray<
                    lens::array_definition<1usize>,
                    Element: ::tock_registers::RegisterArray<lens::array_definition<0usize>,
                        Element: ::tock_registers::Register<DataType = u8> + Read + Write> >;
                fn array_definition(self) -> Self::array_definition;
                type scalar_reference: a::Interface;
                fn scalar_reference(self) -> Self::scalar_reference;
                type array_reference: ::tock_registers::RegisterArray<
                    lens::array_reference<1usize>, Element: ::tock_registers::RegisterArray<
                        lens::array_reference<0usize>, Element: b::Interface> >;
                fn array_reference(self) -> Self::array_reference;
                type flat_array_definition: ::tock_registers::RegisterArray<
                    lens::flat_array_definition, Element:
                        ::tock_registers::Register<DataType = u8> + Read>;
                fn flat_array_definition(self) -> Self::flat_array_definition;
                type flat_array_reference: ::tock_registers::RegisterArray<
                    lens::flat_array_reference, Element: c::Interface>;
                fn flat_array_reference(self) -> Self::flat_array_reference;
            }
            pub mod lens {
                pub enum array_definition<const N: usize> {}
                impl ::tock_registers::array::Len for array_definition<0usize> { const LEN: usize = 2; }
                impl ::tock_registers::array::Len for array_definition<1usize> { const LEN: usize = 3; }
                pub enum array_reference<const N: usize> {}
                impl ::tock_registers::array::Len for array_reference<0usize> { const LEN: usize = 2; }
                impl ::tock_registers::array::Len for array_reference<1usize> { const LEN: usize = 3; }
                pub enum flat_array_definition {}
                impl ::tock_registers::array::Len for flat_array_definition { const LEN: usize = 2; }
                pub enum flat_array_reference {}
                impl ::tock_registers::array::Len for flat_array_reference { const LEN: usize = 2; }
            }
            // We allow trait_duplication_in_bounds. For this macro to not emit duplicate bounds,
            // it would have to parse the trait references and figure out if they are identical,
            // which we don't want to do (we want to treat the paths as opaque to limit complexity
            // and for future-proofing).
            #bus_comment #[allow(clippy::trait_duplication_in_bounds)]
            pub trait Bus: ::tock_registers::Address + ::tock_registers::DataTypeBus<u8> +
                ::tock_registers::DataTypeBus<u8> + a::Bus + b::Bus +
                ::tock_registers::DataTypeBus<u8> + c::Bus + sealed::Bus
            {
                const SIZE: usize;
                const scalar_definition_offset: usize = 0;
                const array_definition_offset: usize = 1;
                const scalar_reference_offset: usize = 8;
                const array_reference_offset: usize = 9;
                const flat_array_definition_offset: usize = 15;
                const flat_array_reference_offset: usize = 17;
            }
            impl Bus for Mmio32 {
                const SIZE: usize = 17 + <<Real<Mmio32> as Interface>::flat_array_reference
                    as ::tock_registers::Span>::SIZE;
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const SIZE: usize = 17 + <<Real<Mmio64> as Interface>::flat_array_reference
                    as ::tock_registers::Span>::SIZE;
            }
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {
                const SIZE: usize = <B as Bus>::SIZE;
            }
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            const _: () = {
                // The call to identity() in each assert! prevents the clippy::eq_op lint from
                // triggering.
                assert!(0 == ::tock_registers::internal::core::convert::identity(0),
                    "offset mismatch for bus Mmio32");
                assert!(0 == ::tock_registers::internal::core::convert::identity(0),
                    "offset mismatch for bus Mmio64");
                assert!(1 == ::tock_registers::internal::core::convert::identity(0 + <<Real<Mmio32>
                    as Interface>::scalar_definition as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(1 == ::tock_registers::internal::core::convert::identity(0 + <<Real<Mmio64>
                    as Interface>::scalar_definition as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(7 == ::tock_registers::internal::core::convert::identity(1 + <<Real<Mmio32>
                    as Interface>::array_definition as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(7 == ::tock_registers::internal::core::convert::identity(1 + <<Real<Mmio64>
                    as Interface>::array_definition as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(8 == ::tock_registers::internal::core::convert::identity(7 + 1),
                    "offset mismatch for bus Mmio32");
                assert!(8 == ::tock_registers::internal::core::convert::identity(7 + 1),
                    "offset mismatch for bus Mmio64");
                assert!(9 == ::tock_registers::internal::core::convert::identity(8 + <<Real<Mmio32>
                    as Interface>::scalar_reference as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(9 == ::tock_registers::internal::core::convert::identity(8 + <<Real<Mmio64>
                    as Interface>::scalar_reference as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(15 == ::tock_registers::internal::core::convert::identity(9 +
                    <<Real<Mmio32> as Interface>::array_reference as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(15 == ::tock_registers::internal::core::convert::identity(9 +
                    <<Real<Mmio64> as Interface>::array_reference as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(17 == ::tock_registers::internal::core::convert::identity(15 +
                    <<Real<Mmio32> as Interface>::flat_array_definition
                    as ::tock_registers::Span>::SIZE), "offset mismatch for bus Mmio32");
                assert!(17 == ::tock_registers::internal::core::convert::identity(15 +
                    <<Real<Mmio64> as Interface>::flat_array_definition
                    as ::tock_registers::Span>::SIZE), "offset mismatch for bus Mmio64");
            };
            mod sealed { pub trait Bus {} }
            #real_comment #[derive(Clone)] pub struct Real<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B>
            where
                real_scalar_definition<B>:
                    ::tock_registers::Register<DataType = u8> + Read + Dance<Waltz>,
                real_array_definition<B>: ::tock_registers::Register<DataType = u8> + Read + Write,
                a::Real<B>: a::Interface,
                b::Real<B>: b::Interface,
                real_flat_array_definition<B>: ::tock_registers::Register<DataType = u8> + Read,
                c::Real<B>: c::Interface,
            {
                type scalar_definition = real_scalar_definition<B>;
                fn scalar_definition(self) -> Self::scalar_definition {
                    unsafe {
                        Self::scalar_definition::new(
                            self.address.byte_add(<B as Bus>::scalar_definition_offset))
                    }
                }
                type array_definition = ::tock_registers::RealRegisterArray<
                    ::tock_registers::RealRegisterArray<real_array_definition<B>,
                        lens::array_definition<0usize> >, lens::array_definition<1usize> >;
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
                    ::tock_registers::RealRegisterArray<b::Real<B>, lens::array_reference<0usize>
                    >, lens::array_reference<1usize> >;
                fn array_reference(self) -> Self::array_reference {
                    unsafe {
                        Self::array_reference::new(
                            self.address.byte_add(<B as Bus>::array_reference_offset))
                    }
                }
                type flat_array_definition = ::tock_registers::RealRegisterArray<
                    real_flat_array_definition<B>, lens::flat_array_definition>;
                fn flat_array_definition(self) -> Self::flat_array_definition {
                    unsafe {
                        Self::flat_array_definition::new(
                            self.address.byte_add(<B as Bus>::flat_array_definition_offset))
                    }
                }
                type flat_array_reference =
                    ::tock_registers::RealRegisterArray<c::Real<B>, lens::flat_array_reference>;
                fn flat_array_reference(self) -> Self::flat_array_reference {
                    unsafe {
                        Self::flat_array_reference::new(
                            self.address.byte_add(<B as Bus>::flat_array_reference_offset))
                    }
                }
            }
            impl<B: Bus> ::tock_registers::Span for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
            #scalar_definition_comment #[derive(Clone)] pub struct real_scalar_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_scalar_definition<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_scalar_definition<B> {}
            impl<B: Bus> ::tock_registers::Span for real_scalar_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
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
            #array_definition_comment #[derive(Clone)] pub struct real_array_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_array_definition<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_array_definition<B> {}
            impl<B: Bus> ::tock_registers::Span for real_array_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_array_definition<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_array_definition<B> {
                type DataType = u8;
            }
            Read!(real_impl, real_array_definition, u8,,);
            Write!(real_impl, real_array_definition, u8,,);
            #flat_array_definition_comment #[derive(Clone)]
            pub struct real_flat_array_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_flat_array_definition<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_flat_array_definition<B> {}
            impl<B: Bus> ::tock_registers::Span for real_flat_array_definition<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> =
                    real_flat_array_definition<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_flat_array_definition<B> {
                type DataType = u8;
            }
            Read!(real_impl, real_flat_array_definition, u8,,);
        }
    };
    assert_tokens_eq(register_map(input, ProcMacro).unwrap(), expected);
}
