// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! These test cases verify that:
//!
//! 1. Doc comments that should be copied from the input are copied correctly.
//! 2. Generated doc comments are correct.

use crate::{generate, test_util::assert_tokens_eq};
use quote::quote;
use syn::parse_quote;

#[test]
fn doc_comments() {
    let input = parse_quote! {
        ::tock_registers
        //! Doc comment A
        //! Doc comment B
        #![buses(Mmio32, Mmio64)]
        //! Doc comment C
        //! Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub foo {
            /// Doc comment G
            /// Doc comment H
            0 => scalar_definition: u8 { Read, Write },
            /// Doc comment I
            /// Doc comment J
            1 => array_definition: [[u8; 2]; 3] { Read, Write },
            /// Doc comment K
            /// Doc comment L
            7 => scalar_reference: a,
            /// Doc comment M
            /// Doc comment N
            8 => array_reference: [[b; 2]; 3],
        }
    };
    let expected = quote! {
        /// Doc comment A
        /// Doc comment B
        /// Doc comment C
        /// Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub mod foo {
            #![allow(clippy::expl_impl_clone_on_copy)]
            #![allow(non_camel_case_types)]
            use super::*;
            /// Trait representing this register block. Driver code can use this trait to work with
            /// both real hardware and fake implementations of the peripheral (for unit testing).
            pub trait Interface: ::tock_registers::internal::core::marker::Copy {
                type scalar_definition: ::tock_registers::Register<DataType = u8> + Read + Write;
                /// Doc comment G
                /// Doc comment H
                fn scalar_definition(self) -> Self::scalar_definition;
                type array_definition: ::tock_registers::RegisterArray<3, Element:
                    ::tock_registers::RegisterArray<2, Element:
                        ::tock_registers::Register<DataType = u8> + Read + Write> >;
                /// Doc comment I
                /// Doc comment J
                fn array_definition(self) -> Self::array_definition;
                type scalar_reference: a::Interface;
                /// Doc comment K
                /// Doc comment L
                fn scalar_reference(self) -> Self::scalar_reference;
                type array_reference: ::tock_registers::RegisterArray<3, Element:
                    ::tock_registers::RegisterArray<2, Element: b::Interface> >;
                /// Doc comment M
                /// Doc comment N
                fn array_reference(self) -> Self::array_reference;
            }
            #[allow(non_upper_case_globals)]
            /// Buses supported by this register block.
            pub trait Bus: ::tock_registers::Address + ::tock_registers::DataTypeBus<u8> +
                ::tock_registers::DataTypeBus<u8> + a::Bus + b::Bus + sealed::Bus
            {
                const BLOCK_SIZE: usize;
                const scalar_definition_offset: usize = 0;
                const array_definition_offset: usize = 1;
                const scalar_reference_offset: usize = 7;
                const array_reference_offset: usize = 8;
            }
            impl Bus for Mmio32 {
                const BLOCK_SIZE: usize = 8 +
                    <<Real<Mmio32> as Interface>::array_reference as ::tock_registers::Block>::SIZE;
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const BLOCK_SIZE: usize = 8 +
                    <<Real<Mmio64> as Interface>::array_reference as ::tock_registers::Block>::SIZE;
            }
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {
                const BLOCK_SIZE: usize = <B as Bus>::BLOCK_SIZE;
            }
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            #[allow(clippy::eq_op)] const _: () = {
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
                assert!(8 == 7 + <<Real<Mmio32> as Interface>::scalar_reference as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio32");
                assert!(8 == 7 + <<Real<Mmio64> as Interface>::scalar_reference as
                    ::tock_registers::Block>::SIZE, "offset mismatch for bus Mmio64");
            };
            mod sealed { pub trait Bus {} }
            /// Struct implementing [Interface] for use with the real hardware.
            pub struct Real<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                /// Constructs an accessor for this register or register block.
                /// # Safety
                /// 1. `address` must point to register(s) on the bus corresponding to
                ///    `B`.
                /// 2. The register(s)' definition (as provided to the
                ///    `tock_registers::registers!` macro) must correctly describe the
                ///    pointed-to register(s).
                /// 3. The returned register accessor must not be used in a way that
                ///    causes data races. The exact requirements depend on the hardware,
                ///    but it's usually best to access a register from only one thread
                ///    at a time.
                pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Real<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B>
            where
                real_scalar_definition<B>: ::tock_registers::Register<DataType = u8> + Read + Write,
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
            #[doc =
                "Struct that provides access to the `scalar_definition` register on real hardware."]
            pub struct real_scalar_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_scalar_definition<B> {
                /// Constructs an accessor for this register or register block.
                /// # Safety
                /// 1. `address` must point to register(s) on the bus corresponding to
                ///    `B`.
                /// 2. The register(s)' definition (as provided to the
                ///    `tock_registers::registers!` macro) must correctly describe the
                ///    pointed-to register(s).
                /// 3. The returned register accessor must not be used in a way that
                ///    causes data races. The exact requirements depend on the hardware,
                ///    but it's usually best to access a register from only one thread
                ///    at a time.
                pub const unsafe fn new(address: B) -> Self {
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
            Write!(real_impl, real_scalar_definition, u8,,);
            #[doc =
                "Struct that provides access to the `array_definition` register on real hardware."]
            pub struct real_array_definition<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_array_definition<B> {
                /// Constructs an accessor for this register or register block.
                /// # Safety
                /// 1. `address` must point to register(s) on the bus corresponding to
                ///    `B`.
                /// 2. The register(s)' definition (as provided to the
                ///    `tock_registers::registers!` macro) must correctly describe the
                ///    pointed-to register(s).
                /// 3. The returned register accessor must not be used in a way that
                ///    causes data races. The exact requirements depend on the hardware,
                ///    but it's usually best to access a register from only one thread
                ///    at a time.
                pub const unsafe fn new(address: B) -> Self {
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
