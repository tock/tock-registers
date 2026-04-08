// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use std::cell::Cell;
use tock_registers::{
    mmio32_registers, FakeRegister, FakeRegisterArray, LocalRegisterCopy, NoAccess, Read,
    RegisterArray, Safe, Write,
};
use {array_demo::Interface as _, variable_increment::Interface as _};

mmio32_registers! {
    variable_increment {
        /// The amount to increment the shared counter by when `counter` is read.
        0 => increment: u8 { Read, Write },
        /// Accessor for the shared counter. Each time this is read, the shared counter is
        /// incremented by `increment`.
        1 => counter: u8 { Read },
    },

    /// A demonstration of register arrays. This type contains a single, shared counter that wraps
    /// modulo 256.
    array_demo {
        /// Reading from the array with index i increments the counter by i+1, wrapping modulo 256.
        0 => counter: [u8; 3] { Read },
        /// A set of variable-counter-incrementers. These all access the same shared counter, but
        /// have separate `increment` variables.
        3 => incrementers: [variable_increment; 3],
    },
}

#[derive(Default)]
struct Fake {
    single_counter: Cell<u8>,
    /// The values of the `increment` registers for the variable incrementers.
    increments: [Cell<u8>; 3],
}

impl<'f> array_demo::Interface for &'f Fake {
    type counter = FakeRegisterArray<Self, FakeRegister<(Self, u8), u8, Safe, NoAccess>, 3>;
    fn counter(self) -> FakeRegisterArray<Self, FakeRegister<(Self, u8), u8, Safe, NoAccess>, 3> {
        FakeRegisterArray::new(self, |s, index| {
            if index >= 3 {
                return None;
            };
            Some(
                FakeRegister::new((s, index as u8 + 1)).on_read(|(s, increment)| {
                    let out = s.single_counter.get();
                    s.single_counter.set(out.wrapping_add(increment));
                    LocalRegisterCopy::new(out)
                }),
            )
        })
    }

    type incrementers = FakeRegisterArray<Self, FakeIncrement<'f>, 3>;
    fn incrementers(self) -> FakeRegisterArray<Self, FakeIncrement<'f>, 3> {
        FakeRegisterArray::new(self, |s, index| {
            Some(FakeIncrement {
                counter: &s.single_counter,
                increment: s.increments.get(index)?,
            })
        })
    }
}

#[derive(Clone, Copy)]
struct FakeIncrement<'f> {
    counter: &'f Cell<u8>,
    increment: &'f Cell<u8>,
}

impl<'f> variable_increment::Interface for FakeIncrement<'f> {
    type increment = FakeRegister<&'f Cell<u8>, u8, Safe, Safe>;
    fn increment(self) -> Self::increment {
        FakeRegister::new(self.increment)
            .on_read(|i| LocalRegisterCopy::new(i.get()))
            .on_write(|i, v| i.set(v.get()))
    }

    type counter = FakeRegister<Self, u8, Safe, NoAccess>;
    fn counter(self) -> Self::counter {
        FakeRegister::new(self).on_read(|s| {
            let out = s.counter.get();
            s.counter.set(out.wrapping_add(s.increment.get()));
            LocalRegisterCopy::new(out)
        })
    }
}

#[test]
fn counter() {
    let fake = Fake::default();
    let element = fake.counter().get(0).unwrap();
    assert_eq!(element.get(), 0);
    assert_eq!(element.get(), 1);
    let element = fake.counter().get(1).unwrap();
    assert_eq!(element.get(), 2);
    assert_eq!(element.get(), 4);
    let element = fake.counter().get(2).unwrap();
    assert_eq!(element.get(), 6);
    assert_eq!(element.get(), 9);
    assert!(fake.counter().get(3).is_none());
    assert_eq!(fake.counter().get(0).unwrap().get(), 12);
    assert_eq!(fake.counter().get(1).unwrap().get(), 13);
    assert_eq!(fake.counter().get(2).unwrap().get(), 15);
    assert!(fake.counter().get(3).is_none());
    assert_eq!(fake.counter().get(0).unwrap().get(), 18);
    assert_eq!(fake.counter().get(1).unwrap().get(), 19);
    assert_eq!(fake.counter().get(2).unwrap().get(), 21);
    let [incrementer0, incrementer1, incrementer2] =
        [0, 1, 2].map(|i| fake.incrementers().get(i).unwrap());
    incrementer0.increment().set(3);
    assert_eq!(fake.incrementers().get(0).unwrap().counter().get(), 24);
    assert_eq!(fake.incrementers().get(1).unwrap().counter().get(), 27);
    assert_eq!(fake.incrementers().get(2).unwrap().counter().get(), 27);
    assert!(fake.incrementers().get(3).is_none());
    incrementer1.increment().set(4);
    assert_eq!(fake.incrementers().get(0).unwrap().counter().get(), 27);
    assert_eq!(fake.incrementers().get(1).unwrap().counter().get(), 30);
    assert_eq!(fake.incrementers().get(2).unwrap().counter().get(), 34);
    assert!(fake.incrementers().get(3).is_none());
    incrementer2.increment().set(5);
    assert_eq!(fake.incrementers().get(0).unwrap().counter().get(), 34);
    assert_eq!(fake.incrementers().get(1).unwrap().counter().get(), 37);
    assert_eq!(fake.incrementers().get(2).unwrap().counter().get(), 41);
    assert!(fake.incrementers().get(3).is_none());
    assert_eq!(fake.counter().get(0).unwrap().get(), 46);
}
