// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

/// This file is a companion to [Unit Testing](doc/UnitTesting.md).
///
/// This is a minimal implementation designed to explain the unit testing hooks
/// with minimal complications. For a more complete example of a typical
/// hardware instantiation and its associated unit testing setup, please
/// consult [the full RNG example](examples/rng_full.rs).

use tock_registers::{mmio32_register_map, Mmio32, Read};

// This defines the peripheral interface. This interface definition is
// independent of underlying implementation (i.e., real hardware or a mock).
mmio32_register_map! {
    /// Registers for a hardware device that generates random numbers.
    pub rng {
        /// This register returns a new random value on every read. It can be
        /// read concurrently by multiple cores, returning separate random data
        /// on each core.
        0 => random_byte: u8 { Read },
    }
}

/// A driver function that exercises the peripheral interface.
///
/// This method provides an interface to use the peripheral to fill the
/// provided buffer with random data.
///
/// This function is unit testable: it can be used with either the real
/// hardware or a fake/mock implementation of the hardware.
pub fn getrandom<R: rng::Interface>(registers: R, buffer: &mut [u8]) {
    for byte in buffer {
        *byte = registers.random_byte().get();
    }
}

/// A function that executes the driver over real hardware.
///
/// This method creates a reference to a real hardware peripheral, and then
/// invokes the driver function on that hardware.
///
/// **Note:** This function is included as an example to show hardware
/// creation, but is not actually runnable; no such RNG exists at the MMIO
/// address specified here.
pub fn getrandom_via_hardware(buffer: &mut [u8]) {
    // Create a (generic) pointer to a peripheral on a 32-bit memory bus.
    const RNG_HARDWARE_ADDRESS: usize = 0x100;
    let mmio = Mmio32::from_addr(RNG_HARDWARE_ADDRESS);

    // Create an `Rng` instance backed by MMIO.
    //
    // Safety: We know this device exists at address RNG_HARDWARE_ADDRESS, and
    // can be accessed from multiple threads with no issue.
    let registers = unsafe { rng::Real::new(mmio) };

    // Invoke the driver method on the hardware peripheral.
    getrandom(registers, buffer);
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;
    use tock_registers::{FakeRegister, NoAccess, Safe};

    /// A fake RNG, which produces an incrementing output. We implement
    /// `Interface` on references to `FakeRng` (this mirrors the real
    /// implementation, which is implemented on a type that points to the
    /// real hardware).
    #[derive(Default)]
    struct FakeRng {
        state: Cell<u8>,
    }
    impl rng::Interface for &FakeRng {
        type random_byte = FakeRegister<Self, u8, Safe, NoAccess>;
        fn random_byte(self) -> FakeRegister<Self, u8, Safe, NoAccess> {
            FakeRegister::new(self).on_read(|this| {
                let next = this.state.get().wrapping_add(1);
                this.state.set(next);
                next
            })
        }
    }

    #[test]
    fn getrandom_test() {
        let mut buffer = [0; 3];

        // Create a mocked peripheral and invoke the driver method
        getrandom(&FakeRng::default(), &mut buffer);

        assert_eq!(buffer, [1, 2, 3]);
    }
}

fn main() {}
