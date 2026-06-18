// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Better Bytes 2026.

use crate::{fields::FieldValue, DataType, LocalRegisterCopy, Register, UIntLike};

/// A register that can be written, but which can induce undefined behavior if written incorrectly.
///
/// # Safety
/// The exact safety invariants of this register are hardware-dependent and should be specified as
/// part of the documentation for the register layout. For every function in this trait, the user
/// is responsible for following the hardware's safety invariants.
pub trait UnsafeWrite: Register {
    /// Set the raw register value
    unsafe fn set(self, value: <Self::DataType as DataType>::Value);

    /// Write the value of one or more fields, overwriting the other fields with
    /// zero
    unsafe fn write(
        &self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        unsafe { self.set(field.value) };
    }

    /// Write the value of one or more fields, maintaining the value of
    /// unchanged fields via a provided original value, rather than a register
    /// read.
    unsafe fn modify_no_read(
        &self,
        original: LocalRegisterCopy<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        let new = field.modify(original.get());
        // Safety: The caller has complied with this register's safety requirements.
        unsafe { self.set(new) };
    }
}

/// The macro that goes along with the UnsafeWrite trait. We don't expect this macro to be used by
/// tock_register's users, instead it is invoked by the generated code.
#[macro_export]
macro_rules! UnsafeWrite {
    (real_impl, $name:ident, $datatype:ty, $($rest:tt)*) => {
        impl<B: Bus + $crate::BusWrite<<$datatype as $crate::DataType>::Value>> $crate::UnsafeWrite
            for $name<B>
        {
            unsafe fn set(self, value: <$datatype as $crate::DataType>::Value) {
                // Safety: The caller assured this register accessor points at a register on bus B
                // with value type $datatype::Value that is safe to write. The code that
                // constructed `self` guaranteed that they would avoid data races (precondition of
                // Self::new). The caller has complied with the hardware-specific safety
                // requirements for reading this register.
                unsafe { self.address.write(value) }
            }
        }
    };
    ($($unknown:tt)*) => {};
}
