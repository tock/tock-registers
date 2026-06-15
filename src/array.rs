// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{internal::RealPhantom, Address, Span};
use core::marker::PhantomData;

/// Interface for an array of registers (or register blocks, or register arrays). Each register
/// type should only implement RegisterArray for a single Len.
pub trait RegisterArray<L: Len>: Copy {
    /// The type of each element of this array.
    type Element: Copy;

    // Default implementations of get() and get_unchecked() are provided that depend on each other.
    // RealArray implementations need to implement at least one of the methods to avoid infinite
    // recursion. In general, real register implementations (such as RealRegisterArray) will
    // implement get_unchecked(), while fake implementations (for unit testing) will probably
    // implement get().

    /// Returns the `index`-th element of this array, or `None` if `index >= L::LEN`.
    fn get(self, index: usize) -> Option<Self::Element> {
        if index >= L::LEN {
            return None;
        }
        // Safety: We returned early if `index >= L::LEN`, so because `index` is an integer type we
        // know that `index < L::LEN`.
        Some(unsafe { self.get_unchecked(index) })
    }

    /// Returns the `index`-th element of this array.
    /// # Safety
    /// `index` must be less than `L::LEN`
    // Because this default implementation will only be used in testing environments, it's okay
    // (and beneficial) for it to have a runtime check.
    unsafe fn get_unchecked(self, index: usize) -> Self::Element {
        self.get(index).unwrap_or_else(|| {
            panic!(
                "get_unchecked called with out-of-bounds index {index}; len = {}",
                L::LEN
            )
        })
    }
}

/// Trait providing the length of a register array.
///
/// # Why does this exist?
/// The obvious way to design `RegisterArray` is to make its length an associated const:
/// ```
/// trait RegisterArray {
///     type Element;
///     const LEN: usize;
/// }
/// ```
/// However, when you have a nested array in a register block:
// TODO: Remove `ignore` once mmio32_register_map exists.
/// ```ignore
/// # fn main() {}
/// # use tock_registers::{mmio32_register_map, Read, Write};
/// mmio32_register_map! {
///     foo {
///         0 => a: [[[u8; 2]; 2]; 2] { Read, Write },
///     }
/// }
/// ```
/// the generated `foo::Interface` trait contains a type bound:
/// ```ignore
/// trait Interface {
///     type a: RegisterArray<Element: RegisterArray<Element: RegisterArray<Element:
///         Register<DataType = u8> + Read + Write>>>;
/// }
/// ```
/// and unfortunately this bound results in a compiler error. As best as I can tell, what happens
/// is that during implied bounds generation, anti-infinite-recursion logic kicks in and prevents
/// some of the implied bounds from being generated.
///
/// This issue can pass through the `Interface` traits, so simply banning nested array fields does
/// not solve the issue:
// TODO: Remove `ignore` once mmio32_register_map exists.
/// ```ignore
/// # fn main() {}
/// # use tock_registers::{mmio32_register_map, Read, Write};
/// mmio32_register_map! {
///     a: [u8; 2] { Read, Write },
///     b: [a; 2],
///     foo {
///         // This results in a compile error too
///         0 => c: [b; 2],
///     },
/// }
/// ```
/// Instead, we solve this problem by making every generated `RegisterArray` bound a different
/// trait. To do that, we have to make the `RegisterArray` trait generic. Technically, we could
/// keep that type parameter separate from the length, but combining them makes the design (and the
/// generated code) simpler.
pub trait Len {
    const LEN: usize;
}

/// Real implementation of RegisterArray.
// Safety invariant: `address` points to an array of `L::LEN` consecutive `Element` registers.
pub struct RealRegisterArray<Element: Span, L: Len> {
    address: Element::Address,
    _phantom: (RealPhantom, PhantomData<L>),
}

impl<Element: Span, L: Len> RealRegisterArray<Element, L> {
    /// Constructs an accessor for the register array at the given address.
    /// # Safety
    /// 1. `address` must point to a register array on the bus corresponding to `Self::Address`.
    /// 2. The element type `Element` and the length `L` must correctly describe the pointed-to
    ///    register array.
    /// 3. The returned register array accessor must not be used in a way that causes data races.
    ///    The exact requirements depend on the hardware, but it's usually best to access registers
    ///    from only one thread at a time.
    pub const unsafe fn new(address: Element::Address) -> RealRegisterArray<Element, L> {
        RealRegisterArray {
            address,
            _phantom: (RealPhantom::new(), PhantomData),
        }
    }
}

// Safety: Element::SIZE must be correct (Span's safety requirement), and there must be an array of
// L `Element`s at `address` (RealRegisterArray's safety invariant). Therefore SIZE's calculation
// is correct.
unsafe impl<Element: Span, L: Len> Span for RealRegisterArray<Element, L> {
    type Address = Element::Address;
    const SIZE: usize = Element::SIZE * L::LEN;

    unsafe fn with_addr(address: Element::Address) -> RealRegisterArray<Element, L> {
        RealRegisterArray {
            address,
            _phantom: Default::default(),
        }
    }

    type Borrowed<'b> = RealRegisterArray<Element::Borrowed<'b>, L>;
}

impl<Element: Span, L: Len> Clone for RealRegisterArray<Element, L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Element: Span, L: Len> Copy for RealRegisterArray<Element, L> {}

impl<Element: Span, L: Len> RegisterArray<L> for RealRegisterArray<Element, L> {
    type Element = Element;

    unsafe fn get_unchecked(self, index: usize) -> Element {
        let offset = index * Element::SIZE;
        // Safety:
        // We know `address` points to an array of `L::LEN` `Element`s. The caller guaranteed that
        // `index < L::LEN`, so index * Element::SIZE is within the array's bounds. That guarantees
        // that this offset falls within the bounds of a register span (as the array itself is a
        // register span).
        let address = unsafe { self.address.byte_add(offset) };
        // Safety: `address` was a correctly calculated index into the array, and we know the
        // element type is `Element`, so `address` points to an `Element`.
        unsafe { Element::with_addr(address) }
    }
}
