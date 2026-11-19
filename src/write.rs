// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Better Bytes 2026.

use crate::fields::{PartialRegisterValue, RegisterValue};
#[cfg(feature = "register_types")]
use crate::{Address, BorrowedBus, Bus};
use crate::{DataType, LocalRegisterCopy, Read, Register, UIntLike};

/// A register that can be written.
pub trait Write: Register {
    /// Set the raw register value
    fn set(self, value: <Self::DataType as DataType>::Value);

    /// Write the entire register value.
    fn write(
        &self,
        value: RegisterValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        self.set(value.value());
    }

    /// Write a [`PartialRegisterValue`] to change only the covered fields,
    /// leaving the other fields unchanged.
    fn modify(
        &self,
        value: impl Into<
            PartialRegisterValue<
                <Self::DataType as DataType>::Value,
                <Self::DataType as DataType>::LongName,
            >,
        >,
    ) where
        Self: Read,
        <Self::DataType as DataType>::Value: UIntLike,
    {
        self.set(value.into().modify(self.get()));
    }

    /// Write the value of one or more fields, maintaining the value of
    /// unchanged fields via a provided original value, rather than a register
    /// read.
    fn modify_no_read(
        &self,
        original: LocalRegisterCopy<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
        value: impl Into<
            PartialRegisterValue<
                <Self::DataType as DataType>::Value,
                <Self::DataType as DataType>::LongName,
            >,
        >,
    ) where
        <Self::DataType as DataType>::Value: UIntLike,
    {
        self.set(value.into().modify(original.get()));
    }
}

/// A Bus that implements `BusWrite<T>` can support Write implementations with DataType T. Other
/// crates (e.g. LiteX registers) can implement this on their own buses so that Write works with
/// them as well.
#[cfg(feature = "register_types")]
pub trait BusWrite<T>: Bus<T> {
    /// # Safety
    /// There must be a writable register of type T at `pointer`, and if the register itself has
    /// safety invariants the caller must satisfy those. The caller is responsible for avoiding
    /// data races.
    unsafe fn write(self, value: T);
}

#[cfg(feature = "register_types")]
impl<T, A: Address + BusWrite<T>> BusWrite<T> for BorrowedBus<'_, A> {
    unsafe fn write(self, value: T) {
        // Safety: We are the same Bus as A, so the caller has already satisfied all the
        // requirements of write.
        unsafe { self.address().write(value) }
    }
}

/// The macro that goes along with the Write trait. We don't expect this macro to be used by
/// tock_register's users, instead it is invoked by the generated code.
#[cfg(feature = "register_types")]
#[doc(hidden)]
#[macro_export]
macro_rules! Write {
    // Provides a real implementation of the trait. The trailing $rest argument is for future
    // compatibility: it allows the procedural macro to pass additional arguments in the future
    // without breaking compatibility with this implementation of Write!.
    (real_impl, $name:ident, $datatype:ty, $($rest:tt)*) => {
        impl<B: Bus + $crate::BusWrite<<$datatype as $crate::DataType>::Value>> $crate::Write
            for $name<B>
        {
            fn set(self, value: <$datatype as $crate::DataType>::Value) {
                // Safety: The caller assured this register accessor points at a register on bus B
                // with value type $datatype::Value that is safe to write. The code that
                // constructed `self` guaranteed that they would avoid data races (precondition of
                // Self::new).
                unsafe { self.address.write(value) }
            }
        }
    };
    // Catch-all case that emits nothing if register_map! invokes it with an unknown first
    // argument. This is so that we can add new functionality into the operations traits without
    // breaking backwards compatibility (though registers! would need to be compatible with this
    // do-nothing block).
    ($($unknown:tt)*) => {};
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use crate::fields::RegisterValue;
    use crate::{register_bitfields, FakeRegister, Safe, Write};

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

    fn fake_status(cell: &Cell<u8>) -> FakeRegister<&Cell<u8>, Status::Register, Safe, Safe> {
        cell.set(0);
        FakeRegister::new(cell)
            .on_read(|cell| cell.get())
            .on_write(|cell, value| cell.set(value))
    }

    #[test]
    fn write() {
        let cell = Cell::new(0);

        let status = fake_status(&cell);
        status.write(RegisterValue::from_partial(Status::TXCOMPLETE::SET));
        assert_eq!(cell.get(), 0b0000_0001);

        let status = fake_status(&cell);
        status.write(RegisterValue::from_partial(Status::RXINTERRUPT::SET));
        assert_eq!(cell.get(), 0b0000_1000);

        let status = fake_status(&cell);
        status.write(RegisterValue::from_partial(Status::RXINTERRUPT::CLEAR));
        assert_eq!(cell.get(), 0b0000_0000);

        let status = fake_status(&cell);
        status.write(RegisterValue::from_partial(
            Status::RXINTERRUPT::SET + Status::TXCOMPLETE::SET,
        ));
        assert_eq!(cell.get(), 0b0000_1001);

        let status = fake_status(&cell);
        status.write(RegisterValue::from_partial(Status::RXCOMPLETE::SET));
        assert_eq!(cell.get(), 0b0000_0100);
        status.write(RegisterValue::from_partial(Status::RXINTERRUPT::SET));
        assert_eq!(cell.get(), 0b0000_1000);
        status.write(RegisterValue::from_partial(Status::MODE::FullDuplex));
        assert_eq!(cell.get(), 0b0000_0000);
        status.write(RegisterValue::from_partial(Status::ERRORCOUNT.val(3)));
        assert_eq!(cell.get(), 0b1100_0000);
    }

    #[test]
    fn modify() {
        let cell = Cell::new(0);

        let status = fake_status(&cell);
        status.modify(Status::TXCOMPLETE::SET);
        assert_eq!(cell.get(), 0b0000_0001);

        let status = fake_status(&cell);
        status.modify(Status::RXINTERRUPT::SET);
        assert_eq!(cell.get(), 0b0000_1000);

        let status = fake_status(&cell);
        status.modify(Status::RXINTERRUPT::CLEAR);
        assert_eq!(cell.get(), 0b0000_0000);

        let status = fake_status(&cell);
        status.modify(Status::RXINTERRUPT::SET + Status::TXCOMPLETE::SET);
        assert_eq!(cell.get(), 0b0000_1001);

        let status = fake_status(&cell);
        status.modify(Status::RXCOMPLETE::SET);
        assert_eq!(cell.get(), 0b0000_0100);
        status.modify(Status::RXINTERRUPT::SET);
        assert_eq!(cell.get(), 0b0000_1100);
        status.modify(Status::MODE::FullDuplex);
        assert_eq!(cell.get(), 0b0000_1100);
        status.modify(Status::ERRORCOUNT.val(3));
        assert_eq!(cell.get(), 0b1100_1100);

        let status = fake_status(&cell);
        status.modify(Status::ERRORCOUNT.val(3));
        assert_eq!(cell.get(), 0b1100_0000);
        status.modify(Status::RXCOMPLETE::SET);
        assert_eq!(cell.get(), 0b1100_0100);
        status.modify(Status::ERRORCOUNT::CLEAR);
        assert_eq!(cell.get(), 0b0000_0100);

        let status = fake_status(&cell);
        status.modify(Status::ERRORCOUNT.val(3));
        assert_eq!(cell.get(), 0b1100_0000);
        status.modify(Status::RXCOMPLETE::SET + Status::RXINTERRUPT::SET);
        assert_eq!(cell.get(), 0b1100_1100);
        status
            .modify(Status::RXCOMPLETE::SET + Status::RXINTERRUPT::SET + Status::MODE::HalfDuplex);
        assert_eq!(cell.get(), 0b1101_1100);
    }
}
