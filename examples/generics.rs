// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Demonstrates registers!'s support for generic buses and operations (and doubles as a test that
//! registers!'s generated code compiles in those cases).

use std::fmt::Debug;
use std::ptr::{read_volatile, NonNull};
use tock_registers::{registers, Address, Bus, BusRead, Read, RegisterArray, Span};

// -----------------------------------------------------------------------------
// Partial implementation of LiteX buses, to test registers!'s support for
// generic buses.
// -----------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct LiteX<const C: u8, const B: u8>(NonNull<()>);

impl Address for LiteX<8, 32> {
    unsafe fn byte_add(self, offset: usize) -> Self {
        Self(unsafe { self.0.byte_add(offset) })
    }
}
impl Address for LiteX<32, 32> {
    unsafe fn byte_add(self, offset: usize) -> Self {
        Self(unsafe { self.0.byte_add(offset) })
    }
}
unsafe impl Bus<u8> for LiteX<8, 32> {
    const PADDED_SIZE: usize = 4;
}
unsafe impl Bus<u8> for LiteX<32, 32> {
    const PADDED_SIZE: usize = 4;
}
impl BusRead<u8> for LiteX<8, 32> {
    unsafe fn read(self) -> u8 {
        unsafe { read_volatile(self.0.as_ptr().cast_const().cast()) }
    }
}
impl BusRead<u8> for LiteX<32, 32> {
    unsafe fn read(self) -> u8 {
        unsafe { read_volatile(self.0.as_ptr().cast_const().cast()) }
    }
}

// -----------------------------------------------------------------------------
// Defines the Dance operation, which is a generic operation.
// -----------------------------------------------------------------------------

pub trait Dance<Style> {
    fn dance(self) -> Style;
}

#[macro_export]
macro_rules! Dance {
    (real_impl, $name:ident, $datatype:ty, <$style:path>, $($rest:tt)*) => {
        impl<B: Bus> $crate::Dance<$style> for $name<B> {
            fn dance(self) -> $style {
                $style
            }
        }
    };
    ($($unknown:tt)*) => {};
}

#[derive(Debug)]
pub struct Tango;
#[derive(Debug)]
pub struct Waltz;

// -----------------------------------------------------------------------------
// Defines the Ball operation, which has an associated type that the user can
// constrain.
// -----------------------------------------------------------------------------

pub trait Ball {
    type Use: Debug;
    fn name(self) -> String;
}

#[macro_export]
macro_rules! Ball {
    (real_impl, $name:ident, $datatype:ty, <Use = $use:path>, $($rest:tt)*) => {
        impl<B: Bus> $crate::Ball for $name<B> {
            type Use = $use;
            fn name(self) -> String {
                format!("{:?}Ball", $use)
            }
        }
    };
    ($($unknown:tt)*) => {};
}

#[derive(Debug)]
pub struct Basket;
#[derive(Debug)]
pub struct Disco;

registers! {
    #![buses(LiteX<8, 32>, LiteX<32, 32>)]

    a: u8 { Read, Dance<Tango>, Dance<Waltz> },
    b: [u8; 2] { Read, Ball<Use = Basket> },
    c: a,
    d: [b; 2],
    e {
        0 => f: u8 { Read, Dance<Tango> },
        [4, 4] => g: [u8; 2] { Read, Ball<Use = Disco> },
        12 => h: c,
        16 => _: 8,
        24 => i: [b; 2],
    },
}

fn main() {
    use e::Interface;
    let mut peripheral = [0u8; e::Real::<LiteX<8, 32>>::SIZE];
    let real = unsafe { e::Real::new(LiteX::<8, 32>(NonNull::from(&mut peripheral).cast())) };
    println!("tango: {:?}", Dance::<Tango>::dance(real.f()));
    println!("waltz: {:?}", Dance::<Waltz>::dance(real.h()));
    println!("sport: {}", real.i().get(0).unwrap().get(0).unwrap().name());
    println!("ceiling: {}", real.g().get(0).unwrap().name());
}
