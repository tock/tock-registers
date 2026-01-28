// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{RegisterLongName, UIntLike};
use core::marker::PhantomData;

/// Type implemented by all registers.
pub trait Register: Copy {
    /// The data type used for this register in the type specification.
    type DataType: DataType;
}

/// Trait used to retrieve information about a register from the type given in its specification.
pub trait DataType {
    /// The register's value type. This is the type passed over the bus when accessing the
    /// register.
    type Value: UIntLike;

    /// The bitfield used when data is read from this register.
    type Read: RegisterLongName;

    /// The bitfield used when data is written to this register.
    type Write: RegisterLongName;
}

impl<U: UIntLike> DataType for U {
    type Value = U;
    type Read = ();
    type Write = ();
}

/// Aliased is a DataType that has a different RegisterLongName when read than it does when
/// written.
pub struct Aliased<Read: DataType, Write: DataType<Value = Read::Value>> {
    _empty: Empty,
    _read: PhantomData<Read>,
    _write: PhantomData<Write>,
}

impl<Read: DataType, Write: DataType<Value = Read::Value>> DataType for Aliased<Read, Write> {
    type Value = Read::Value;
    type Read = Read::Read;
    type Write = Write::Write;
}

/// Used to make Aliased uninhabited.
enum Empty {}
