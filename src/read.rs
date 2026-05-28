// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Better Bytes 2026.

use crate::debug::{RegisterDebugInfo, RegisterDebugValue};
use crate::fields::{Field, FieldValue, TryFromValue};
#[cfg(feature = "register_types")]
use crate::{Address, BorrowedBus, Bus};
use crate::{DataType, LocalRegisterCopy, Register, UIntLike};
use core::marker::PhantomData;

/// A register that can be read.
pub trait Read: Register {
    /// Get the raw register value
    fn get(self) -> <Self::DataType as DataType>::Value;

    /// Read the value of the given field
    fn read(
        self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as DataType>::LongName>,
    ) -> <Self::DataType as DataType>::Value
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        field.read(self.get())
    }

    /// Set the raw register value
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
    fn read_as_enum<E: TryFromValue<<Self::DataType as DataType>::Value, EnumType = E>>(
        self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as DataType>::LongName>,
    ) -> Option<E>
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        field.read_as_enum(self.get())
    }

    /// Make a local copy of the register
    fn extract(
        self,
    ) -> LocalRegisterCopy<
        <Self::DataType as DataType>::Value,
        <Self::DataType as DataType>::LongName,
    > {
        LocalRegisterCopy::new(self.get())
    }

    /// Check if one or more bits in a field are set
    fn is_set(
        self,
        field: Field<<Self::DataType as DataType>::Value, <Self::DataType as DataType>::LongName>,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        field.is_set(self.get())
    }

    /// Check if any bits corresponding to the mask in the passed `FieldValue`
    /// are set.  This function is identical to `is_set()` but operates on a
    /// `FieldValue` rather than a `Field`, allowing for checking if any bits
    /// are set across multiple, non-contiguous portions of a bitfield.
    fn any_matching_bits_set(
        self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        field.any_matching_bits_set(self.get())
    }

    /// Check if all specified parts of a field match
    fn matches_all(
        self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        field.matches_all(self.get())
    }

    /// Check if any of the passed parts of a field exactly match the contained
    /// value. This allows for matching on unset bits, or matching on specific
    /// values in multi-bit fields.
    fn matches_any(
        self,
        fields: &[FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >],
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        let value = self.get();
        fields
            .iter()
            .any(|field| value & field.mask() == field.value)
    }

    /// Returns a [`RegisterDebugValue`] that implements [`core::fmt::Debug`]. The debug
    /// information is extracted from `<Register>::DebugInfo`.
    fn debug(
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
        RegisterDebugValue {
            data: self.get(),
            _reg: PhantomData,
        }
    }
}

/// A Bus that implements `BusRead<T>` can support Read implementations with DataType T. Other
/// crates (e.g. LiteX registers) can implement this on their own buses so that Read works with
/// them as well.
#[cfg(feature = "register_types")]
pub trait BusRead<T>: Bus<T> {
    /// # Safety
    /// There must be a readable register of type T at `pointer`, and if the register itself has
    /// safety invariants (i.e. it is `UnsafeRead`) the caller must satisfy those. The caller is
    /// responsible for avoiding data races.
    unsafe fn read(self) -> T;
}

#[cfg(feature = "register_types")]
impl<'b, T, A: Address + BusRead<T>> BusRead<T> for BorrowedBus<'b, A> {
    unsafe fn read(self) -> T {
        // Safety: We are the same Bus as A, so the caller has already satisfied all the
        // requirements of read.
        unsafe { self.address().read() }
    }
}

/// The macro that goes along with the Read trait. We don't expect this macro to be used by
/// tock_register's users, instead it is invoked by the generated code.
#[cfg(feature = "register_types")]
#[macro_export]
macro_rules! Read {
    // Provides a real implementation of the trait. The trailing $rest argument is for future
    // compatibility: it allows the procedural macro to pass additional arguments in the future
    // without breaking compatibility with this implementation of Read!.
    (real_impl, $name:ident, $datatype:ty, $($rest:tt)*) => {
        impl<B: Bus + $crate::BusRead<<$datatype as $crate::DataType>::Value>> $crate::Read
            for $name<B>
        {
            fn get(self) -> <$datatype as $crate::DataType>::Value {
                // Safety: The caller assured this GenericReal points at a register on bus B with
                // value type $datatype::Value that is safe to read. The code that constructed
                // `self` guaranteed that they would avoid data races (precondition of Self::new).
                unsafe { self.address.read() }
            }
        }
    };
    // Catch-all case that emits nothing if register_map! invokes it with an unknown first
    // argument. This is so that we can add new functionality into the operations macros without
    // breaking backwards compatibility (though registers! would need to be compatible with this
    // do-nothing block).
    ($($unknown:tt)*) => {};
}
