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

#[test]
fn offsets() {
    let input = quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {
            0 => variable_size: usize { Read },
            [4, 8] => size_variable_pos: u32 { Read },
            #[aliased]
            6 => aliased: u16 { Read },
            [8, 12] => _: [4, 0],
            12 => fixed_pos: u32 { Read },
            16 => _,
            [20, 24] => padded_pos: u8 { Read },
            // This pattern of inferred-size padding followed by explicit-size padding is weird and
            // probably won't be used in practice, but it allows for the total size of the block to
            // be specified without having to specify the size of the trailing padding.
            [21, 25] => _,
            [24, 32] => _: 0,
        }
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_comment = real_doc_comment();
    let new_comment = new_doc_comment();
    let variable_size_comment = field_struct_doc_comment(&parse_quote![variable_size]);
    let size_variable_pos_comment = field_struct_doc_comment(&parse_quote![size_variable_pos]);
    let aliased_comment = field_struct_doc_comment(&parse_quote![aliased]);
    let fixed_pos_comment = field_struct_doc_comment(&parse_quote![fixed_pos]);
    let padded_pos_comment = field_struct_doc_comment(&parse_quote![padded_pos]);
    let expected = quote! {
        pub mod foo {
            #![allow(non_camel_case_types)]
            use super::*;
            #interface_comment pub trait Interface: ::tock_registers::internal::core::marker::Copy {
                type variable_size: ::tock_registers::Register<DataType = usize> + Read;
                fn variable_size(self) -> Self::variable_size;
                type size_variable_pos: ::tock_registers::Register<DataType = u32> + Read;
                fn size_variable_pos(self) -> Self::size_variable_pos;
                type aliased: ::tock_registers::Register<DataType = u16> + Read;
                fn aliased(self) -> Self::aliased;
                type fixed_pos: ::tock_registers::Register<DataType = u32> + Read;
                fn fixed_pos(self) -> Self::fixed_pos;
                type padded_pos: ::tock_registers::Register<DataType = u8> + Read;
                fn padded_pos(self) -> Self::padded_pos;
            }
            pub mod lens {}
            #bus_comment #[allow(clippy::trait_duplication_in_bounds)]
            pub trait Bus: ::tock_registers::Address + ::tock_registers::DataTypeBus<usize> +
                ::tock_registers::DataTypeBus<u32> + ::tock_registers::DataTypeBus<u16> +
                ::tock_registers::DataTypeBus<u32> + ::tock_registers::DataTypeBus<u8> + sealed::Bus
            {
                const SIZE: usize;
                const variable_size_offset: usize = 0;
                const size_variable_pos_offset: usize;
                const aliased_offset: usize = 6;
                const fixed_pos_offset: usize = 12;
                const padded_pos_offset: usize;
            }
            impl Bus for Mmio32 {
                const SIZE: usize = 24 + 0;
                const size_variable_pos_offset: usize = 4;
                const padded_pos_offset: usize = 20;
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const SIZE: usize = 32 + 0;
                const size_variable_pos_offset: usize = 8;
                const padded_pos_offset: usize = 24;
            }
            impl sealed::Bus for Mmio64 {}
            impl<B: Bus> Bus for ::tock_registers::BorrowedBus<'_, B> {
                const SIZE: usize = <B as Bus>::SIZE;
                const size_variable_pos_offset: usize = <B as Bus>::size_variable_pos_offset;
                const padded_pos_offset: usize = <B as Bus>::padded_pos_offset;
            }
            impl<B: Bus> sealed::Bus for ::tock_registers::BorrowedBus<'_, B> {}
            const _: () = {
                assert!(0 == ::tock_registers::internal::core::convert::identity(0),
                    "offset mismatch for bus Mmio32");
                assert!(0 == ::tock_registers::internal::core::convert::identity(0),
                    "offset mismatch for bus Mmio64");
                assert!(4 == ::tock_registers::internal::core::convert::identity(0 +
                    <<Real<Mmio32> as Interface>::variable_size as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(8 == ::tock_registers::internal::core::convert::identity(0 +
                    <<Real<Mmio64> as Interface>::variable_size as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(8 == ::tock_registers::internal::core::convert::identity(4 + <<Real<Mmio32>
                    as Interface>::size_variable_pos as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(12 == ::tock_registers::internal::core::convert::identity(8 + <<Real<Mmio64>
                    as Interface>::size_variable_pos as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(12 == ::tock_registers::internal::core::convert::identity(8 + 4),
                    "offset mismatch for bus Mmio32");
                assert!(12 == ::tock_registers::internal::core::convert::identity(12 + 0),
                    "offset mismatch for bus Mmio64");
                assert!(16 == ::tock_registers::internal::core::convert::identity(12 +
                    <<Real<Mmio32> as Interface>::fixed_pos as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(16 == ::tock_registers::internal::core::convert::identity(12 +
                    <<Real<Mmio64> as Interface>::fixed_pos as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
                assert!(21 == ::tock_registers::internal::core::convert::identity(20 +
                    <<Real<Mmio32> as Interface>::padded_pos as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio32");
                assert!(25 == ::tock_registers::internal::core::convert::identity(24 +
                    <<Real<Mmio64> as Interface>::padded_pos as ::tock_registers::Span>::SIZE),
                    "offset mismatch for bus Mmio64");
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
                real_variable_size<B>: ::tock_registers::Register<DataType = usize> + Read,
                real_size_variable_pos<B>: ::tock_registers::Register<DataType = u32> + Read,
                real_aliased<B>: ::tock_registers::Register<DataType = u16> + Read,
                real_fixed_pos<B>: ::tock_registers::Register<DataType = u32> + Read,
                real_padded_pos<B>: ::tock_registers::Register<DataType = u8> + Read,
            {
                type variable_size = real_variable_size<B>;
                fn variable_size(self) -> Self::variable_size {
                    unsafe {
                        Self::variable_size::new(
                            self.address.byte_add(<B as Bus>::variable_size_offset))
                    }
                }
                type size_variable_pos = real_size_variable_pos<B>;
                fn size_variable_pos(self) -> Self::size_variable_pos {
                    unsafe {
                        Self::size_variable_pos::new(
                            self.address.byte_add(<B as Bus>::size_variable_pos_offset))
                    }
                }
                type aliased = real_aliased<B>;
                fn aliased(self) -> Self::aliased {
                    unsafe { Self::aliased::new(self.address.byte_add(<B as Bus>::aliased_offset)) }
                }
                type fixed_pos = real_fixed_pos<B>;
                fn fixed_pos(self) -> Self::fixed_pos {
                    unsafe {
                        Self::fixed_pos::new(self.address.byte_add(<B as Bus>::fixed_pos_offset))
                    }
                }
                type padded_pos = real_padded_pos<B>;
                fn padded_pos(self) -> Self::padded_pos {
                    unsafe {
                        Self::padded_pos::new(self.address.byte_add(<B as Bus>::padded_pos_offset))
                    }
                }
            }
            unsafe impl<B: Bus> ::tock_registers::Span for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = Real<::tock_registers::BorrowedBus<'b, B>>;
            }
            #variable_size_comment #[derive(Clone)] pub struct real_variable_size<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_variable_size<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for real_variable_size<B> {}
            unsafe impl<B: Bus> ::tock_registers::Span for real_variable_size<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<usize>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_variable_size<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_variable_size<B> {
                type DataType = usize;
            }
            Read!(real_impl, real_variable_size, usize,,);
            #size_variable_pos_comment #[derive(Clone)] pub struct real_size_variable_pos<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_size_variable_pos<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for real_size_variable_pos<B> {}
            unsafe impl<B: Bus> ::tock_registers::Span for real_size_variable_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u32>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_size_variable_pos<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_size_variable_pos<B> {
                type DataType = u32;
            }
            Read!(real_impl, real_size_variable_pos, u32,,);
            #aliased_comment #[derive(Clone)] pub struct real_aliased<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_aliased<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for real_aliased<B> {}
            unsafe impl<B: Bus> ::tock_registers::Span for real_aliased<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u16>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_aliased<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_aliased<B> { type DataType = u16; }
            Read!(real_impl, real_aliased, u16,,);
            #fixed_pos_comment #[derive(Clone)] pub struct real_fixed_pos<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_fixed_pos<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_fixed_pos<B> {}
            unsafe impl<B: Bus> ::tock_registers::Span for real_fixed_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u32>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_fixed_pos<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_fixed_pos<B> {
                type DataType = u32;
            }
            Read!(real_impl, real_fixed_pos, u32,,);
            #padded_pos_comment #[derive(Clone)] pub struct real_padded_pos<B: Bus> {
                address: B,
                _phantom: ::tock_registers::internal::RealPhantom,
            }
            impl<B: Bus> real_padded_pos<B> {
                #new_comment pub const unsafe fn new(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_padded_pos<B> {}
            unsafe impl<B: Bus> ::tock_registers::Span for real_padded_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u8>>::PADDED_SIZE;
                unsafe fn with_addr(address: B) -> Self {
                    Self { address, _phantom: ::tock_registers::internal::RealPhantom::new() }
                }
                type Borrowed<'b> = real_padded_pos<::tock_registers::BorrowedBus<'b, B>>;
            }
            impl<B: Bus> ::tock_registers::Register for real_padded_pos<B> {
                type DataType = u8;
            }
            Read!(real_impl, real_padded_pos, u8,,);
        }
    };
    assert_tokens_eq(register_map(input, ProcMacro).unwrap(), expected);
}
