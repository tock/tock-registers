// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use tock_registers::{mmio32_register_map, Mmio32, Read};

mmio32_register_map! {
    /// Registers for a hardware device that generates random numbers.
    pub rng {
        /// This register returns a new random value on every read. It can be read concurrently by
        /// multiple cores, returning separate random data on each core.
        0 => random_byte: u8 { Read },
    }
}

/// Method to create and use an instance of Rng backed by real hardware.
///
/// Normally, this logic would be external to the driver, e.g. in some board
/// hardware setup method or similar.
///
/// None of this code is exercised when running tests, rather, it's included
/// as a reference for a minimal complete example.
pub fn create_and_use_rng<R: rng::Interface>() {
    // MMIO address of the hardware peripheral (e.g., from a datasheet).
    const RNG_HARDWARE_ADDRESS: usize = 0x100 as usize;

    // Create Rng Instance
    let mmio = Mmio32::from_addr(RNG_HARDWARE_ADDRESS);
    // Safety: We know this device exists at address RNG_HARDWARE_ADDRESS, and
    // can be accessed from multiple threads with no issue.
    let registers = unsafe { rng::Real::new(mmio) };
    let rng = Rng::new(registers);

    // Use the hardware Rng
    let mut buffer = [0; 3];
    rng.getrandom(&mut buffer);
}

/// Instance of an RNG (may be backed by hardware or mocks).
struct Rng<R: rng::Interface> {
    registers: R,
}

impl<R: rng::Interface> Rng<R> {
    pub const fn new(regs: R) -> Rng<R> {
        Rng { registers: regs }
    }

    /// A driver method that fills the provided buffer with random data.
    ///
    /// This function is unit testable: it can be used with either the real
    /// hardware or a fake/mock implementation of the hardware.
    pub fn getrandom(&self, buffer: &mut [u8]) {
        for byte in buffer {
            *byte = self.registers.random_byte().get();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;
    use tock_registers::{FakeRegister, NoAccess, Safe};

    /// A fake RNG, which produces an incrementing output. We implement Interface on references to
    /// FakeRng (this mirrors the real implementation, which is implemented on a type that points
    /// to the real hardware).
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
        let fake_rng = FakeRng::default();
        let rng = Rng::new(&fake_rng);

        let mut buffer = [0; 3];
        rng.getrandom(&mut buffer);
        assert_eq!(buffer, [1, 2, 3]);
    }
}

fn main() {}
