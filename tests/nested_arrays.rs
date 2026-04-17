// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#![no_std]

//! When length is a type parameter of RegisterArray, this pattern of nested arrays tends to cause
//! compilation errors in the generated code (some of the implied trait bounds that the proc macro
//! relies on do not exist). This regression test prevents us from reintroducing that error, as it
//! managed to slip by all the other tests.

use tock_registers::{mmio32_registers, Read, Register, RegisterArray, Write};

mmio32_registers! {
    pub a: [u8; 2] { Read, Write },
    pub b: [a; 2],
    pub c: [b; 2],
    pub d: [c; 2],
    pub e {
        0 => array: [d; 2],
    }
}

pub fn check0<R: Register<DataType = u8> + Read + Write>() {}

pub fn check1<A: RegisterArray<a::Len, Element: Register<DataType = u8> + Read + Write>>() {
    check0::<A::Element>();
}

pub fn check2<A: a::Interface>() {
    check0::<A::Element>();
    check1::<A>();
}

pub fn check3<B: RegisterArray<b::Len, Element: a::Interface>>() {
    check0::<<B::Element as RegisterArray<_>>::Element>();
    check1::<B::Element>();
    check2::<B::Element>();
}

pub fn check4<B: b::Interface>() {
    check0::<<B::Element as RegisterArray<_>>::Element>();
    check1::<B::Element>();
    check2::<B::Element>();
    check3::<B>();
}

pub fn check5<C: RegisterArray<c::Len, Element: b::Interface>>() {
    check0::<<<C::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check1::<<C::Element as RegisterArray<_>>::Element>();
    check2::<<C::Element as RegisterArray<_>>::Element>();
    check3::<C::Element>();
    check4::<C::Element>();
}

pub fn check6<C: c::Interface>() {
    check0::<<<C::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check1::<<C::Element as RegisterArray<_>>::Element>();
    check2::<<C::Element as RegisterArray<_>>::Element>();
    check3::<C::Element>();
    check4::<C::Element>();
    check5::<C>();
}

pub fn check7<D: RegisterArray<d::Len, Element: c::Interface>>() {
    check0::<<<<D::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check1::<<<D::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check2::<<<D::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check3::<<D::Element as RegisterArray<_>>::Element>();
    check4::<<D::Element as RegisterArray<_>>::Element>();
    check5::<D::Element>();
    check6::<D::Element>();
}

pub fn check8<D: d::Interface>() {
    check0::<<<<D::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check1::<<<D::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check2::<<<D::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check3::<<D::Element as RegisterArray<_>>::Element>();
    check4::<<D::Element as RegisterArray<_>>::Element>();
    check5::<D::Element>();
    check6::<D::Element>();
    check7::<D>();
}

pub fn check9<E: RegisterArray<e::lens::array, Element: d::Interface>>() {
    check0::<<<<<E::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check1::<<<<E::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check2::<<<<E::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check3::<<<E::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check4::<<<E::Element as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check5::<<E::Element as RegisterArray<_>>::Element>();
    check6::<<E::Element as RegisterArray<_>>::Element>();
    check7::<E::Element>();
    check8::<E::Element>();
}

#[rustfmt::skip]
pub fn check10<E: e::Interface>() {
    check0::<<<<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check1::<<<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check2::<<<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check3::<<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check4::<<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element
        as RegisterArray<_>>::Element>();
    check5::<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check6::<<<E::array as RegisterArray<_>>::Element as RegisterArray<_>>::Element>();
    check7::<<E::array as RegisterArray<_>>::Element>();
    check8::<<E::array as RegisterArray<_>>::Element>();
    check9::<E::array>();
}
