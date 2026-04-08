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
        /// This register returns a new random value on every read.
        0 => random_byte: u8 { Read },
    }
}

/// A driver for this hardware, which fills the provided buffer with random
/// data.
pub fn getrandom(buffer: &mut [u8]) {
    use { core::ptr::without_provenance_mut, rng::Interface as _ };
    // Safety: We know this CPU has the RNG peripheral at address 0x10000.
    let registers = unsafe { rng::Real::<Mmio32>::new(Mmio32(without_provenance_mut(0x10000))) };
    for byte in buffer {
        *byte = registers.random_byte().get();
    }
}
```

To make the driver unit-testable, make it generic over the `Interface` trait:

```rust
pub fn getrandom<R: rng::Interface>(registers: R, buffer: &mut [u8]) {
    for byte in buffer {
        *byte = registers.random_byte().get();
    }
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
        getrandom(&FakeRng::default(), &mut buffer);
        assert_eq!(buffer, [1, 2, 3]);
    }
}
```

See [examples/rng.rs] for the complete example.

Of course, you also need the "real" (non-unit-test) version of this code. You
can add a new function to do that:

```rust
/// Fills the provided buffer with random / data.
pub fn getrandom_real(buffer: &mut [u8]) {
    use { core::ptr::without_provenance_mut, rng::Interface as _ };
    // Safety: We know this CPU has the RNG peripheral at address 0x10000.
    let registers = unsafe { rng::Real::<Mmio32>::new(Mmio32(without_provenance_mut(0x10000))) };
    getrandom(registers, buffer);
}
```

Now code that requires the real hardware can use `getrandom_real`, while code
that needs to remain unit-testable can use `getrandom`.
