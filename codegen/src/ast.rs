// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

//! The Abstract Syntax Tree for a register_map! invocation.

use proc_macro2::TokenStream;
use quote::quote;
use std::{ops::Index, slice};
use syn::{Attribute, Ident, LitInt, Path, Type, TypePath, Visibility};

/// Represents the full input to the register_map! procedural macro.
///
/// Note that `tock_registers::register_map!` prepends `$crate` to the input provided by the user,
/// so that the generated code can refer to tock_registers even if the user has renamed the crate.
/// Therefore, after `tock_registers::register_map!` is expanded, the full input looks like:
///
/// ```
/// # use tock_registers::{Mmio32, Read, Write};
/// # fn main() {}
/// tock_registers::internal::register_map! {
///     ::tock_registers             // The prepended $crate
///     //! Global doc comment       // This doc comment should attach to everything in the macro.
///     #![buses(Mmio32)]            // Global buses attribute
///     a: u8 { Read, Write },       // A register defined by primitive type and operation list
///     /// Doc comment              // Doc comment that should attach to `b`
///     pub b: [a; 2],               // A register array that refers to another definition
///     /// Doc comment              // Doc comment that should attach to `foo`
///     pub foo {                    // Start of a register block
///         0 => c: u8 { Read },     // Field register defined by primitive type and operation list
///         1 => _: 1,               // Padding of size 1 byte
///         2 => d: a,               // Field register that refers to another definition
///         /// Doc comment          // Doc comment that should attach to `e`
///         3 => e: [b; 2],          // Field array register that contains another definition
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug))]
pub struct Input {
    /// The $crate passed in by the register_map! macro_rules macro (used to refer to the
    /// tock_registers crate).
    pub tock_registers: Path,
    pub layouts: Vec<Layout>,
}

/// An individual register or register block layout.
///
/// ```
/// # use tock_registers::{Read, Write};
/// # fn main() {}
/// tock_registers::mmio32_register_map! {
///     // `a` is a Layout
///     a: u8 { Read, Write },
///
///     // `b` is a Layout, and it includes the doc comment before it.
///     /// Doc comment
///     pub b: [a; 2],
///
///     // `foo` is a Layout, and it includes the doc comment and attributes before it.
///     /// Doc comment
///     pub foo {
///         0 => c: u8 { Read },  // Individual fields are `Field`s, not Layouts
///         1 => _: 1,
///         2 => d: a,
///         3 => e: [b; 2],
///     }
/// }
/// ```
///
/// When a Layout is parsed, if no `#[bus]` or `#[buses]` attribute is present, `bus` will be set
/// to `BusAttr::Buses(vec![])`. The Parse impl for `Input` will correct the `bus` entry.
#[cfg_attr(test, derive(Debug))]
pub struct Layout {
    /// Doc comments, converted into outer attributes.
    pub docs: Vec<Attribute>,
    pub bus: BusAttr,
    pub visibility: Visibility,
    pub name: Ident,
    pub value: Value,
}

/// The `#[bus]` or `#[buses]` attribute for a layout.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Clone)]
pub enum BusAttr {
    Bus(TypePath),
    Buses(Vec<TypePath>),
}

impl BusAttr {
    pub fn as_slice(&self) -> &[TypePath] {
        match self {
            BusAttr::Bus(bus) => slice::from_ref(bus),
            BusAttr::Buses(buses) => buses,
        }
    }

    /// Returns the `= <Bus>` default type for the `B: Bus` argument on a real struct, or an empty
    /// stream if there is no such bound.
    pub fn generic_default(&self) -> TokenStream {
        match self {
            BusAttr::Bus(bus) => quote![= #bus],
            BusAttr::Buses(_) => TokenStream::new(),
        }
    }

    /// Returns the number of buses.
    pub fn len(&self) -> usize {
        match self {
            BusAttr::Bus(_) => 1,
            BusAttr::Buses(buses) => buses.len(),
        }
    }
}

/// The part of a register that begins after the register's name.
///
/// For individual register layouts, this starts with the colon, and in register blocks this begins
/// with the opening brace.
///
/// ```
/// # use tock_registers::{Read, Write};
/// # fn main() {}
/// tock_registers::mmio32_register_map! {
///     aa: u8 { Read, Write },
///     //^^^^^^^^^^^^^^^^^^^^ Value::Single
///
///     /// Doc comment
///     pub b: [aa; 2],
///     //   ^^^^^^^^ Value::Single
///
///     /// Doc comment
///     pub foo {
///     //      ^  The Value::Block starts here, and continues through the final }
///         0 => c: u8 { Read },
///         1 => _: 1,
///         2 => d: aa,
///         3 => e: [b; 2],
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug))]
pub enum Value {
    Block(Vec<Field>),
    Single(RegisterSpec),
}

/// An individual field in a register block. A Field can be padding or a register.
///
/// ```
/// # use tock_registers::{Read, Write};
/// # fn main() {}
/// tock_registers::mmio32_register_map! {
///     a: u8 { Read, Write },
///
///     pub foo {
///         0 => c: u8 { Read },
///       //^^^^^^^^^^^^^^^^^^^ Field
///
///         1 => _: 1,
///       //^^^^^^^^ Field (padding)
///
///         #[aliased] 2 => d: a,
///       //^^^^^^^^^^^^^^^^^^^^ Field
///
///         // The doc comment is also part of the field
///         /// Doc comment
///         2 => f: [a; 256],
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Field {
    pub offsets: PerBusInt,
    pub field_def: FieldDef,
}

/// Contents of a field.
///
/// Note that when a FieldDef::Register is initially parsed, `aliased` is always false and `docs`
/// is always empty. The Parse impl on Field sets those fields.
///
/// ```
/// # use tock_registers::Read;
/// # fn main() {}
/// tock_registers::mmio32_register_map! {
///     status: u8 { Read },
///
///     foo {
///         0 => c: u8 { Read },
///         //   ^^^^^^^^^^^^^^ FieldDef::Register
///
///         1 => _,
///         //   ^ FieldDef::Padding
///
///         2 => d: status,
///         //   ^^^^^^^^^ FieldDef::Register
///
///         3 => _: 1,
///         //   ^^^^ FieldDef::Padding
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum FieldDef {
    Padding(Option<PerBusInt>),
    Register {
        /// Doc comments for this register.
        docs: Vec<Attribute>,
        aliased: bool,
        name: Ident,
        spec: RegisterSpec,
    },
}

/// Per-bus integer literal. Used for both field offsets and padding sizes. This can be a single
/// value, which applies to all buses, or an array of values. The number of values in the array
/// must match the number of buses.
///
/// ```
/// # use tock_registers::{Mmio32, Mmio64, Read, Write};
/// # fn main() {}
/// tock_registers::register_map! {
///     #[buses(Mmio32, Mmio64)]
///     foo {
///         0 => c: u8 { Read },
///       //^ PerBusInt::Single
///
///       //v PerBusInt::Single
///         1 => _: 1,
///       //        ^ PerBusInt::Single
///
///         2 => d: usize { Read, Write },
///       //^ PerBusInt::Single
///
///         [6, 10] => e: u8 { Read },
///       //^^^^^^^ PerBusInt::Array
///
///       //vvvvvvv PerBusInt::Array
///         [7, 11] => _: [4, 0],
///       //              ^^^^^^ PerBusInt::Array
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum PerBusInt {
    Array(Vec<LitInt>),
    Single(LitInt),
}

impl Index<usize> for PerBusInt {
    type Output = LitInt;
    fn index(&self, index: usize) -> &LitInt {
        match self {
            PerBusInt::Array(vec) => &vec[index],
            PerBusInt::Single(int) => int,
        }
    }
}

/// A single register specification. A register specification can appear in two places: as its own
/// top-level layout or as a field within a register block. The specification can either be an
/// inline register definition (specifies the register's DataType and operations) or a register
/// reference (whic refers to a register definition elsewhere). The RegisterSpec begins immediately
/// after the register's name (i.e. it includes the ':').
///
/// ```
/// # use tock_registers::{Read, Write};
/// # fn main() {}
/// tock_registers::mmio32_register_map! {
///     pub pin: u8 { Read, Write },
///     //     ^^^^^^^^^^^^^^^^^^^^ Top-level RegisterSpec defining a new register
///     pub pin_pair: [pin; 2],
///     //          ^^^^^^^^^^ Top-level RegisterSpec referencing a separate definition (`status`)
///     pub foo {
///         0x0 => ctrl: u8 { Read, Write },
///         //         ^^^^^^^^^^^^^^^^^^^^ Field RegisterSpec defining a new register
///
///         0x1 => _: 1,  // Padding is NOT a RegisterSpec
///
///         0x2 => pins: [pin; 2],
///         //         ^^^^^^^^^^ RegisterSpec referencing a separate definition
///     }
/// }
/// ```
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct RegisterSpec {
    /// element_type can be a primitive type (for register definitions with operation lists) or a
    /// path to another register definition (for register references). If the register type
    /// specification is an array, this is the innermost type (i.e. element_type does not mention
    /// that it is an array).
    pub element_type: Type,
    /// The array sizes. If this register specification is a nested array, the sizes are listed
    /// from the innermost array to the outermost. For example, `[[[u8; 2]; 3]; 4]` would have
    /// sizes list `[2, 3, 4]`.
    pub array_sizes: Vec<LitInt>,

    /// Operations, if this is a register definition. If this is a register reference, this will be
    /// None.
    pub operations: Option<Vec<Path>>,
}
