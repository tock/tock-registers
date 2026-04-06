// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{internal::RealPhantom, Address, Block};

/// Interface for an array of registers (or register blocks, or register arrays).
pub trait RegisterArray: Copy {
    /// The type of each element of this array.
    type Element: Copy;

    /// The number of elements of this array.
    const LEN: usize;

    /// Returns the `index`-th element of this array, or `None` if `index >= LEN`.
    fn get(self, index: usize) -> Option<Self::Element> {
        if index >= Self::LEN {
            return None;
        }
        // Safety: We returned early if `index >= Self::LEN`, so because `index` is an integer type
        // we know that `index < Self::LEN`.
        Some(unsafe { self.get_unchecked(index) })
    }

    /// Returns the `index`-th element of this array.
    /// # Safety
    /// `index` must be less than `LEN`
    unsafe fn get_unchecked(self, index: usize) -> Self::Element {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "get_unchecked called with out-of-bounds index {index}; len = {}",
                Self::LEN
            )
        })
    }
}

/// Real implementation of RegisterArray.
// Safety invariant: `address` points to an array of `LEN` consecurity `Element` registers.
#[derive(Clone, Copy)]
pub struct RealRegisterArray<Element: Block, const LEN: usize> {
    address: Element::Address,
    _phantom: RealPhantom,
}

impl<Element: Block, const LEN: usize> RealRegisterArray<Element, LEN> {
    /// Constructs an accessor for the register array at the given address.
    /// # Safety
    /// 1. `address` must point to a register array on the bus corresponding to `Self::Address`.
    /// 2. The register array's definition (as provided to the
    ///    [`registers`](macro@crate::registers) macro) must correctly describe the pointed-to
    ///    register array.
    /// 3. The returned register array accessor must not be used in a way that causes data races.
    ///    The exact requirements depend on the hardware, but it's usually best to access registers
    ///    from only one thread at a time.
    pub unsafe fn new(address: Element::Address) -> RealRegisterArray<Element, LEN> {
        RealRegisterArray {
            address,
            _phantom: RealPhantom::new(),
        }
    }
}

impl<Element: Block, const LEN: usize> Block for RealRegisterArray<Element, LEN> {
    type Address = Element::Address;
    const SIZE: usize = LEN * Element::SIZE;

    unsafe fn new(address: Element::Address) -> RealRegisterArray<Element, LEN> {
        RealRegisterArray {
            address,
            _phantom: RealPhantom::new(),
        }
    }

    type Borrowed<'b> = RealRegisterArray<Element::Borrowed<'b>, LEN>;
}

impl<Element: Block, const LEN: usize> RegisterArray for RealRegisterArray<Element, LEN> {
    type Element = Element;
    const LEN: usize = LEN;

    unsafe fn get_unchecked(self, index: usize) -> Element {
        let offset = index * Element::SIZE;
        // Safety:
        // We know `address` points to an array of `LEN` `Element`s. The caller guaranteed that
        // `index <= LEN`, so index * Element::SIZE is within the array's bounds. That guarantees
        // that this offset falls within the bounds of a register block (as the array itself is a
        // register block).
        let address = unsafe { self.address.byte_add(offset) };
        // Safety: `address` was a correctly calculated index into the array, and we know the
        // element type is `Element`, so `address` points to an `Element`.
        unsafe { Element::new(address) }
    }
}
