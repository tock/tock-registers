// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{RegisterLongName, UIntLike};

/// Trait implemented by accessors for individual registers.
pub trait Register: Copy {
    /// This register's data type, as specified in [`register_map!`](crate::register_map)'s input.
    type DataType: DataType;
}

/// Conveys information about a register's data type.
pub trait DataType {
    /// The register's value type. This is the type passed over the bus when accessing the
    /// register. This will typically be UIntLike, but can sometimes be other types such as raw
    /// pointers.
    type Value: Copy;

    /// This register's bitfield.
    type LongName: RegisterLongName;
}

impl<U: UIntLike> DataType for U {
    type Value = U;
    type LongName = ();
}

impl<T: Sized> DataType for *const T {
    type Value = *const T;
    type LongName = ();
}

impl<T: Sized> DataType for *mut T {
    type Value = *mut T;
    type LongName = ();
}
