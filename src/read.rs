// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Better Bytes 2026.

use crate::debug::{RegisterDebugInfo, RegisterDebugValue};
use crate::fields::{Field, PartialRegisterValue, TryFromValue};
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
    /// # use core::ptr::NonNull;
    /// # use tock_registers::{register_bitfields, register_map_mmio64};
    /// # use tock_registers::{Mmio64, Read};
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
    /// register_map_mmio64![example: EXAMPLEREG::Register { Read }];
    /// # fn main() {
    /// let mut reg: u8 = 2;
    /// let mmio = Mmio64::new(NonNull::from(&mut reg).cast());
    /// let reg = unsafe { example::Real::new(mmio) };
    ///
    /// match reg.read_as_enum(EXAMPLEREG::TESTFIELD) {
    ///     Some(EXAMPLEREG::TESTFIELD::Value::Foo) => "Tock",
    ///     Some(EXAMPLEREG::TESTFIELD::Value::Bar) => "is",
    ///     Some(EXAMPLEREG::TESTFIELD::Value::Baz) => "awesome!",
    ///     None => panic!("boo!"),
    /// };
    /// # }
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
    /// or `PartialRegisterValue` are set.  This function is identical to
    /// `is_set()` but operates on a field value rather than a `Field`,
    /// allowing for checking if any bits are set across multiple,
    /// non-contiguous portions of a bitfield.
    fn any_matching_bits_set(
        self,
        field: impl Into<
            PartialRegisterValue<
                <Self::DataType as DataType>::Value,
                <Self::DataType as DataType>::LongName,
            >,
        >,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        field.into().any_matching_bits_set(self.get())
    }

    /// Check if the fields covered in the [`PartialRegisterValue`] value
    /// exactly match the same fields in the register.
    ///
    /// Fields which are not covered in the [`PartialRegisterValue`] are
    /// ignored.
    fn matches(
        self,
        value: impl Into<
            PartialRegisterValue<
                <Self::DataType as DataType>::Value,
                <Self::DataType as DataType>::LongName,
            >,
        >,
    ) -> bool
    where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        value.into().matches(self.get())
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
    /// safety invariants the caller must satisfy those. The caller is responsible for avoiding
    /// data races.
    unsafe fn read(self) -> T;
}

#[cfg(feature = "register_types")]
impl<T, A: Address + BusRead<T>> BusRead<T> for BorrowedBus<'_, A> {
    unsafe fn read(self) -> T {
        // Safety: We are the same Bus as A, so the caller has already satisfied all the
        // requirements of read.
        unsafe { self.address().read() }
    }
}

#[cfg(feature = "register_types")]
#[doc(hidden)]
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
                // Safety: The caller assured this register accessor points at a register on bus B
                // with value type $datatype::Value that is safe to read. The code that constructed
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

#[cfg(test)]
mod tests {
    use crate::{register_bitfields, FakeRegister, NoAccess, Read, Safe};

    register_bitfields![u8,
        Status [
            TXCOMPLETE  OFFSET(0) NUMBITS(1) [],
            TXINTERRUPT OFFSET(1) NUMBITS(1) [],
            RXCOMPLETE  OFFSET(2) NUMBITS(1) [],
            RXINTERRUPT OFFSET(3) NUMBITS(1) [],
            MODE        OFFSET(4) NUMBITS(2) [
                FullDuplex = 0,
                HalfDuplex = 1,
                Loopback = 2,
                Disabled = 3
            ],
            ERRORCOUNT OFFSET(6) NUMBITS(2) []
        ],
    ];

    fn fake_status(value: u8) -> FakeRegister<u8, Status::Register, Safe, NoAccess> {
        FakeRegister::new(value).on_read(|value| value)
    }

    #[test]
    fn matches() {
        // TXCOMPLETE set, MODE == FullDuplex.
        let status = fake_status(0b0000_0001);
        let m = status.matches(Status::TXCOMPLETE::SET + Status::MODE::FullDuplex);
        assert!(m);

        // Flip TXCOMPLETE off: no longer matches.
        let status = fake_status(0b0000_0000);
        let m = status.matches(Status::TXCOMPLETE::SET + Status::MODE::FullDuplex);
        assert!(!m);

        // Three flags set, matches only two flags.
        let status = fake_status(0b0000_0111);
        let m = status.matches(Status::TXCOMPLETE::SET + Status::TXINTERRUPT::SET);
        assert!(m);

        // A two bit field matches.
        let status = fake_status(0b0010_0111);
        let m = status.matches(Status::MODE::Loopback);
        assert!(m);

        // A two bit non-value field matches.
        let status = fake_status(0b1011_0111);
        let m = status.matches(Status::ERRORCOUNT.val(2));
        assert!(m);
    }

    #[test]
    fn any_matching_bits_set() {
        // TXCOMPLETE set
        let status = fake_status(0b0000_0001);
        let m = status.any_matching_bits_set(Status::TXCOMPLETE::SET);
        assert!(m);

        // Flip TXCOMPLETE off: no longer matches.
        let status = fake_status(0b0000_0000);
        let m = status.any_matching_bits_set(Status::TXCOMPLETE::SET);
        assert!(!m);

        // Three flags set, matches only two flags.
        let status = fake_status(0b0000_0111);
        let m = status.any_matching_bits_set(Status::TXCOMPLETE::SET + Status::TXINTERRUPT::SET);
        assert!(m);

        // Three flags set, matches all flags.
        let status = fake_status(0b0000_0111);
        let m = status.any_matching_bits_set(
            Status::TXCOMPLETE::SET + Status::TXINTERRUPT::SET + Status::RXCOMPLETE::SET,
        );
        assert!(m);
    }
}
