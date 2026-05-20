// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{RegisterLongName, UIntLike};

/// Type implemented by all registers.
pub trait Register: Copy {
    /// The data type used for this register in the type specification.
    type DataType: DataType;
}

/// Trait used to retrieve information about a register from the type given in its specification.
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
