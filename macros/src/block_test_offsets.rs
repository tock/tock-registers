// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::block::{
    bus_doc_comment, field_struct_doc_comment, interface_doc_comment, real_doc_comment,
};
use crate::generate;
use crate::test_util::assert_tokens_eq;
use quote::quote;
use syn::parse_quote;

#[test]
fn offsets() {
    let input = parse_quote! {
        ::tock_registers
        #[buses(Mmio32, Mmio64)]
        pub foo {
            0 => variable_size: usize { Read },
            [4, 8] => variable_pos: u32 { Read },
            [8, 12] => _: [4, 0],
            12 => final_fixed_pos: u32 { Read },
        }
    };
    let interface_comment = interface_doc_comment();
    let bus_comment = bus_doc_comment();
    let real_comment = real_doc_comment();
    let variable_size_comment = field_struct_doc_comment(&parse_quote![variable_size]);
    let variable_pos_comment = field_struct_doc_comment(&parse_quote![variable_pos]);
    let final_fixed_pos_comment = field_struct_doc_comment(&parse_quote![final_fixed_pos]);
    let expected = quote! {
        pub mod foo {
            #![allow(clippy::expl_impl_clone_on_copy)]
            #![allow(non_camel_case_types)]
            use super::*;
            #interface_comment pub trait Interface: ::tock_registers::internal::core::marker::Copy {
                type variable_size: ::tock_registers::Register<DataType = usize> + Read;
                fn variable_size(self) -> Self::variable_size;
                type variable_pos: ::tock_registers::Register<DataType = u32> + Read;
                fn variable_pos(self) -> Self::variable_pos;
                type final_fixed_pos: ::tock_registers::Register<DataType = u32> + Read;
                fn final_fixed_pos(self) -> Self::final_fixed_pos;
            }
            #[allow(non_upper_case_globals)]
            #bus_comment pub trait Bus: ::tock_registers::Address +
                ::tock_registers::DataTypeBus<usize> + ::tock_registers::DataTypeBus<u32> +
                ::tock_registers::DataTypeBus<u32> + sealed::Bus
            {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<4usize>;
                const variable_size_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[0usize];
                const variable_pos_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[1usize];
                const final_fixed_pos_offset: usize = <Self as Bus>::BLOCK_INFO.offsets[3usize];
            }
            impl Bus for Mmio32 {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<4usize> =
                    ::tock_registers::internal::BlockInfo::new([true, true, true, true,],
                        [<Self as ::tock_registers::DataTypeBus<usize> >::PADDED_SIZE,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,
                         4,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,],
                        [&[], &[], &[], &[],]);
            }
            impl sealed::Bus for Mmio32 {}
            impl Bus for Mmio64 {
                const BLOCK_INFO: ::tock_registers::internal::BlockInfo<4usize> =
                    ::tock_registers::internal::BlockInfo::new([true, true, true, true,],
                        [<Self as ::tock_registers::DataTypeBus<usize> >::PADDED_SIZE,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,
                         0,
                         <Self as ::tock_registers::DataTypeBus<u32> >::PADDED_SIZE,],
                        [&[], &[], &[], &[],]);
            }
            impl sealed::Bus for Mmio64 {}
            const _: () = {
                assert!(0 == <Mmio32 as Bus>::variable_size_offset, "offset mismatch");
                assert!(0 == <Mmio64 as Bus>::variable_size_offset, "offset mismatch");
                assert!(4 == <Mmio32 as Bus>::variable_pos_offset, "offset mismatch");
                assert!(8 == <Mmio64 as Bus>::variable_pos_offset, "offset mismatch");
                assert!(8 == <Mmio32 as Bus>::BLOCK_INFO.offsets[2usize], "offset mismatch");
                assert!(12 == <Mmio64 as Bus>::BLOCK_INFO.offsets[2usize], "offset mismatch");
                assert!(12 == <Mmio32 as Bus>::final_fixed_pos_offset, "offset mismatch");
                assert!(12 == <Mmio64 as Bus>::final_fixed_pos_offset, "offset mismatch");
            };
            mod sealed { pub trait Bus {} }
            #real_comment pub struct Real<B: Bus>(B);
            impl<B: Bus> Real<B> {
                pub const unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone for Real<B> {
                #[inline] fn clone(&self) -> Self { *self }
            }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for Real<B> {}
            impl<B: Bus> Interface for Real<B>
            where
                real_variable_size<B>: ::tock_registers::Register<DataType = usize> + Read,
                real_variable_pos<B>: ::tock_registers::Register<DataType = u32> + Read,
                real_final_fixed_pos<B>: ::tock_registers::Register<DataType = u32> + Read,
            {
                type variable_size = real_variable_size<B>;
                fn variable_size(self) -> Self::variable_size {
                    unsafe {
                        Self::variable_size::new(self.0.byte_add(<B as Bus>::variable_size_offset))
                    }
                }
                type variable_pos = real_variable_pos<B>;
                fn variable_pos(self) -> Self::variable_pos {
                    unsafe {
                        Self::variable_pos::new(self.0.byte_add(<B as Bus>::variable_pos_offset))
                    }
                }
                type final_fixed_pos = real_final_fixed_pos<B>;
                fn final_fixed_pos(self) -> Self::final_fixed_pos {
                    unsafe {
                        Self::final_fixed_pos::new(
                            self.0.byte_add(<B as Bus>::final_fixed_pos_offset)
                        )
                    }
                }
            }
            impl<B: Bus> ::tock_registers::Block for Real<B> {
                type Address = B;
                const SIZE: usize = <B as Bus>::BLOCK_INFO.block_size;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            #variable_size_comment pub struct real_variable_size<B: Bus>(B);
            impl<B: Bus> real_variable_size<B> {
                pub unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_variable_size<B> { #[inline] fn clone(&self) -> Self { *self } }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for real_variable_size<B> {}
            impl<B: Bus> ::tock_registers::Block for real_variable_size<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<usize>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::Register for real_variable_size<B> {
                type DataType = usize;
            }
            Read!(real_impl, real_variable_size, usize,);
            #variable_pos_comment pub struct real_variable_pos<B: Bus>(B);
            impl<B: Bus> real_variable_pos<B> {
                pub unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_variable_pos<B> { #[inline] fn clone(&self) -> Self { *self } }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy for real_variable_pos<B> {}
            impl<B: Bus> ::tock_registers::Block for real_variable_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u32>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::Register for real_variable_pos<B> {
                type DataType = u32;
            }
            Read!(real_impl, real_variable_pos, u32,);
            #final_fixed_pos_comment pub struct real_final_fixed_pos<B: Bus>(B);
            impl<B: Bus> real_final_fixed_pos<B> {
                pub unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::internal::core::clone::Clone
            for real_final_fixed_pos<B> { #[inline] fn clone(&self) -> Self { *self } }
            impl<B: Bus> ::tock_registers::internal::core::marker::Copy
            for real_final_fixed_pos<B> {}
            impl<B: Bus> ::tock_registers::Block for real_final_fixed_pos<B> {
                type Address = B;
                const SIZE: usize = <B as ::tock_registers::DataTypeBus<u32>>::PADDED_SIZE;
                unsafe fn new(address: B) -> Self { Self(address) }
            }
            impl<B: Bus> ::tock_registers::Register for real_final_fixed_pos<B> {
                type DataType = u32;
            }
            Read!(real_impl, real_final_fixed_pos, u32,);
        }
    };
    assert_tokens_eq(generate(input), expected);
}
