// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

#![no_implicit_prelude]

use ::tock_registers::{Read, Write};

::tock_registers::mmio32_registers! {
    // An individual register, which can be re-used later.
    // Roughly equivalent to (from tock registers v1):
    //     type Status = ReadOnly<u8>;
    status: u8 { Read },

    // A register array, which can be re-used later.
    // Roughly equivalent to (from tock registers v1):
    //     type Buttons = [ReadOnly<u8>; 4];
    buttons: [u8; 4] { Read },

    // Scalar reference to an existing register.
    status_ref: status,

    // Arrays can be created by referring to other registers too.
    // Roughly equivalent to (from tock registers v1):
    //     type StatusArray = [Status; 4];
    status_array: [status; 4],

    // A register block. This is the equivalent to register_structs!
    simple_foo {
        // This defines a field called simple_status, whose type is defined
        // above as `status` (i.e. this is a read-only u8).
        0x0 => simple_status: status,

        // This defines a field called simple_buttons, whose type is the
        // register array `buttons`.
        0x1 => simple_buttons: buttons,

        // This defines a register field inline. Most registers will be defined
        // this way.
        0x5 => control: u16 { Read, Write },
    },

    // A larger register block. This register block contains an instance of a
    // simple_foo inside of itself.
    complex_foo {
        // The nested simple_foo
        0x0 => nested_foo: simple_foo,

        // An array of status registers.
        0x7 => status_array: [status; 2],

        // Ar array of inline-defined registers.
        0x9 => inline_array: [u8; 2] { Read, Write },
    },

    // Arrays can be created out of register blocks as well.
    many_simple_foos: [simple_foo; 8],

    // Oh, and to keep the example's size limited, I don't show this, but:
    // types should be able to refer to types defined in other crates/modules.
    // I.e. simple_foo could be defined in one crate, which the crate that
    // defines complex_foo depends on.
}
