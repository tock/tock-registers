// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2025.

//! The buses provided by tock-registers (Mmio32/Mmio64 and their nullable variants) implement
//! BusRead/BusWrite on every platform, including platforms that they don't match. That is a
//! backwards-compatibility feature that is necessary to make it practical to migrate legacy code
//! onto [register_map]. However, other buses do NOT need to do that. This example shows how a bus
//! can support operations on ONLY the target architecture it is meant for, and serves as a test to
//! make sure that [register_map]'s generated code does not break that functionality.

use core::marker::PhantomData;
use tock_registers::{register_map, Address, Bus, Read};

/// A Bus for x86's port I/O. This only implements byte-sized reads to limit the size of this
/// example.
#[derive(Clone, Copy)]
pub struct Port {
    number: u16,
    _phantom: PhantomData<*mut ()>,
}

impl Port {
    /// Returns a new Port with the given number.
    // This is optional as well, but kept for this example because we use it in main().
    pub fn new(number: u16) -> Self {
        Self {
            number,
            _phantom: PhantomData,
        }
    }
}

// `register_map!` emits offset tests that run on every architecture, including unit test
// environments. Those test depend on the Bus trait, which has Address as a supertrait. Therefore,
// we have to implement Address regardless of the target.
impl Address for Port {
    unsafe fn byte_add(self, offset: usize) -> Port {
        Port {
            number: self.number + offset as u16,
            _phantom: PhantomData,
        }
    }
}

// Safety: Byte registers are 1 apart in I/O ports.
unsafe impl Bus<u8> for Port {
    const PADDED_SIZE: usize = 1;
}

#[cfg(target_arch = "x86")]
impl tock_registers::BusRead<u8> for Port {
    unsafe fn read(self) -> u8 {
        panic!("This is an example, lets not perform a register access for real.");
    }
}

register_map! {
    #[bus(Port)]
    registers {
        0 => status: u8 { Read },
    }
}

fn main() {
    let _real = unsafe { registers::Real::new(Port::new(1)) };
    // This won't compile on non-x86 architectures, because `Real` does not implement Interface.
    #[cfg(target_arch = "x86")]
    _real.read();
}
