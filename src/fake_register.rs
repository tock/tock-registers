// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#[cfg(feature = "register_types")]
use crate::RegisterArray;
use crate::{DataType, LocalRegisterCopy, Read, Register, Write};

/// A fake register, for use in unit tests.
///
/// ```
/// # use std::cell::Cell;
/// # use tock_registers::{mmio32_registers, FakeRegister, LocalRegisterCopy, NoAccess, Read, Safe};
/// mmio32_registers! {
///     /// A random number generator, which simply returns a random byte on each read.
///     rng {
///         0 => random_byte: u8 { Read },
///     }
/// }
/// /// Fake version of the RNG peripheral which outputs incrementing "random" bytes.
/// struct FakeRng(Cell<u8>);
/// // Instead of defining a FakeRandomByte struct and implementing Register<DataType = u8> and
/// // Read on it, this implementation uses FakeRegister. This saves a lot of boilerplate,
/// // especially when a peripheral has many registers.
/// impl rng::Interface for &FakeRng {
///     type random_byte = FakeRegister<Self, u8, Safe, NoAccess>;
///     fn random_byte(self) -> FakeRegister<Self, u8, Safe, NoAccess> {
///         FakeRegister::new(self).on_read(|this| {
///             let next = this.0.get().wrapping_add(1);
///             this.0.set(next);
///             LocalRegisterCopy::new(next)
///         })
///     }
/// }
/// # fn main() {}
/// ```
///
/// A FakeRegister is constructed using the builder pattern. When the FakeRegister is first
/// constructed, it does not support any operations (the read and write access are both NoAccess).
/// That can be changed by calling [on_read](FakeRegister::on_read),
/// [on_write](FakeRegister::on_write), [on_unsafe_read](FakeRegister::on_unsafe_read), and/or
/// [on_unsafe_write](FakeRegister::on_unsafe_write), which will return a new FakeRegister with a
/// different type.
///
/// FakeRegister is limited to the Read and Write operations. Crates that add other operations
/// should consider implementing their own version of FakeRegister to make it easy to write fake
/// peripherals that support those operations.
pub struct FakeRegister<Data: Copy, DT: DataType, R: Access, W: Access> {
    data: Data,
    read: R::ReadFn<Data, DT>,
    write: W::WriteFn<Data, DT>,
}

impl<Data: Copy, DT: DataType> FakeRegister<Data, DT, NoAccess, NoAccess> {
    /// Constructs a new FakeRegister. `data` will be provided to any read and/or write functions
    /// that are provided to this FakeRegister.
    ///
    /// A typical use for `data` is to pass a reference to the simulated hardware's state. If you
    /// don't need to pass any data to your read/write functions, you can set `data` to `()`.
    pub const fn new(data: Data) -> Self {
        Self {
            data,
            read: (),
            write: (),
        }
    }
}

impl<Data: Copy, DT: DataType, R: Access, W: Access> FakeRegister<Data, DT, R, W> {
    /// Returns a new FakeRegister that implements [`trait@Read`] by invoking `fcn`.
    pub const fn on_read(
        self,
        fcn: fn(Data) -> LocalRegisterCopy<DT::Value, DT::LongName>,
    ) -> FakeRegister<Data, DT, Safe, W> {
        FakeRegister {
            data: self.data,
            read: fcn,
            write: self.write,
        }
    }

    /// Returns a new FakeRegister that implements [`trait@Write`] by invoking `fcn`.
    pub const fn on_write(
        self,
        fcn: fn(Data, LocalRegisterCopy<DT::Value, DT::LongName>),
    ) -> FakeRegister<Data, DT, R, Safe> {
        FakeRegister {
            data: self.data,
            read: self.read,
            write: fcn,
        }
    }

    /// Returns a new FakeRegister that implements `trait@UnsafeRead` by invoking `fcn`.
    pub const fn on_unsafe_read(
        self,
        fcn: unsafe fn(Data) -> LocalRegisterCopy<DT::Value, DT::LongName>,
    ) -> FakeRegister<Data, DT, Unsafe, W> {
        FakeRegister {
            data: self.data,
            read: fcn,
            write: self.write,
        }
    }

    /// Returns a new FakeRegister that implements `trait@UnsafeWrite` by invoking `fcn`.
    pub const fn on_unsafe_write(
        self,
        fcn: unsafe fn(Data, LocalRegisterCopy<DT::Value, DT::LongName>),
    ) -> FakeRegister<Data, DT, R, Unsafe> {
        FakeRegister {
            data: self.data,
            read: self.read,
            write: fcn,
        }
    }
}

// #[derive(Clone, Copy)] emits overly-conservative trait bounds which make FakeRegister not
// cloneable, so we manually implement Clone + Copy instead.
impl<Data: Copy, DT: DataType, R: Access, W: Access> Copy for FakeRegister<Data, DT, R, W> {}
impl<Data: Copy, DT: DataType, R: Access, W: Access> Clone for FakeRegister<Data, DT, R, W> {
    fn clone(&self) -> Self {
        *self
    }
}

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
        (self.read)(self.data).get()
    }
}

impl<Data: Copy, DT: DataType, R: Access> Write for FakeRegister<Data, DT, R, Safe> {
    fn set(self, value: DT::Value) {
        (self.write)(self.data, LocalRegisterCopy::new(value))
    }
}

/// Trait used to control whether a FakeRegister implements Read, Write, UnsafeRead, and
/// UnsafeWrite. This trait is sealed, as the only options are [`NoAccess`], [`Safe`], and
/// [`Unsafe`].
pub trait Access: sealed::Access {
    /// The function pointer type used for read operations.
    type ReadFn<Data: Copy, DT: DataType>: Copy;
    /// The function pointer type used for write operations.
    type WriteFn<Data: Copy, DT: DataType>: Copy;
}

/// Indicates that this FakeRegister does not implement the corresponding read or write operation.
pub enum NoAccess {}
impl Access for NoAccess {
    type ReadFn<Data: Copy, DT: DataType> = ();
    type WriteFn<Data: Copy, DT: DataType> = ();
}

/// Indicates that this FakeRegister does implements the corresponding safe read or write
/// operation.
pub enum Safe {}
impl Access for Safe {
    type ReadFn<Data: Copy, DT: DataType> = fn(Data) -> LocalRegisterCopy<DT::Value, DT::LongName>;
    type WriteFn<Data: Copy, DT: DataType> = fn(Data, LocalRegisterCopy<DT::Value, DT::LongName>);
}

/// Indicates that this FakeRegister does implements the corresponding unsafe read or write
/// operation.
pub enum Unsafe {}
impl Access for Unsafe {
    type ReadFn<Data: Copy, DT: DataType> =
        unsafe fn(Data) -> LocalRegisterCopy<DT::Value, DT::LongName>;
    type WriteFn<Data: Copy, DT: DataType> =
        unsafe fn(Data, LocalRegisterCopy<DT::Value, DT::LongName>);
}

mod sealed {
    pub trait Access {}
}

impl sealed::Access for NoAccess {}
impl sealed::Access for Safe {}
impl sealed::Access for Unsafe {}

/// A fake register array, for use in unit tests.
///
/// ```
/// # use std::cell::Cell;
/// # use tock_registers::{mmio32_registers, FakeRegister, FakeRegisterArray};
/// # use tock_registers::{LocalRegisterCopy, Read, Write, Safe};
/// mmio32_registers! {
///     /// An array of registers that remember values written into them.
///     storage {
///         0 => scratch: [u8; 4] { Read, Write },
///     }
/// }
/// /// Fake version of the storage peripheral.
/// struct Fake([Cell<u8>; 4]);
/// impl<'f> storage::Interface for &'f Fake {
///     type scratch = FakeRegisterArray<Self, FakeRegister<&'f Cell<u8>, u8, Safe, Safe>, 4>;
///     fn scratch(self) -> Self::scratch {
///         FakeRegisterArray::new(self, |s, i| Some(
///             FakeRegister::new(s.0.get(i)?)
///                 .on_read(|c| LocalRegisterCopy::new(c.get()))
///                 .on_write(|c, v| c.set(v.get()))
///         ))
///     }
/// }
/// # fn main() {}
/// ```
///
/// Unlike FakeRegister, FakeRegisterArray is not limited to the Read and Write operations. You can
/// embed any element type in it (including a fake version of a register block).
#[cfg(feature = "register_types")]
#[derive(Clone, Copy)]
pub struct FakeRegisterArray<Data: Copy, Element: Copy, const LEN: usize> {
    data: Data,
    get: fn(Data, usize) -> Option<Element>,
}

#[cfg(feature = "register_types")]
impl<Data: Copy, Element: Copy, const LEN: usize> FakeRegisterArray<Data, Element, LEN> {
    /// Constructs a new FakeRegisterArray. Whenever the array is indexed (using
    /// [RegisterArray::get] or [RegisterArray::get_unchecked]), `get_fcn` is called and passed
    /// `data` and the index.
    ///
    /// Note that `get_fcn` must return `None` if the index is `>= LEN`. FakeRegisterArray does not
    /// perform the bounds check, because it is expected that most `get_fcn` implementations will
    /// need to perform that bounds check anyway (as they will likely be indexing into a Rust
    /// array).
    pub const fn new(data: Data, get_fcn: fn(Data, usize) -> Option<Element>) -> Self {
        Self { data, get: get_fcn }
    }
}

#[cfg(feature = "register_types")]
impl<Data: Copy, Element: Copy, const LEN: usize> RegisterArray<LEN>
    for FakeRegisterArray<Data, Element, LEN>
{
    type Element = Element;

    fn get(self, index: usize) -> Option<Element> {
        (self.get)(self.data, index)
    }
}
