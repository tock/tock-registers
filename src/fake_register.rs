// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{DataType, Read, Register, UIntLike, Write};

pub struct FakeRegister<Data: Copy, DT: DataType, R: Access, W: Access> {
    data: Data,
    read: R::ReadFn<Data, DT::Value>,
    write: W::WriteFn<Data, DT::Value>,
}

impl<Data: Copy, DT: DataType> FakeRegister<Data, DT, NoAccess, NoAccess> {
    pub fn new(data: Data) -> Self {
        Self {
            data,
            read: (),
            write: (),
        }
    }
}

impl<Data: Copy, DT: DataType, R: Access, W: Access> FakeRegister<Data, DT, R, W> {
    pub fn on_read(self, fcn: fn(Data) -> DT::Value) -> FakeRegister<Data, DT, Safe, W> {
        FakeRegister {
            data: self.data,
            read: fcn,
            write: self.write,
        }
    }

    pub fn on_write(self, fcn: fn(Data, DT::Value)) -> FakeRegister<Data, DT, R, Safe> {
        FakeRegister {
            data: self.data,
            read: self.read,
            write: fcn,
        }
    }

    pub fn on_unsafe_read(
        self,
        fcn: unsafe fn(Data) -> DT::Value,
    ) -> FakeRegister<Data, DT, Unsafe, W> {
        FakeRegister {
            data: self.data,
            read: fcn,
            write: self.write,
        }
    }

    pub fn on_unsafe_write(
        self,
        fcn: unsafe fn(Data, DT::Value),
    ) -> FakeRegister<Data, DT, R, Unsafe> {
        FakeRegister {
            data: self.data,
            read: self.read,
            write: fcn,
        }
    }
}

impl<Data: Copy, DT: DataType, R: Access, W: Access> Clone for FakeRegister<Data, DT, R, W> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Data: Copy, DT: DataType, R: Access, W: Access> Copy for FakeRegister<Data, DT, R, W> {}

impl<Data: Copy + Default, DT: DataType> Default for FakeRegister<Data, DT, NoAccess, NoAccess> {
    fn default() -> Self {
        Self::new(Data::default())
    }
}

impl<Data: Copy, DT: DataType, R: Access, W: Access> Register for FakeRegister<Data, DT, R, W> {
    type DataType = DT;
}

impl<Data: Copy, DT: DataType, W: Access> Read for FakeRegister<Data, DT, Safe, W> {
    fn get(self) -> DT::Value {
        (self.read)(self.data)
    }
}

impl<Data: Copy, DT: DataType, R: Access> Write for FakeRegister<Data, DT, R, Safe> {
    fn set(self, value: DT::Value) {
        (self.write)(self.data, value)
    }
}

pub trait Access: sealed::Access {
    type ReadFn<Data: Copy, Value: UIntLike>: Copy;
    type WriteFn<Data: Copy, Value: UIntLike>: Copy;
}

pub enum NoAccess {}
impl Access for NoAccess {
    type ReadFn<Data: Copy, Value: UIntLike> = ();
    type WriteFn<Data: Copy, Value: UIntLike> = ();
}

pub enum Safe {}
impl Access for Safe {
    type ReadFn<Data: Copy, Value: UIntLike> = fn(Data) -> Value;
    type WriteFn<Data: Copy, Value: UIntLike> = fn(Data, Value);
}

pub enum Unsafe {}
impl Access for Unsafe {
    type ReadFn<Data: Copy, Value: UIntLike> = unsafe fn(Data) -> Value;
    type WriteFn<Data: Copy, Value: UIntLike> = unsafe fn(Data, Value);
}

mod sealed {
    pub trait Access {}
}

impl sealed::Access for NoAccess {}
impl sealed::Access for Safe {}
impl sealed::Access for Unsafe {}
