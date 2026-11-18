// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Module containing the [`LocalRegisterCopy`] type. Please refer to
//! its documentation.

use core::fmt;
use core::marker::PhantomData;

use crate::fields::{Field, PartialRegisterValue, TryFromValue};
use crate::{RegisterLongName, UIntLike};

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
pub struct LocalRegisterCopy<T: Copy, R: RegisterLongName = ()> {
    value: T,
    associated_register: PhantomData<R>,
}

impl<T: Copy, R: RegisterLongName> LocalRegisterCopy<T, R> {
    pub const fn new(value: T) -> Self {
        LocalRegisterCopy {
            value,
            associated_register: PhantomData,
        }
    }

    /// Get the raw register value
    #[inline]
    pub fn get(&self) -> T {
        self.value
    }

    /// Set the raw register value
    #[inline]
    pub fn set(&mut self, value: T) {
        self.value = value;
    }
}

impl<T: UIntLike, R: RegisterLongName> LocalRegisterCopy<T, R> {
    /// Read the value of the given field
    #[inline]
    pub fn read(&self, field: Field<T, R>) -> T {
        field.read(self.get())
    }

    /// Read value of the given field as an enum member
    #[inline]
    pub fn read_as_enum<E: TryFromValue<T, EnumType = E>>(&self, field: Field<T, R>) -> Option<E> {
        field.read_as_enum(self.get())
    }

    /// Write the value of one or more fields, overwriting the other fields with
    /// zero
    #[inline]
    pub fn write(&mut self, field: impl Into<PartialRegisterValue<T, R>>) {
        self.set(field.into().value());
    }

    /// Write the value of one or more fields, leaving the other fields
    /// unchanged
    #[inline]
    pub fn modify(&mut self, field: impl Into<PartialRegisterValue<T, R>>) {
        self.set(field.into().modify(self.get()));
    }

    /// Check if the field value in `value` exactly matches the same field in
    /// the register.
    #[inline]
    pub fn matches(&self, field: impl Into<PartialRegisterValue<T, R>>) -> bool {
        field.into().matches(self.get())
    }

    /// Do a bitwise AND operation of the stored value and the passed in value
    /// and return a new LocalRegisterCopy.
    #[inline]
    #[must_use]
    pub fn bitand(&self, rhs: T) -> LocalRegisterCopy<T, R> {
        LocalRegisterCopy::new(self.value & rhs)
    }

    #[inline]
    pub fn debug(&self) -> crate::debug::RegisterDebugValue<T, R>
    where
        R: crate::debug::RegisterDebugInfo<T>,
    {
        crate::debug::RegisterDebugValue {
            data: self.get(),
            _reg: core::marker::PhantomData,
        }
    }
}

impl<T: Copy + fmt::Debug, R: RegisterLongName> fmt::Debug for LocalRegisterCopy<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

// Helper macro to implement From<LocalRegisterCopy<T: UIntLike>, R>>
// for <T: UIntLike>
macro_rules! From_impl_for {
    ($type:ty) => {
        impl<R: RegisterLongName> From<LocalRegisterCopy<$type, R>> for $type {
            fn from(r: LocalRegisterCopy<$type, R>) -> $type {
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
