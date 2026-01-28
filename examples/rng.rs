// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use tock_registers::{mmio32_registers, Read};

mmio32_registers! {
    /// Registers for a hardware device that generates random numbers.
    pub rng {
        /// This register returns a new random value on every read.
        0 => random_byte: u8 { Read },
    }
}

/// A driver for this hardware, which fills the provided buffer with random
/// data.
pub fn getrandom<R: rng::Interface>(registers: R, buffer: &mut [u8]) {
    for byte in buffer {
        *byte = registers.random_byte().get();
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
        let mut buffer = [0; 3];
        getrandom(&FakeRng::default(), &mut buffer);
        assert_eq!(buffer, [1, 2, 3]);
    }
}

fn main() {}
