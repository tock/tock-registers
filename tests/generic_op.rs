// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! Tests registers! using a custom operation that is generic.

//trait MyOp<const NUMBER: usize> {}
//
//macro_rules! MyOp {
//    (real_impl, $name:ident, $datatype:ty, $($rest:tt)*) => {
//        impl<B: Bus> $crate::MyOp<const NUMBER: 0> for $name<B> {}
//    };
//    ($($unknown:tt)*) => {};
//}
//
//tock_registers::mmio32_registers! {
//    pub foo: u8 { MyOp<0> },
//}
