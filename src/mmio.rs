// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use crate::{Address, Bus, BusRead, BusWrite};
use core::ptr::{read_volatile, write_volatile, NonNull};

/// Macro to declare the Mmio* structs and provide a few impls for each.
macro_rules! mmio_structs {
    [$($(#[$docs:meta])* $name:ident($storage:ty))*] => {$(
        $(#[$docs])*
        #[derive(Clone, Copy)]
        pub struct $name($storage);

        impl $name {
            /// Returns a new MMIO address containing the given pointer.
            pub const fn new(ptr: $storage) -> Self {
                Self(ptr)
            }
        }

        impl Address for $name {
            unsafe fn byte_add(self, offset: usize) -> Self {
                // Safety: The safety requirements of Address::byte_add require self + offset to
                // remain within this register span and not wrap.
                Self(unsafe { self.0.byte_add(offset) })
            }
        }

        // Safety: Mmio* does not expose safe operations to access registers on its own. Instead,
        // it is used through two types:
        // 1. The Real<> structs, which are always !Send + !Sync
        // 2. RegisterSender<>, which is Send if the bus is Send but which guarantees that only one
        //    thread can access the registers at a time. Making Mmio* Send enables RegisterSender<>
        //    to work with it.
        unsafe impl Send for $name {}
    )*}
}

mmio_structs! {
    /// MMIO register bus for 32-bit systems.
    Mmio32(NonNull<()>)

    /// MMIO register bus for 64-bit systems.
    Mmio64(NonNull<()>)

    /// MMIO register bus for 32-bit systems that can point to a register at the 0 address.
    Mmio32Nullable(*mut ())

    /// MMIO register bus for 64-bit systems that can point to a register at the 0 address.
    Mmio64Nullable(*mut ())
}

/// Macro to provide the with_addr functions for the non-nullable Mmio* structs.
macro_rules! with_addr_nonnull {
    [$($name:ident)*] => {$(impl $name {
        /// Constructs a new MMIO register bus pointing to the given address. If you have an
        /// address from a datasheet and want to access it as a register struct, this is probably
        /// the method you want.
        ///
        /// The MMIO register bus is created without provenance. That is fine for accessing MMIO
        /// memory, but it means it cannot be used to access Rust memory. If you want to access
        /// Rust memory (i.e. if this is pointing somewhere other than MMIO memory) then you must
        /// use [`new`](Self::new) to construct this bus instead.
        ///
        /// # Panics
        /// Panics if the provided address is null, and panics during const evaluation if the
        /// pointer cannot be determined to be null or not. If you want to access a register at
        /// address 0, use one of the nullable MMIO structs.
        #[track_caller]
        pub const fn with_addr(addr: usize) -> Self {
            // TODO: Once the MSRV reaches:
            // 1.84: replace the pointer cast with core::ptr::without_provenance_mut
            // 1.85: replace NonNull::new_unchecked with NonNull::new
            // 1.89: replace NonNull::new/new_unchecked with NonNull::without_provenance (this
            //       obsoletes the above two steps).
            assert!(addr != 0);
            let ptr = addr as *mut ();
            // Safety: We've already confirmed that the address is not null (ptr::null() documents
            // that null pointers have address 0), and we have not changed the address, so ptr is
            // not null.
            Self(unsafe { NonNull::new_unchecked(ptr) })
        }
    })*}
}

with_addr_nonnull![Mmio32 Mmio64];

/// Macro to provide the with_addr functions for the Mmio*Nullable structs.
macro_rules! with_addr_nullable {
    [$($name:ident)*] => {$(impl $name {
        /// Constructs a new MMIO register bus pointing to the given address. If you have an
        /// address from a datasheet and want to access it as a register struct, this is probably
        /// the method you want.
        ///
        /// The MMIO register bus is created without provenance. That is fine for accessing MMIO
        /// memory, but it means it cannot be used to access Rust memory. If you want to access
        /// Rust memory (i.e. if this is pointing somewhere other than MMIO memory) then you must
        /// use [`new`](Self::new) to construct this bus instead.
        pub const fn with_addr(addr: usize) -> Self {
            // TODO: Once the MSRV reaches 1.84, replace this cast with
            // core::ptr::without_provenance_mut.
            Self(addr as *mut ())
        }
    })*}
}

with_addr_nullable![Mmio32Nullable Mmio64Nullable];

// TODO: Implement a arm64_secure_vm feature that uses confidential-vm-safe instructions:
// https://github.com/google/safe-mmio/blob/main/src/aarch64_mmio.rs
// https://github.com/rust-lang/rust/issues/131894
// The code is structured so we should be able to swap out the definition of bus_op_impls! without
// changing any other part of the code.
/// Macro to implement BusRead/BusWrite for the Mmio* structs.
macro_rules! bus_op_impls {
    [$nonnull:ident $nullable:ident $value:ty] => {
        impl BusRead<$value> for $nonnull {
            unsafe fn read(self) -> $value {
                // BusRead::read's preconditions guarantee that a readable register with value type
                // $value exists at address self.0, and the caller is responsible for avoiding data
                // races and satisfying any other unsafe invariants of the register.
                unsafe { self.0.cast().read_volatile() }
            }
        }
        impl BusRead<$value> for $nullable {
            unsafe fn read(self) -> $value {
                // BusRead::read's preconditions guarantee that a readable register with value type
                // $value exists at address self.0, and the caller is responsible for avoiding data
                // races and satisfying any other unsafe invariants of the register.
                unsafe { read_volatile(self.0.cast_const().cast()) }
            }
        }
        impl BusWrite<$value> for $nonnull {
            unsafe fn write(self, value: $value) {
                // BusRead::write's preconditions guarantee that a writable register with value
                // type $value exists at address self.0, and the caller is responsible for avoiding
                // data races and satisfying any other unsafe invariants of the register.
                //unsafe { write_volatile(self.0.cast().as_ptr(), value) }
                unsafe { self.0.cast().write_volatile(value) }
            }
        }
        impl BusWrite<$value> for $nullable {
            unsafe fn write(self, value: $value) {
                // BusRead::write's preconditions guarantee that a writable register with value
                // type $value exists at address self.0, and the caller is responsible for avoiding
                // data races and satisfying any other unsafe invariants of the register.
                unsafe { write_volatile(self.0.cast(), value) }
            }
        }
    }
}

/// Macro to implement the Bus traits for the Mmio* structs.
macro_rules! bus_impls {
    [$nonnull:ident, $nullable:ident, $value:ty, $size:literal] => {
        /// Safety: All the bus_impls! invocations have the correct size.
        unsafe impl Bus<$value> for $nonnull {
            const PADDED_SIZE: usize = $size;
        }
        /// Safety: All the bus_impls! invocations have the correct size.
        unsafe impl Bus<$value> for $nullable {
            const PADDED_SIZE: usize = $size;
        }
        bus_op_impls![$nonnull $nullable $value];
    }
}

bus_impls!(Mmio32, Mmio32Nullable, u8, 1);
bus_impls!(Mmio32, Mmio32Nullable, u16, 2);
bus_impls!(Mmio32, Mmio32Nullable, u32, 4);
bus_impls!(Mmio32, Mmio32Nullable, u64, 8);
bus_impls!(Mmio32, Mmio32Nullable, usize, 4);
bus_impls!(Mmio64, Mmio64Nullable, u8, 1);
bus_impls!(Mmio64, Mmio64Nullable, u16, 2);
bus_impls!(Mmio64, Mmio64Nullable, u32, 4);
bus_impls!(Mmio64, Mmio64Nullable, u64, 8);
bus_impls!(Mmio64, Mmio64Nullable, u128, 16);
bus_impls!(Mmio64, Mmio64Nullable, usize, 8);
