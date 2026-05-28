// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! These test cases verify that:
//!
//! 1. Doc comments that should be copied from the input are copied correctly.
//! 2. Generated doc comments are correct.

use crate::{register_map, test_util::assert_tokens_eq, Env::ProcMacro};
use quote::quote;

#[test]
fn scalar_definition() {
    let input = quote! {
        ::tock_registers
        //! Doc comment A
        //! Doc comment B
        #![buses(Mmio32, Mmio64)]
        //! Doc comment C
        //! Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub foo: u8 { Read, Write },
    };
    let expected = quote! {
        /// Doc comment A
        /// Doc comment B
        /// Doc comment C
        /// Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub mod foo {
            use super::*;
            /// Trait representing this register's operations. Driver code can use this trait to work
            /// with both real hardware and fake implementations of the register (for unit testing).
            pub trait Interface: ::tock_registers::Register<DataType = u8> + Read + Write {}
            /// Buses supported by this register.
            pub trait Bus: ::tock_registers::DataTypeBus<u8> + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            /// Struct that implements [Interface] for use with the real hardware.
            #[derive(Clone)] pub struct Real<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Real<B> {
                /// Constructs an accessor for this register or register block.
                /// # Safety
                /// 1. `address` must point to register(s) on the bus corresponding to
                ///    `B`.
                /// 2. The register(s)' definition (as provided to the
                ///    `tock_registers::register_map!` macro) must correctly
                ///    describe the pointed-to register(s).
                /// 3. The returned register accessor must not be used in a way that
                ///    causes data races. The exact requirements depend on the hardware,
                ///    but it's usually best to access a register from only one thread
                ///    at a time.
                pub const unsafe fn new(address: B) -> Self {
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
            Read!(real_impl, Real, u8,,);
            Write!(real_impl, Real, u8,,);
            impl<B: Bus> Interface for Real<B> where
                Self: ::tock_registers::Register<DataType = u8> + Read + Write {}
        }
    };
    assert_tokens_eq(register_map(input, ProcMacro).unwrap(), expected);
}

#[test]
fn array_definition() {
    let input = quote! {
        ::tock_registers
        //! Doc comment A
        //! Doc comment B
        #![buses(Mmio32, Mmio64)]
        //! Doc comment C
        //! Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub foo: [u8; 2] { Read, Write }
    };
    let expected = quote! {
        /// Doc comment A
        /// Doc comment B
        /// Doc comment C
        /// Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub mod foo {
            use super::*;
            /// Trait representing this register's operations. Driver code can use this trait to work
            /// with both real hardware and fake implementations of the register (for unit testing).
            pub trait Interface: ::tock_registers::RegisterArray<Len,
                Element: ::tock_registers::Register<DataType = u8> + Read + Write> {}
            pub enum Len {}
            impl ::tock_registers::array::Len for Len { const LEN: usize = 2; }
            /// Buses supported by this register.
            pub trait Bus: ::tock_registers::DataTypeBus<u8> + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            /// Implementation of an element of this register array for use with real hardware.
            /// This implements the tock_registers::Register trait as well as any operation traits
            /// specified in the register definition.
            #[derive(Clone)] pub struct Element<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> Element<B> {
                /// Constructs an accessor for this register or register block.
                /// # Safety
                /// 1. `address` must point to register(s) on the bus corresponding to
                ///    `B`.
                /// 2. The register(s)' definition (as provided to the
                ///    `tock_registers::register_map!` macro) must correctly
                ///    describe the pointed-to register(s).
                /// 3. The returned register accessor must not be used in a way that
                ///    causes data races. The exact requirements depend on the hardware,
                ///    but it's usually best to access a register from only one thread
                ///    at a time.
                pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Element<B> {}
            impl<B: Bus> ::tock_registers::Span for Element<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Element<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for Element<B> { type DataType = u8; }
            Read!(real_impl, Element, u8,,);
            Write!(real_impl, Element, u8,,);
            /// Implementation of [Interface] for use with real hardware.
            pub type Real<B> = ::tock_registers::RealRegisterArray<Element<B>, Len>;
            impl<B: Bus> Interface for Real<B> where
                Element<B>: ::tock_registers::Register<DataType = u8> + Read + Write {}
        }
    };
    assert_tokens_eq(register_map(input, ProcMacro).unwrap(), expected);
}

#[test]
fn scalar_reference() {
    let input = quote! {
        ::tock_registers
        //! Doc comment A
        //! Doc comment B
        #![buses(Mmio32, Mmio64)]
        //! Doc comment C
        //! Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub foo: status,
    };
    let expected = quote! {
        /// Doc comment A
        /// Doc comment B
        /// Doc comment C
        /// Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub mod foo {
            use super::*;
            /// Trait representing this register's operations. Driver code can use this trait to work
            /// with both real hardware and fake implementations of the register (for unit testing).
            pub trait Interface: status::Interface {}
            /// Buses supported by this register.
            pub trait Bus: status::Bus + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            /// Implementation of [Interface] for use with real hardware.
            pub type Real<B> = status::Real<B>;
            impl<B: Bus> Interface for Real<B> where Self: status::Interface {}
        }
    };
    assert_tokens_eq(register_map(input, ProcMacro).unwrap(), expected);
}

#[test]
fn array_reference() {
    let input = quote! {
        ::tock_registers
        //! Doc comment A
        //! Doc comment B
        #![buses(Mmio32, Mmio64)]
        //! Doc comment C
        //! Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub foo: [[status; 2]; 3],
    };
    let expected = quote! {
        /// Doc comment A
        /// Doc comment B
        /// Doc comment C
        /// Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub mod foo {
            use super::*;
            /// Trait representing this register's operations. Driver code can use this trait to work
            /// with both real hardware and fake implementations of the register (for unit testing).
            pub trait Interface: ::tock_registers::RegisterArray<Len<1usize>,
                Element: ::tock_registers::RegisterArray<Len<0usize>, Element: status::Interface>
            > {}
            pub enum Len<const N: usize> {}
            impl ::tock_registers::array::Len for Len<0usize> { const LEN: usize = 2; }
            impl ::tock_registers::array::Len for Len<1usize> { const LEN: usize = 3; }
            /// Buses supported by this register.
            pub trait Bus: status::Bus + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {}
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            mod sealed { pub trait Bus {} }
            /// Implementation of [Interface] for use with real hardware.
            pub type Real<B> = ::tock_registers::RealRegisterArray<
                ::tock_registers::RealRegisterArray<status::Real<B>, Len<0usize> >, Len<1usize> >;
            impl<B: Bus> Interface for Real<B> where status::Real<B>: status::Interface {}
        }
    };
    assert_tokens_eq(register_map(input, ProcMacro).unwrap(), expected);
}
