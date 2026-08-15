// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Example of a safe DMA abstraction crate. This is obviously simplified relative to a real DMA
//! crate, e.g. it only supports 'static buffers.

use core::cell::Cell;
use tock_registers::{Bus, DataType, Register};

pub trait UnsafeWrite: Register {
    /// # Safety: The safety invariants are hardware-specific, and depend on which register this
    /// is.
    unsafe fn set(self, value: <Self::DataType as DataType>::Value);
}
#[macro_export]
macro_rules! UnsafeWrite {
    (real_impl, $name:ident, $datatype:ty,, $($rest:tt)*) => {
        impl<B: Bus + tock_registers::BusWrite<<$datatype as tock_registers::DataType>::Value>>
            UnsafeWrite for $name<B>
        {
            unsafe fn set(self, value: <$datatype as tock_registers::DataType>::Value) {
                // Safety: The caller assured this register accessor points at a register on bus B
                // with value type $datatype::Value that is safe to write. The code that
                // constructed `self` guaranteed that they would avoid data races (precondition of
                // Self::new). The caller followed the hardware's safety invariants.
                unsafe { self.address.write(value) }
            }
        }
    };
    ($($unknown:tt)*) => {};
}

// Operation for a DMA enable register. Assumes that 0 == not running and 1 == enabled.
pub trait DmaEnable: Register {
    /// Performs the fence necessary to let DMA access this buffer, then starts the DMA operation.
    ///
    /// # Safety: The address and length registers must point to a buffer which the hardware may
    /// read and write.
    unsafe fn start(self);

    /// Checks if this DMA operation is ongoing.
    fn is_running(self) -> bool;
}
#[macro_export]
macro_rules! DmaEnable {
    (real_impl, $name:ident, $datatype:ty,, $($rest:tt)*) => {
        impl<
                B: Bus
                    + $crate::safe_dma::BusDmaEnable<
                        <$datatype as tock_registers::DataType>::Value,
                    >,
            > DmaEnable for $name<B>
        {
            unsafe fn start(self) {
                // Safety: The caller assured this register accessor points at a register on bus B
                // with value type $datatype::Value that is safe to write. The code that
                // constructed `self` guaranteed that they would avoid data races (precondition of
                // Self::new). The caller guarantees that the address and length registers are set
                // correctly.
                unsafe { self.address.start() }
            }

            fn is_running(self) -> bool {
                self.address.get() != 0
            }
        }
    };
    ($($unknown:tt)*) => {};
}
/// # Safety
/// `get` must never return `0` unless the corresponding DMA operation has been stopped.
pub unsafe trait BusDmaEnable<T>: Bus<T> {
    /// # Safety
    /// There must be a writable register of type T at `pointer`. The caller is responsible for
    /// avoiding data races. The DMA address and length register but point to a buffer the hardware
    /// can read and write.
    unsafe fn start(self);

    /// Gets the current value of the enable register. Note that this must be an Acquire-ordered
    /// operation, so that DMA writes are ordered-before anything the Rust code does after getting
    /// a `0` value.
    fn get(self) -> T;
}
/// Safety: get() must be correct.
#[cfg(target_arch = "x86_64")]
unsafe impl BusDmaEnable<u8> for tock_registers::Mmio64 {
    unsafe fn start(self) {
        let ptr = self.address().as_ptr();
        // Safety: We know this is a u8 register that we can write 1 to. This assumes the register
        // is located in memory for which DMA is cache-coherent (so no memory fence instruction is
        // required). This functions as a release fence, guaranteeing the buffer is fully written
        // before the DMA operation is started.
        #[cfg(not(miri))]
        unsafe {
            core::arch::asm!("mov BYTE PTR [{}],1", in(reg) ptr);
        }
        #[cfg(miri)]
        unsafe { &*ptr.cast::<core::sync::atomic::AtomicU8>() }
            .store(1, core::sync::atomic::Ordering::Release);
    }

    fn get(self) -> u8 {
        let val: u16;
        let ptr = self.address().as_ptr();
        // Safety: We know this points to a u8 DMA enable register, so we can read it. This is an
        // Acquire-ordered operation, so a read of 0 is sequenced after the DMA operation's writes.
        #[cfg(not(miri))]
        unsafe {
            core::arch::asm!("movzx {0:x}, BYTE PTR [{1}]", out(reg) val, in(reg) ptr)
        };
        #[cfg(miri)]
        {
            use core::sync::atomic::{AtomicU8, Ordering};
            val = unsafe { &*ptr.cast::<AtomicU8>() }
                .load(Ordering::Acquire)
                .into();
        }
        val as u8
    }
}

/// Generates the DmaManager type for a peripheral.
// This is currently hardcoded to a single DMA channel, a real DMA abstraction crate would need to
// support multiple DMA channels.
#[macro_export]
macro_rules! dma_manager {
    (
        // Visibility of the DmaManager struct.
        $visibility:vis
        // The name of the DmaManager struct to be generated.
        $struct_name:ident,
        // The register map module.
        // Note: this should actually be a path, but macro_rules! macros don't seem to be able to
        // append to a path (to generate the Interface trait reference). A procedural macro could
        // do that, but for simplicity we'll just use ident for this example.
        $map:ident,
        // The name of the address register. Must have datatype *mut u8 and be UnsafeWrite.
        $address:ident,
        // The name of the length register. Must have datatype usize and be UnsafeWrite.
        $len:ident,
        // The name of the enable register. Must have datatype u8 and be DmaEnable.
        $enable:ident $(,)?
    ) => {
        $visibility struct $struct_name<R> {
            // The DMA buffer that is current in use, or None if no DMA operation is ongoing.
            // Safety invariant: If Some(), this is identical to a `&'static mut [u8]` with one
            // exception: it may alias with an ongoing operation by this DMA channel.
            buffer: core::cell::Cell<Option<core::ptr::NonNull<[u8]>>>,
            // The registers for the peripheral this DMA manager supports.
            registers: R,
        }

        impl<R: $map::Interface> $struct_name<R> {
            /// Constructs this DMA manager.
            ///
            /// # Safety
            /// `registers` must point to a valid peripheral instance. The returned struct must be
            /// the only thing that performs `unsafe` operations on this peripheral's DMA
            /// registers.
            pub unsafe fn new(registers: R) -> Self {
                Self {
                    // Safety: This is not Some(_) so no safety invariant applies.
                    buffer: core::cell::Cell::new(None),
                    registers,
                }
            }

            #[cfg_attr(not(test), allow(dead_code))]
            pub fn new_fake() -> (R, Self) where R: Clone + Default {
                let registers = R::default();
                (registers.clone(), Self {
                    // Safety: This is not Some(_) so no safety invariant applies.
                    buffer: core::cell::Cell::new(None),
                    registers,
                })
            }

            /// Returns a copy of the peripheral's register handle.
            // This example doesn't use it, but this is how the driver would get access to the
            // registers for manipulating non-DMA registers.
            #[allow(dead_code)]
            pub fn registers(&self) -> R where R: Clone {
                self.registers.clone()
            }

            /// Starts the DMA operation with the given buffer.
            pub fn start(&self, buffer: &'static mut [u8]) {
                let enable_reg = self.registers.$enable();
                if enable_reg.is_running() {
                    panic!("DMA operation already ongoing");
                }
                let buffer = core::ptr::NonNull::from(buffer);
                let address = buffer.as_ptr().cast();
                let len = buffer.len();
                let address_reg = self.registers.$address();
                let len_reg = self.registers.$len();
                unsafe {
                    // Safety: The DMA operation is disabled.
                    address_reg.set(address);
                    // Safety: The DMA operation is disabled.
                    len_reg.set(len);
                }
                // Safety: The address and len point to a static-lifetime mutable buffer. We know
                // that when this function was called, `buffer` was the only live reference to the
                // buffer (because it was a &mut reference), and we have converted it into a
                // NonNull pointer, so there are no live references pointing at the buffer.
                unsafe {
                    enable_reg.start();
                }
                // Safety: This was converted directly from a valid &'static mut [u8].
                self.buffer.set(Some(buffer));
            }

            /// Returns the DMA buffer, if the DMA operation has completed.
            pub fn get_buffer(&self) -> Option<&'static mut [u8]> {
                if self.registers.$enable().is_running() {
                    return None;
                }
                // Safety: We already checked the DMA operation is ongoing, so by `self.buffer`'s
                // safety invariant nothing aliases with `b`. By `self.buffer`'s safety invariant,
                // `b` meets all other requirements to be converted back to a `&'static mut [u8]`.
                self.buffer.take().map(|mut b| unsafe { b.as_mut() })
            }
        }
    };
}

/// A fake register that implements UnsafeWrite by writing the passed value into the given Cell.
pub struct FakeUnsafeWrite<'c, T: DataType>(&'c Cell<T::Value>);

impl<'c, T: DataType> FakeUnsafeWrite<'c, T> {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(value: &'c Cell<T::Value>) -> Self {
        FakeUnsafeWrite(value)
    }
}

impl<'c, T: DataType> Clone for FakeUnsafeWrite<'c, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'c, T: DataType> Copy for FakeUnsafeWrite<'c, T> {}

impl<'c, T: DataType> Register for FakeUnsafeWrite<'c, T> {
    type DataType = T;
}

impl<'c, T: DataType> UnsafeWrite for FakeUnsafeWrite<'c, T> {
    unsafe fn set(self, value: T::Value) {
        self.0.set(value)
    }
}
