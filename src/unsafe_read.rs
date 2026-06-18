// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Better Bytes 2026.

use crate::debug::{RegisterDebugInfo, RegisterDebugValue};
use crate::fields::{Field, FieldValue, TryFromValue};
use crate::{DataType, LocalRegisterCopy, Register, UIntLike};
use core::marker::PhantomData;

/// A register that can be read, but which can induce undefined behavior if read at the wrong time.
///
/// # Safety
/// The exact safety invariants of this register are hardware-dependent and should be specified as
/// part of the documentation for the register layout. For every function in this trait, the user
/// is responsible for following the hardware's safety invariants.
pub trait UnsafeRead: Register {
    /// Get the raw register value
    unsafe fn get(self) -> <Self::DataType as DataType>::Value;

    /// Read the value of the given field
    unsafe fn read(
        self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as DataType>::LongName>,
    ) -> <Self::DataType as DataType>::Value
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        field.read(unsafe { self.get() })
    }

    /// Read value of the given field as an enum member
    ///
    /// The [`register_bitfields!`](crate::register_bitfields) macro will
    /// generate an enum containing the various named field variants and
    /// implementing the required [`TryFromValue`] trait. It is accessible as
    /// `$REGISTER_NAME::$FIELD_NAME::Value`.
    ///
    /// This method can be useful to symbolically represent read register field
    /// states throughout the codebase and to enforce exhaustive matches over
    /// all defined valid register field values.
    ///
    /// ## Usage Example
    ///
    /// ```rust
    /// # use tock_registers::interfaces::Readable;
    /// # use tock_registers::registers::InMemoryRegister;
    /// # use tock_registers::register_bitfields;
    /// register_bitfields![u8,
    ///     EXAMPLEREG [
    ///         TESTFIELD OFFSET(0) NUMBITS(2) [
    ///             Foo = 0,
    ///             Bar = 1,
    ///             Baz = 2,
    ///         ],
    ///     ],
    /// ];
    ///
    /// let reg: InMemoryRegister<u8, EXAMPLEREG::Register> =
    ///     InMemoryRegister::new(2);
    ///
    /// match reg.read_as_enum(EXAMPLEREG::TESTFIELD) {
    ///     Some(EXAMPLEREG::TESTFIELD::Value::Foo) => "Tock",
    ///     Some(EXAMPLEREG::TESTFIELD::Value::Bar) => "is",
    ///     Some(EXAMPLEREG::TESTFIELD::Value::Baz) => "awesome!",
    ///     None => panic!("boo!"),
    /// };
    /// ```
    unsafe fn read_as_enum<E: TryFromValue<<Self::DataType as DataType>::Value, EnumType = E>>(
        self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as DataType>::LongName>,
    ) -> Option<E>
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        field.read_as_enum(unsafe { self.get() })
    }

    /// Make a local copy of the register
    unsafe fn extract(
        self,
    ) -> LocalRegisterCopy<
        <Self::DataType as DataType>::Value,
        <Self::DataType as DataType>::LongName,
    > {
        // Safety: The caller has complied with this register's safety requirements.
        LocalRegisterCopy::new(unsafe { self.get() })
    }

    /// Check if one or more bits in a field are set
    unsafe fn is_set(
        self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as DataType>::LongName>,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        field.is_set(unsafe { self.get() })
    }

    /// Check if any bits corresponding to the mask in the passed `FieldValue`
    /// are set.  This function is identical to `is_set()` but operates on a
    /// `FieldValue` rather than a `Field`, allowing for checking if any bits
    /// are set across multiple, non-contiguous portions of a bitfield.
    unsafe fn any_matching_bits_set(
        self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        field.any_matching_bits_set(unsafe { self.get() })
    }

    /// Check if all specified parts of a field match
    unsafe fn matches_all(
        self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        field.matches_all(unsafe { self.get() })
    }

    /// Check if any of the passed parts of a field exactly match the contained
    /// value. This allows for matching on unset bits, or matching on specific
    /// values in multi-bit fields.
    unsafe fn matches_any(
        self,
        fields: &[FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >],
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        // Safety: The caller has complied with this register's safety requirements.
        let value = unsafe { self.get() };
        fields
            .iter()
            .any(|field| value & field.mask() == field.value)
    }

    /// Returns a [`RegisterDebugValue`] that implements [`core::fmt::Debug`]. The debug
    /// information is extracted from `<Register>::DebugInfo`.
    unsafe fn debug(
        self,
    ) -> RegisterDebugValue<
        <Self::DataType as DataType>::Value,
        <Self::DataType as DataType>::LongName,
    >
    where
        <Self::DataType as DataType>::Value: UIntLike,
        <Self::DataType as DataType>::LongName:
            RegisterDebugInfo<<Self::DataType as DataType>::Value>,
    {
        // Safety: The caller has complied with this register's safety requirements.
        let data = unsafe { self.get() };
        RegisterDebugValue {
            data,
            _reg: PhantomData,
        }
    }
}

/// The macro that goes along with the UnsafeRead trait. We don't expect this macro to be used by
/// tock_register's users, instead it is invoked by the generated code.
#[macro_export]
macro_rules! UnsafeRead {
    (real_impl, $name:ident, $datatype:ty, $($rest:tt)*) => {
        impl<B: Bus + $crate::BusRead<<$datatype as $crate::DataType>::Value>> $crate::UnsafeRead
            for $name<B>
        {
            unsafe fn get(self) -> <$datatype as $crate::DataType>::Value {
                // Safety: The caller assured this register accessor points at a register on bus B
                // with value type $datatype::Value that is safe to read. The code that constructed
                // `self` guaranteed that they would avoid data races (precondition of Self::new).
                // The caller has complied with the hardware-specific safety requirements for
                // reading this register.
                unsafe { self.address.read() }
            }
        }
    };
    ($($unknown:tt)*) => {};
}
