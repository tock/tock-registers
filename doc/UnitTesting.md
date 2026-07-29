# Unit Testing

tock-registers is designed to enable you to unit test code that depends on
tock-registers.

[examples/rng.rs](../examples/rng.rs) is a complete testable implementation of
the interfaces described here.  It demonstrates a hardware definition and
associated test harness one might expect to see in a typical codebase.

## Description

Suppose you have hardware RNG peripheral with the following register interface:
```rust
use tock_registers::{mmio32_register_map, Mmio32, Read};

mmio32_register_map! {
    /// Registers for a hardware device that generates random numbers.
    pub rng {
        /// This register returns a new random value on every read. It can be read concurrently by
        /// multiple cores, returning separate random data on each core.
        0 => random_byte: u8 { Read },
    }
}
```

A typical pattern for drivers is to create a struct to hold an instance of a peripheral, i.e.:
```rust
/// Instance of an RNG (may be backed by hardware or mocks).
struct Rng<R: rng::Interface> {
    registers: R,
}

impl<R: rng::Interface> Rng<R> {
    pub const fn new(regs: R) -> Rng<R> {
        Rng { registers: regs }
    }
}
```

And then the driver presents a high-level interface by defining methods that
operate on the low-level register interface:
```rust
impl<R: rng::Interface> Rng<R> {
    /// A driver method that fills the provided buffer with random data.
    ///
    /// This method is unit testable: it can be used with either the real
    /// hardware or a fake/mock implementation of the hardware.
    pub fn getrandom(&self, buffer: &mut [u8]) {
        for byte in buffer {
            *byte = self.registers.random_byte().get();
        }
    }
}
```

Because the driver object is generic over the `rng::Interface` trait, it is
unit-testable by simply instantiating the driver with an alternative version of
the `rng::Interface` trait that emulates the hardware peripheral:
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

        // Create an instance of an `Rng` backed by mocked hardware.
        let fake_rng = FakeRng::default();
        let rng = Rng::new(&fake_rng);

        // Invoke the driver method (on the mocked hardware).
        rng.getrandom(&mut buffer);

        assert_eq!(buffer, [1, 2, 3]);
    }
}
```
