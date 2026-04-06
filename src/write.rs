// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright Better Bytes 2026.

use crate::{fields::FieldValue, DataType, LocalRegisterCopy, Read, Register};
#[cfg(feature = "register_types")]
use crate::{Address, BorrowedBus, Bus, UIntLike};

/// A register that can be written.
pub trait Write: Register {
    /// Set the raw register value
    fn set(self, value: <Self::DataType as DataType>::Value);

    /// Write the value of one or more fields, overwriting the other fields with
    /// zero
    fn write(
        &self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) {
        self.set(field.value);
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
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) {
        self.set(field.modify(original.get()));
    }
}

/// A register that can be read and written with the same RegisterLongName. This is automatically
/// implemented for registers that are both Read and Write with the same RegisterLongName.
pub trait ReadWrite: Read + Write {
    /// Write the value of one or more fields, leaving the other fields
    /// unchanged
    fn modify(
        &self,
        field: FieldValue<
            <Self::DataType as DataType>::Value,
            <Self::DataType as DataType>::LongName,
        >,
    ) {
        self.set(field.modify(self.get()));
    }
}

impl<R: Read + Write> ReadWrite for R {}

/// A Bus that implements `BusWrite<T>` can support Write implementations with DataType T. Other
/// crates (e.g. LiteX registers) can implement this on their own buses so that Write works with
/// them as well.
#[cfg(feature = "register_types")]
pub trait BusWrite<T: UIntLike>: Bus<T> {
    /// # Safety
    /// There must be a writable register of type T at `pointer`, and if the register itself has
    /// safety invariants (i.e. it is `UnsafeWrite`) the caller must satisfy those. The caller is
    /// responsible for avoiding data races.
    unsafe fn write(self, value: T);
}

#[cfg(feature = "register_types")]
impl<'b, T: UIntLike, A: Address + BusWrite<T>> BusWrite<T> for BorrowedBus<'b, A> {
    unsafe fn write(self, value: T) {
        // Safety: We are the same Bus as A, so the caller has already satisfied all the
        // requirements of write.
        unsafe { self.address().write(value) }
    }
}

/// The macro that goes along with the Write trait. We don't expect this macro to be used by
/// tock_register's users, instead it is invoked by the generated code.
#[cfg(feature = "register_types")]
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
                // Safety: The caller assured this GenericReal points at a register on bus B with
                // value type $datatype::Value that is safe to write. The code that constructed
                // `self` guaranteed that they would avoid data races (precondition of Self::new).
                unsafe { self.address.write(value) }
            }
        }
    };
    // Catch-all case that emits nothing if registers! invokes it with an unknown first argument.
    // This is so that we can add new functionality into the operations traits without breaking
    // backwards compatibility (though registers! would need to be compatible with this do-nothing
    // block).
    ($($unknown:tt)*) => {};
}
