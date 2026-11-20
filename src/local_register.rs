// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Module containing the [`LocalRegisterCopy`] type. Please refer to
//! its documentation.

use core::fmt;

use crate::fields::{Field, FieldValue, TryFromValue};
use crate::{DataType, UIntLike};

/// A read-write copy of register contents.
///
/// This behaves very similarly to a read-write register, but instead of doing a
/// volatile read to MMIO to get the value for each function call, a copy of the
/// register contents are stored locally in memory. This allows a peripheral to
/// do a single read on a register, and then check which bits are set without
/// having to do a full MMIO read each time. It also allows the value of the
/// register to be "cached" in case the peripheral driver needs to clear the
/// register in hardware yet still be able to check the bits.  You can write to
/// a local register, which will modify the stored value, but will not modify
/// any hardware because it operates only on local copy.
///
/// This type does not implement the [`Read`](trait@crate::Read) and
/// [`Write`](trait@crate::Write) traits because it requires a mutable reference
/// to modify the contained value. It still mirrors the interface which would be
/// exposed by a type implementing [`Read`](trait@crate::Read) and
/// [`Write`](trait@crate::Write).
#[derive(Copy, Clone)]
pub struct LocalRegisterCopy<DT: DataType> {
    value: DT::Value,
}

impl<DT: DataType> LocalRegisterCopy<DT> {
    pub const fn new(value: DT::Value) -> Self {
        LocalRegisterCopy { value }
    }

    /// Get the raw register value
    #[inline]
    pub fn get(&self) -> DT::Value {
        self.value
    }

    /// Set the raw register value
    #[inline]
    pub fn set(&mut self, value: DT::Value) {
        self.value = value;
    }
}

impl<DT: DataType<Value: UIntLike>> LocalRegisterCopy<DT> {
    /// Read the value of the given field
    #[inline]
    pub fn read(&self, field: Field<DT::Value, DT::LongName>) -> DT::Value {
        field.read(self.get())
    }

    /// Read value of the given field as an enum member
    #[inline]
    pub fn read_as_enum<E: TryFromValue<DT::Value, EnumType = E>>(
        &self,
        field: Field<DT::Value, DT::LongName>,
    ) -> Option<E> {
        field.read_as_enum(self.get())
    }

    /// Write the value of one or more fields, overwriting the other fields with
    /// zero
    #[inline]
    pub fn write(&mut self, field: FieldValue<DT::Value, DT::LongName>) {
        self.set(field.value);
    }

    /// Write the value of one or more fields, leaving the other fields
    /// unchanged
    #[inline]
    pub fn modify(&mut self, field: FieldValue<DT::Value, DT::LongName>) {
        self.set(field.modify(self.get()));
    }

    /// Check if one or more bits in a field are set
    #[inline]
    pub fn is_set(&self, field: Field<DT::Value, DT::LongName>) -> bool {
        field.is_set(self.get())
    }

    /// Check if any bits corresponding to the mask in the passed `FieldValue`
    /// are set.
    #[inline]
    pub fn any_matching_bits_set(&self, field: FieldValue<DT::Value, DT::LongName>) -> bool {
        field.any_matching_bits_set(self.get())
    }

    /// Check if all specified parts of a field match
    #[inline]
    pub fn matches_all(&self, field: FieldValue<DT::Value, DT::LongName>) -> bool {
        field.matches_all(self.get())
    }

    /// Check if any of the passed parts of a field exactly match the contained
    /// value. This allows for matching on unset bits, or matching on specific
    /// values in multi-bit fields.
    #[inline]
    pub fn matches_any(&self, fields: &[FieldValue<DT::Value, DT::LongName>]) -> bool {
        fields
            .iter()
            .any(|field| self.get() & field.mask() == field.value)
    }

    /// Do a bitwise AND operation of the stored value and the passed in value
    /// and return a new LocalRegisterCopy.
    #[inline]
    #[must_use]
    pub fn bitand(&self, rhs: DT::Value) -> LocalRegisterCopy<DT> {
        LocalRegisterCopy::new(self.value & rhs)
    }

    #[inline]
    pub fn debug(&self) -> crate::debug::RegisterDebugValue<DT::Value, DT::LongName>
    where
        DT::LongName: crate::debug::RegisterDebugInfo<DT::Value>,
    {
        crate::debug::RegisterDebugValue {
            data: self.get(),
            _reg: core::marker::PhantomData,
        }
    }
}

impl<DT: DataType<Value: fmt::Debug>> fmt::Debug for LocalRegisterCopy<DT> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

// Helper macro to implement From<LocalRegisterCopy<T: UIntLike>, R>>
// for <T: UIntLike>
macro_rules! From_impl_for {
    ($type:ty) => {
        impl<DT: DataType<Value = $type>> From<LocalRegisterCopy<DT>> for $type {
            fn from(r: LocalRegisterCopy<DT>) -> $type {
                r.value
            }
        }
    };
}

From_impl_for!(u8);
From_impl_for!(u16);
From_impl_for!(u32);
From_impl_for!(u64);
From_impl_for!(u128);
From_impl_for!(usize);
