// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! # Addresses and Buses
//!
//! tock-registers supports many different types of registers, including:
//! 1. MMIO registers
//! 2. Several types of LiteX registers (these are MMIO, but have padding within the registers that
//!    depends on the chip's configuration and the register's data type)
//! 3. x86 port IO
//!
//! tock-registers calls each of these classes a "bus", as the mechanism used to access the
//! registers is different. The mechanism in which a register is addressed depends on the bus, so
//! each register address type corresponds to the bus. In other words:
//!
//! * The *type* of the address pointer corresponds to the bus.
//! * The *value* of the address pointer corresponds to the register on that bus.
//!
//! Therefore, all real register types ("real" here means it's not a fake/mock/stub for testing)
//! have a generic argument for their address type. That address type implements the `Address`
//! trait. In addition, address types should implement [`Bus<T>`] for every value type `T` that
//! they support.

use crate::DataType;
use core::marker::PhantomData;

/// Trait for addresses that can be offset without changing their type. See the module-level docs
/// for more information on address types.
pub trait Address: Copy {
    /// Adds the given offset to this address, returning an address pointing to a later location on
    /// the bus.
    /// # Safety
    /// `self` must point into a register span on the correct bus (based on `Self`). The entire
    /// range from `self` to the result (inclusive) must be in bounds of that register span. In
    /// particular, this must not "wrap around" the address space.
    unsafe fn byte_add(self, offset: usize) -> Self;
}

/// Address types should implement `Bus<T>` for each primitive type `T` that they support.
///
/// # Safety
/// PADDED_SIZE must be correct, as the generated code relies on PADDED_SIZE to calculate register
/// offsets.
pub unsafe trait Bus<T>: Address {
    /// The size that a value of type T takes in this bus' address space. This exists because LiteX
    /// buses have intra-register padding for some types.
    const PADDED_SIZE: usize;
}

/// An accessor for a span of registers. Every Real type implements this. This trait is used to
/// construct the register accessors, including fields of register blocks and elements of arrays.
///
/// # Safety
/// SIZE must be correct, as it is used to calculate register offsets.
pub unsafe trait Span: Copy {
    type Address: Address;
    /// Size this register span occupies in the address space. Depends on the address type.
    const SIZE: usize;

    /// Constructs an accessor for a register span.
    /// # Safety
    /// 1. `address` must point to register(s) on the bus corresponding to `Self::Address`.
    /// 2. The register(s)' definition (as provided to the
    ///    [`register_map`](crate::register_map) macro) must correctly describe the pointed-to
    ///    register(s).
    /// 3. The returned register span accessor must not be used in a way that causes data races.
    ///    The exact requirements depend on the hardware, but it's usually best to access a
    ///    register span from only one thread at a time.
    // The usual name for this constructor would be `new`. However, `const` is not supported in
    // traits, so the `const new` function for the Real types is an inherent method. The
    // `clippy::same_name_method` lint triggers on the redundant `new` functions, so to make this
    // work in projects that forbid `same_name_method` we use a different name instead.
    unsafe fn with_addr(address: Self::Address) -> Self;

    /// Type of this register with its bus wrapped in BorrowedBus.
    type Borrowed<'b>: Span<Address = BorrowedBus<'b, Self::Address>>;
}

/// An alias for `Bus<D::Value>`. Used so you don't have to write
/// `BusValue<<T as DataType>::Value>` (this simplifies the generated code quite a bit).
///
/// # Safety
/// PADDED_SIZE must be correct, as the generated code relies on PADDED_SIZE to calculate register
/// offsets.
pub unsafe trait DataTypeBus<D: DataType>: Bus<D::Value> {
    /// Alias for Bus::PADDED_SIZE.
    // Safety: Bus' safety condition is that Bus::PADDED_SIZE is correct.
    const PADDED_SIZE: usize = <Self as Bus<D::Value>>::PADDED_SIZE;
}
// Safety: The provided value for PADDED_SIZE is correct.
unsafe impl<D: DataType, T: Bus<D::Value>> DataTypeBus<D> for T {}

/// A Bus that has a lifetime attached with it. Returned by [`RegisterSender::borrow`].
#[derive(Clone, Copy)]
pub struct BorrowedBus<'b, A: Address> {
    address: A,
    // Makes BorrowedBus covariant with respect to 'b and neither Send nor Sync.
    _phantom: PhantomData<&'b *mut ()>,
}

impl<'b, A: Address> BorrowedBus<'b, A> {
    /// Constructs a new BorrowedBus. Note that in general, BorrowedBus should be constructed by
    /// using [`RegisterSender`], but you can use this to build your own RegisterSender-like
    /// abstraction.
    pub const fn new(address: A) -> Self {
        Self {
            address,
            _phantom: PhantomData,
        }
    }

    /// Returns this bus' address.
    pub fn address(self) -> A {
        self.address
    }
}

impl<'b, A: Address> Address for BorrowedBus<'b, A> {
    unsafe fn byte_add(self, offset: usize) -> Self {
        Self {
            // Safety: This is the same Bus as A, even though it is a distinct type. Therefore, the
            // caller of this function already guaranteed that self.address is the correct type,
            // and promised that adding `offset` will remain within the register span.
            address: unsafe { self.address.byte_add(offset) },
            _phantom: PhantomData,
        }
    }
}

/// Safety: We are the same bus as A, so the padded size of each register type is the same.
unsafe impl<'b, T, A: Address + Bus<T>> Bus<T> for BorrowedBus<'b, A> {
    const PADDED_SIZE: usize = A::PADDED_SIZE;
}

/// A utility for sharing a register span between threads.
///
/// Unlike the `Real` structs, this implements [`Send`], so it can be moved between threads (using
/// something like a channel or mutex). To access the register span, call
/// [`borrow`](RegisterSender::borrow) to get a temporary handle to the register span.
///
/// Note that a bus can choose whether RegisterSender works with it by deciding whether to
/// implement Send.
pub struct RegisterSender<R: Span>
where
    R::Address: Send,
{
    address: R::Address,
    // Makes RegisterSender !Sync (we have a unsafe Send impl to make it Send despite this bound).
    _phantom: PhantomData<*mut ()>,
}

// Safety: RegisterSender is not Copy or Sync, so only one thread can have an active borrow at a
// time. That means the hierarchy of Real<> structs can only be accessed from one thread at a time.
unsafe impl<R: Span> Send for RegisterSender<R> where R::Address: Send {}

impl<R: Span> RegisterSender<R>
where
    R::Address: Send,
{
    /// Constructs a new RegisterSender for the given register span.
    /// # Safety
    /// 1. `address` must point to register(s) on the bus corresponding to `Self::Address`.
    /// 2. The register(s)' definition (as provided to the
    ///    [`register_map`](crate::register_map) macro) must correctly describe the pointed-to
    ///    register(s).
    /// 3. Nothing other than handles returned by [`borrow`](Self::borrow) (and handles derived
    ///    from them) may be used to access this register span.
    pub unsafe fn new(address: R::Address) -> Self {
        Self {
            address,
            _phantom: PhantomData,
        }
    }

    /// Borrows this `RegisterSender`, returning a handle that can be used to access the
    /// register(s) it points to. Because the returned handle borrows this `RegisterSender`, the
    /// returned handles must be dropped before the `RegisterSender` can be sent to another thread.
    pub fn borrow(&self) -> R::Borrowed<'_> {
        let borrowed_bus = BorrowedBus::new(self.address);
        // Safety: All of the requirements for Span::with_addr() were met by the caller when they
        // called `RegisterSender::new`.
        unsafe { R::Borrowed::with_addr(borrowed_bus) }
    }
}
