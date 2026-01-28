// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! # Addresses and Buses
//!
//! tock-registers supports many different types of registers, including:
//! 1. MMIO registers
//! 2. Several types of LiteX registers (these are MMIO, but have padding within the registers that
//!    depends on the chip's configuration and the register's data type)
//! 3. x86 port IO
//!
//! tock-registers calls each of these classes a "bus", as the mechanism used to access the
//! registers is different. The mechanism in which a register is addressed depends on the bus, so
//! each register address type corresponds to the bus. In other words:
//!
//! * The *type* of the address pointer corresponds to the bus.
//! * The *value* of the address pointer corresponds to the register on that bus.
//!
//! Therefore, all real register types ("real" here means it's not a fake/mock/stub for testing)
//! have a generic argument for their address type. That address type implements the `Address`
//! trait. In addition, address types should implement [`Bus<T>`] for every type `T` that they support.

use crate::{DataType, UIntLike};

/// Trait for addresses that can be offset without changing their type. See the module-level docs
/// for more information on address types.
pub trait Address: Copy {
    /// Adds the given offset to this address, returning an address pointing to a later location on
    /// the bus.
    /// # Safety
    /// `self` must point into a register span on the correct bus (based on `Self`). The entire
    /// range from `self` to the result (inclusive) must be in bounds of that register span. In
    /// particular, this must not "wrap around" the address space.
    unsafe fn byte_add(self, offset: usize) -> Self;
}

/// Address types should implement `Bus<T>` for each primitive type `T` that they support.
pub trait Bus<T: UIntLike>: Address {
    /// The size that a value of type T takes in this bus' address space. This exists because LiteX
    /// buses have intra-register padding for some types.
    const PADDED_SIZE: usize;
}

/// An accessor for a block of registers. Every Real type implements this. This trait is used to
/// construct the register blocks, including sub-blocks for larger register blocks and elements of
/// arrays.
pub trait Block: Copy {
    type Address: Address;
    /// Size this blocks occupies in the address space. Depends on the address type.
    const SIZE: usize;

    /// Constructs an accessor for a register block.
    /// # Safety
    /// `address` must point to registers on the bus corresponding to Self::Address with the layout
    /// that correctly matches this type's definition.
    unsafe fn new(address: Self::Address) -> Self;
}

/// An alias for `Bus<D::Value>`. Used so you don't have to write
/// `BusValue<<T as DataType>::Value>` (this simplifies the generated code quite a bit).
pub trait DataTypeBus<D: DataType>: Bus<D::Value> {
    /// Alias for Bus::PADDED_SIZE;
    const PADDED_SIZE: usize = <Self as Bus<D::Value>>::PADDED_SIZE;
}
impl<D: DataType, T: Bus<D::Value>> DataTypeBus<D> for T {}
