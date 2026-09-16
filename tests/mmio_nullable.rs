// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! This test verifies that the Mmio32Nullable and Mmio64Nullable buses work
//! correctly. Indirectly, it also verifies that register offsets are calculated
//! correctly. It is designed to run in Miri on 32-bit and 64-bit targets.

#![no_std]

use core::cell::UnsafeCell;
use tock_registers::{register_map, Mmio32Nullable, Mmio64Nullable, Read, RegisterArray, Write};
use {inner_block::Interface as _, outer_block::Interface as _};

register_map! {
    #![buses(Mmio32Nullable, Mmio64Nullable)]
    // External registers to reference.
    a: u8 { Read, Write },
    b: u16 { Read, Write },
    inner_block {
        0 => scalar_definition: usize { Read, Write },
        [4, 8] => _: [4, 8],
        [8, 16] => array_definition: [[u32; 2]; 3] { Read, Write },
        [32, 40] => scalar_reference: a,
        [33, 41] => _,
        [36, 44] => array_reference: [[b; 2]; 3],
    },
    outer_block {
        0 => scalar: u64 { Read, Write },
        #[aliased]
        1 => overlapped: u8 { Read, Write },
        8 => nested: inner_block,
        [56, 64] => nested_array: [inner_block; 2],
        [152, 176] => flat_array: [u8; 2] { Read },
        [154, 178] => flat_array_reference: [a; 2],
    },
}

#[derive(PartialEq)]
#[repr(C)]
struct InnerBlock {
    scalar_definition: usize,
    _padding0: usize,
    array_definition: [[u32; 2]; 3],
    scalar_reference: u8,
    _padding1: [u8; 3],
    array_reference: [[u16; 2]; 3],
}

#[derive(PartialEq)]
#[repr(C)]
struct OuterBlock {
    scalar: u64,
    nested: InnerBlock,
    nested_array: [InnerBlock; 2],
    flat_array: [u8; 2],
    flat_array_reference: [u8; 2],
}

#[cfg(target_pointer_width = "32")]
type Mmio = Mmio32Nullable;
#[cfg(target_pointer_width = "64")]
type Mmio = Mmio64Nullable;

#[test]
fn mmio() {
    let peripheral = UnsafeCell::new(OuterBlock {
        scalar: 1,
        nested: InnerBlock {
            scalar_definition: 2,
            _padding0: 0,
            array_definition: [[3, 4], [5, 6], [7, 8]],
            scalar_reference: 9,
            _padding1: [0; 3],
            array_reference: [[10, 11], [12, 13], [14, 15]],
        },
        nested_array: [
            InnerBlock {
                scalar_definition: 16,
                _padding0: 0,
                array_definition: [[17, 18], [19, 20], [21, 22]],
                scalar_reference: 23,
                _padding1: [0; 3],
                array_reference: [[24, 25], [26, 27], [28, 29]],
            },
            InnerBlock {
                scalar_definition: 30,
                _padding0: 0,
                array_definition: [[31, 32], [33, 34], [35, 36]],
                scalar_reference: 37,
                _padding1: [0; 3],
                array_reference: [[38, 39], [40, 41], [42, 43]],
            },
        ],
        flat_array: [44, 45],
        flat_array_reference: [46, 47],
    });
    let mmio = Mmio::new(peripheral.get().cast());
    let registers = unsafe { outer_block::Real::new(mmio) };
    // Pointer offset validation: verifies that pointer offsets are correctly calculated through
    // the various field types.
    assert_eq!(registers.scalar().get(), 1);
    assert_eq!(registers.overlapped().get(), 0);
    assert_eq!(registers.nested().scalar_definition().get(), 2);
    let array = registers.nested().array_definition();
    assert_eq!(array.get(0).unwrap().get(0).unwrap().get(), 3);
    assert_eq!(array.get(0).unwrap().get(1).unwrap().get(), 4);
    assert_eq!(array.get(1).unwrap().get(0).unwrap().get(), 5);
    assert_eq!(array.get(1).unwrap().get(1).unwrap().get(), 6);
    assert_eq!(array.get(2).unwrap().get(0).unwrap().get(), 7);
    assert_eq!(array.get(2).unwrap().get(1).unwrap().get(), 8);
    assert_eq!(registers.nested().scalar_reference().get(), 9);
    let array = registers.nested().array_reference();
    assert_eq!(array.get(0).unwrap().get(0).unwrap().get(), 10);
    assert_eq!(array.get(0).unwrap().get(1).unwrap().get(), 11);
    assert_eq!(array.get(1).unwrap().get(0).unwrap().get(), 12);
    assert_eq!(array.get(1).unwrap().get(1).unwrap().get(), 13);
    assert_eq!(array.get(2).unwrap().get(0).unwrap().get(), 14);
    assert_eq!(array.get(2).unwrap().get(1).unwrap().get(), 15);
    let nested = registers.nested_array().get(0).unwrap();
    assert_eq!(nested.scalar_definition().get(), 16);
    let array = nested.array_definition();
    assert_eq!(array.get(0).unwrap().get(0).unwrap().get(), 17);
    assert_eq!(array.get(0).unwrap().get(1).unwrap().get(), 18);
    assert_eq!(array.get(1).unwrap().get(0).unwrap().get(), 19);
    assert_eq!(array.get(1).unwrap().get(1).unwrap().get(), 20);
    assert_eq!(array.get(2).unwrap().get(0).unwrap().get(), 21);
    assert_eq!(array.get(2).unwrap().get(1).unwrap().get(), 22);
    assert_eq!(nested.scalar_reference().get(), 23);
    let array = nested.array_reference();
    assert_eq!(array.get(0).unwrap().get(0).unwrap().get(), 24);
    assert_eq!(array.get(0).unwrap().get(1).unwrap().get(), 25);
    assert_eq!(array.get(1).unwrap().get(0).unwrap().get(), 26);
    assert_eq!(array.get(1).unwrap().get(1).unwrap().get(), 27);
    assert_eq!(array.get(2).unwrap().get(0).unwrap().get(), 28);
    assert_eq!(array.get(2).unwrap().get(1).unwrap().get(), 29);
    let nested = registers.nested_array().get(1).unwrap();
    assert_eq!(nested.scalar_definition().get(), 30);
    let array = nested.array_definition();
    assert_eq!(array.get(0).unwrap().get(0).unwrap().get(), 31);
    assert_eq!(array.get(0).unwrap().get(1).unwrap().get(), 32);
    assert_eq!(array.get(1).unwrap().get(0).unwrap().get(), 33);
    assert_eq!(array.get(1).unwrap().get(1).unwrap().get(), 34);
    assert_eq!(array.get(2).unwrap().get(0).unwrap().get(), 35);
    assert_eq!(array.get(2).unwrap().get(1).unwrap().get(), 36);
    assert_eq!(nested.scalar_reference().get(), 37);
    let array = nested.array_reference();
    assert_eq!(array.get(0).unwrap().get(0).unwrap().get(), 38);
    assert_eq!(array.get(0).unwrap().get(1).unwrap().get(), 39);
    assert_eq!(array.get(1).unwrap().get(0).unwrap().get(), 40);
    assert_eq!(array.get(1).unwrap().get(1).unwrap().get(), 41);
    assert_eq!(array.get(2).unwrap().get(0).unwrap().get(), 42);
    assert_eq!(array.get(2).unwrap().get(1).unwrap().get(), 43);
    assert_eq!(registers.flat_array().get(0).unwrap().get(), 44);
    assert_eq!(registers.flat_array().get(1).unwrap().get(), 45);
    assert_eq!(registers.flat_array_reference().get(0).unwrap().get(), 46);
    assert_eq!(registers.flat_array_reference().get(1).unwrap().get(), 47);
    // External write: verify Mmio can handle varying register values.
    unsafe { (*peripheral.get()).scalar = 48 };
    assert_eq!(registers.scalar().get(), 48);
    // Perform a write.
    registers.scalar().set(u64::MAX);
    assert_eq!(unsafe { (*peripheral.get()).scalar }, u64::MAX);
    assert_eq!(registers.overlapped().get(), 0xff);
}

/// This performs offsetting operations using a base pointer without provenance.
/// This is an accurate model for real registers (it differs from [mmio] which
/// uses pointers with provenance to access a stack variable). In Miri, this can
/// catch UB caused by using offset functions that require the pointers to point
/// at allocations.
#[test]
fn offset() {
    let mmio = Mmio::new(core::ptr::null_mut());
    // Safety: This is not sound, there is not actually an outer_block at this
    // address. But this test never accesses the values, only computes offsets,
    // so it should not trigger UB.
    let registers = unsafe { outer_block::Real::new(mmio) };

    // Calculate the offsets for each register in outer_block.
    registers.scalar();
    registers.overlapped();
    fn check_inner(inner: inner_block::Real<Mmio>) {
        inner.scalar_definition();
        inner.array_definition().get(0).unwrap().get(0).unwrap();
        inner.array_definition().get(0).unwrap().get(1).unwrap();
        inner.array_definition().get(1).unwrap().get(0).unwrap();
        inner.array_definition().get(1).unwrap().get(1).unwrap();
        inner.array_definition().get(2).unwrap().get(0).unwrap();
        inner.array_definition().get(2).unwrap().get(1).unwrap();
        inner.scalar_reference();
        inner.array_reference().get(0).unwrap().get(0).unwrap();
        inner.array_reference().get(0).unwrap().get(1).unwrap();
        inner.array_reference().get(1).unwrap().get(0).unwrap();
        inner.array_reference().get(1).unwrap().get(1).unwrap();
        inner.array_reference().get(2).unwrap().get(0).unwrap();
        inner.array_reference().get(2).unwrap().get(1).unwrap();
    }
    check_inner(registers.nested());
    check_inner(registers.nested_array().get(0).unwrap());
    check_inner(registers.nested_array().get(1).unwrap());
    registers.flat_array().get(0).unwrap();
    registers.flat_array().get(1).unwrap();
    registers.flat_array_reference().get(0).unwrap();
    registers.flat_array_reference().get(1).unwrap();
}
