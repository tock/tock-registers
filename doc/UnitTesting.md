# Unit Testing

tock-registers is designed to enable you to unit test code that depends on
tock-registers.

Suppose you have a register interface and a driver that uses the register
interface:

```rust
use tock_registers::{mmio32_registers, Mmio32, Read};

mmio32_registers! {
    /// Registers for a hardware device that generates random numbers.
    pub rng {
        /// This register returns a new random value on every read. It can be read concurrently by
        /// multiple cores, returning separate random data on each core.
        0 => random_byte: u8 { Read },
    }
}

/// A driver for this hardware, which fills the provided buffer with random
/// data.
pub fn getrandom(buffer: &mut [u8]) {
    use rng::Interface;
    let mmio = Mmio32::from_addr(0x100);
    // Safety: We know this device exists at address 0x100, and can be access from multiple threads
    // with no issue.
    let registers = unsafe { rng::Real::new(mmio) };
    for byte in buffer {
        *byte = registers.random_byte().get();
    }
}
```

To make the driver unit-testable, split the portion that uses the registers
out into a separate function that is generic over the `rng::Interface` trait:

```rust
/// A driver for this hardware, which fills the provided buffer with random data. This function is
/// unit testable: it can be used with either the real hardware or a fake/mock implementation of
/// the hardware.
pub fn getrandom_impl<R: rng::Interface>(registers: R, buffer: &mut [u8]) {
    for byte in buffer {
        *byte = registers.random_byte().get();
    }
}

/// Driver usable with the real hardware. Fills the provided buffer with random data.
pub fn getrandom(buffer: &mut [u8]) {
    let mmio = Mmio32::from_addr(0x100);
    // Safety: We know this device exists at address 0x100, and can be access from multiple threads
    // with no issue.
    let registers = unsafe { rng::Real::new(mmio) };
    getrandom_impl(registers, buffer);
}
```

You can now create an alternative version of the Interface trait which emulates
the hardware peripheral:

```rust
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
        getrandom_impl(&FakeRng::default(), &mut buffer);
        assert_eq!(buffer, [1, 2, 3]);
    }
}
```

See [../examples/rng.rs] for the complete example.
