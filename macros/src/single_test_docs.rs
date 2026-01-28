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
fn scalar_definition() {
    let input = parse_quote! {
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
            #![allow(clippy::expl_impl_clone_on_copy)]
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
            mod sealed { pub trait Bus {} }
            /// Struct that implements [Interface] for use with the real hardware.
            pub struct Real<B: Bus>(B);
            impl<B: Bus> Real<B> {
                pub unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Real<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> ::tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::Register for Real<B> { type DataType = u8; }
            Read!(real_impl, Real, u8,);
            Write!(real_impl, Real, u8,);
            impl<B: Bus> Interface for Real<B> where
                Self: ::tock_registers::Register<DataType = u8> + Read + Write {}
        }
    };
    assert_tokens_eq(generate(input), expected);
}

#[test]
fn array_definition() {
    let input = parse_quote! {
        ::tock_registers
        //! Doc comment A
        //! Doc comment B
        #![buses(Mmio32, Mmio64)]
        //! Doc comment C
        //! Doc comment D
        /// Doc comment E
        /// Doc comment F
        pub foo: [[u8; 2]; 3] { Read, Write }
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
            use super::*;
            /// Trait representing this register's operations. Driver code can use this trait to work
            /// with both real hardware and fake implementations of the register (for unit testing).
            pub trait Interface: ::tock_registers::RegisterArray<
                Element: ::tock_registers::RegisterArray<
                    Element: ::tock_registers::Register<DataType = u8> + Read + Write
                >
            > {}
            /// Buses supported by this register.
            pub trait Bus: ::tock_registers::DataTypeBus<u8> + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            mod sealed { pub trait Bus {} }
            /// Implementation of an element of this register array for use with real hardware.
            /// This implements the tock_registers::Register trait as well as any operation traits
            /// specified in the register definition.
            pub struct Element<B: Bus>(B);
            impl<B: Bus> Element<B> {
                pub unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Element<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Element<B> {}
            impl<B: Bus> ::tock_registers::Block for Element<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::Register for Element<B> { type DataType = u8; }
            Read!(real_impl, Element, u8,);
            Write!(real_impl, Element, u8,);
            /// Implementation of [Interface] for use with real hardware.
            pub type Real<B> = ::tock_registers::RealRegisterArray<
                ::tock_registers::RealRegisterArray<Element<B>, 2>, 3
            >;
            impl<B: Bus> Interface for Real<B> where
                Element<B>: ::tock_registers::Register<DataType = u8> + Read + Write {}
        }
    };
    assert_tokens_eq(generate(input), expected);
}

#[test]
fn scalar_reference() {
    let input = parse_quote! {
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
            mod sealed { pub trait Bus {} }
            /// Implementation of [Interface] for use with real hardware.
            pub type Real<B> = status::Real<B>;
            impl<B: Bus> Interface for Real<B> where Self: status::Interface {}
        }
    };
    assert_tokens_eq(generate(input), expected);
}

#[test]
fn array_reference() {
    let input = parse_quote! {
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
            pub trait Interface: ::tock_registers::RegisterArray<
                Element: ::tock_registers::RegisterArray<Element: status::Interface>
            > {}
            /// Buses supported by this register.
            pub trait Bus: status::Bus + sealed::Bus {}
            impl Bus for Mmio32 {}
            impl Bus for Mmio64 {}
            impl sealed::Bus for Mmio32 {}
            impl sealed::Bus for Mmio64 {}
            mod sealed { pub trait Bus {} }
            /// Implementation of [Interface] for use with real hardware.
            pub type Real<B> = ::tock_registers::RealRegisterArray<
                ::tock_registers::RealRegisterArray<status::Real<B>, 2>,
            3>;
            impl<B: Bus> Interface for Real<B> where status::Real<B>: status::Interface {}
        }
    };
    assert_tokens_eq(generate(input), expected);
}
