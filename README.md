# ![Tock-Register-Interface](http://www.tockos.org/assets/img/tockregisters.svg "Tock Registers Logo")

This crate provides an interface and types for defining and
manipulating registers and bitfields.

**NOTE:** tock-registers is currently undergoing a significant redesign, and
this README describes the new design. If you are here to learn how to use
tock-registers today, I suggest looking at a [recent release's
README](https://github.com/tock/tock-registers/tree/v0.10.1) instead.

## Defining registers

Note: tock-registers has several pieces of functionality that are not described
in this document, including support for [non-MMIO
registers](doc/AddingRegisterTypes.md). See the Rustdoc for
[`register_map!`](src/map.rs) for complete documentation.

A register block is defined using the
`mmio32_register_map!`/`mmio64_register_map!` macros:

```rust
use tock_registers::{mmio32_register_map, Mmio32, Read, Write};

// Specifies that these registers are memory-mapped IO registers with 32-bit
// addresses.
mmio32_register_map! {
    /// Documentation for `peripheral`.
    peripheral {
        // Control register: read-write. `Control` is defined using the
        // `register_bitfields!` macro.
        0x000 => cr: Control::Register { Read, Write },

        // Status register: read-only
        0x001 => s: Status::Register { Read },

        // Registers can be bytes, halfwords, or words:
        0x002 => byte0: u8 { Read, Write },
        0x003 => byte1: u8 { Read, Write },
        0x004 => short: u16 { Read, Write },

        // Empty space between registers must be marked with a padding field.
        0x006 => _,
        0x008 => word: u32 { Read, Write },

        // The type for a register can be anything. Conveniently, you can use an
        // array when there are a bunch of similar registers.
        // If you have several adjacent registers, you can specify an array:
        0x00C => gpio_pins: [u32; 4] { Read, Write },
        // Array registers can have bitfields as well
        0x01C => port_ctrl: [PortCtrl::Register; 4] { Read, Write },
    }
}
```

The fields (registers and padding) must be in order of increasing offset with no
gaps.

The macro generates a module that is designed to be used both on real hardware
and in a unit test environment. Therefore, the module contains traits and a
struct:

```rust
mod peripheral {
    // Allows users to refer to register types and operations from the
    // surrounding scope.
    use super::*;

    // Trait representing the peripheral's registers.
    pub trait Interface: Copy {
        type cr: Register<DataType = Control::Register> + Read + Write;
        fn cr(self) -> Self::cr;
        type s: Register<DataType = Status::Register> + Read;
        fn s(self) -> Self::s;
        /* byte0, byte1, short, and word are similar, omitted */
        type gpio_pins: RegisterArray<Element: Register<DataType = u32> + Read + Write>;
        fn gpio_pins(self) -> Self::gpio_pins;
        type port_ctrl: RegisterArray<Element: Register<DataType = PortCtrl::Register> + Read + Write>;
    }

    // Trait representing the buses that this peripheral can be on.
    pub trait Bus: /* bounds omitted */ { /* omitted */ }
    impl Bus for Mmio32 {}

    // Implementation of `Interface` for use on the real chip. `Real` is a
    // handle that refers to the registers -- e.g. for MMIO registers `Real`
    // wraps a pointer.
    pub struct Real<B: Bus> { /* omitted */ }

    impl<B: Bus> Interface for Real<B> { /* omitted */ }
}
```

See [Unit Testing](doc/UnitTesting.md) for information on how to test code that
uses tock-registers.

The visibility of the generated module matches the visibility of its definition
in the macro invocation. In other words, you can make the module public using
the `pub` keyword just before the module name:

```rust
use tock_registers::{mmio32_register_map, Read};

mmio32_register_map! {
    pub peripheral {
        0 => status: u8 { Read },
    }
}
```

will generate:

```rust
pub mod peripheral {
    // Contents omitted
}
```

## Defining bitfields

Bitfields are defined through the `register_bitfields!` macro:

```rust
register_bitfields! [
    // First parameter is the register width. Can be u8, u16, u32, or u64.
    u32,

    // Each subsequent parameter is a register abbreviation, its descriptive
    // name, and its associated bitfields.
    // The descriptive name defines this 'group' of bitfields. Only registers
    // defined with data type Control::Register can use these bitfields.
    Control [
        // Bitfields are defined as:
        // name OFFSET(shift) NUMBITS(num) [ /* optional values */ ]

        // This is a two-bit field which includes bits 4 and 5
        RANGE OFFSET(4) NUMBITS(2) [
            // Each of these defines a name for a value that the bitfield can be
            // written with or matched against. Note that this set is not exclusive--
            // the field can still be written with arbitrary constants.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ],

        // A common case is single-bit bitfields, which usually just mean
        // 'enable' or 'disable' something.
        EN  OFFSET(3) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) []
    ],

    // Another example:
    // Status register
    Status [
        TXCOMPLETE  OFFSET(0) NUMBITS(1) [],
        TXINTERRUPT OFFSET(1) NUMBITS(1) [],
        RXCOMPLETE  OFFSET(2) NUMBITS(1) [],
        RXINTERRUPT OFFSET(3) NUMBITS(1) [],
        MODE        OFFSET(4) NUMBITS(3) [
            FullDuplex = 0,
            HalfDuplex = 1,
            Loopback = 2,
            Disabled = 3
        ],
        ERRORCOUNT OFFSET(6) NUMBITS(3) []
    ],

    // In a simple case, offset can just be a number, and the number of bits
    // is set to 1:
    InterruptFlags [
        // This is equivalent to writing
        // UNDES OFFSET(10) NUMBITS(1) [],
        UNDES   10,
        TXEMPTY  9,
        NSSR     8,
        OVRES    3,
        MODF     2,
        TDRE     1,
        RDRF     0
    ]
]
```

This generates the modules `Control`, `Status`, and `InterruptFlags`

```rust

mod Control {
    pub struct Register;
    pub const MODE: Field<u8, Register> = Field::<u8, Register>::new(0b111, 0);
    pub const ENABLE: Field<u8, Register> = Field::<u8, Register>::new(0b1, 3);

    pub mod MODE {
        use super::{FieldValue, Register};
        pub const Mode0: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b111, 0, 0);
        pub const Mode1: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b111, 0, 1);
        pub const Mode2: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b111, 0, 2);
        pub const Mode3: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b111, 0, 3);
        pub const SET: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b111, 0, 0b111);
        pub const CLEAR: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b111, 0, 0);

        #[repr(u8)]
        pub enum Value { Mode0 = 0, Mode1 = 1, Mode2 = 2, Mode3 = 3 }
        impl TryFromValue<u8> for Value {
            fn try_from(v: u8) -> Option<Self> {
                match v {
                    0 => Some(Value::Mode0),
                    1 => Some(Value::Mode1),
                    2 => Some(Value::Mode2),
                    3 => Some(Value::Mode3),
                    _ => None,
                }
            }
        }
    }

    pub mod ENABLE {
        use super::{FieldValue, Register};
        pub const Disabled: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b1, 3, 0);
        pub const Enabled: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b1, 3, 1);
        pub const SET: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b1, 3, 1);
        pub const CLEAR: FieldValue<u8, Register> = FieldValue::<u8, Register>::new(0b1, 3, 0);

        #[repr(u8)]
        pub enum Value { Disabled = 0, Enabled = 1 }
        impl TryFromValue<u8> for Value {
            fn try_from(v: u8) -> Option<Self> {
                match v {
                    0 => Some(Value::Disabled),
                    1 => Some(Value::Enabled),
                    _ => None,
                }
            }
        }
    }
}
```

The macro generates a module for each register (e.g., Control, Status, InterruptFlags) that includes:
- A `Register` struct for each register, which acts as a placeholder for the register type.
- `Field`s within the register are defined as constants, such as `RANGE`, `EN`, and `INT` for the `Control` register.
- Each field is represented by the `Field` type, which encapsulates the bit offset and width.


## Register Operation Summary

`Read` and `Write`, which the above examples use, are two examples of operations
that registers can support. They provide the following methods:

```rust
Read:
.get() -> T                                    // Get the raw register value
.read(field: Field<T, R>) -> T                 // Read the value of the given field
.read_as_enum<E>(field: Field<T, R>) -> Option<E> // Read value of the given field as a enum member
.is_set(field: Field<T, R>) -> bool            // Check if one or more bits in a field are set
.any_matching_bits_set(value: FieldValue<T, R>) -> bool  // Check if any bits corresponding to the mask in the passed field are set
.matches_all(value: FieldValue<T, R>) -> bool  // Check if all specified parts of a field match
.matches_any(&self, fields: &[FieldValue<T, R>]) -> bool // Check if any specified parts of a field match
.extract() -> LocalRegisterCopy<T, R>          // Make local copy of register
.debug() -> RegisterDebugValue<T, R>           // Returns a type that implements Debug

Write:
.set(value: T)                                 // Set the raw register value
.write(value: FieldValue<T, R>)                // Write the value of one or more fields,
                                               //  overwriting other fields to zero
.modify(value: FieldValue<T, R>)               // Write the value of one or more fields, leaving
                                               // other fields unchanged (requires the register is
                                               // Read as well)
.modify_no_read(                               // Write the value of one or more fields,
      original: LocalRegisterCopy<T, R>,       //  leaving other fields unchanged, but pass in
      value: FieldValue<T, R>)                 //  the original value, instead of doing a register read
```

In addition to `Read` and `Write`, tock-registers also provides the `UnsafeRead`
and `UnsafeWrite` operations for hardware registers that are unsafe (such as DMA
peripherals).

External crates can [define new
operations](doc/AddingRegisterTypes.md#adding-new-operations), allowing them to
support register types that tock-registers does not directly support.

## Example: Using registers and bitfields

Assuming we have defined a `peripheral` module and the corresponding bitfields
as in the previous two sections. We also have a `peripheral::Real` instance
named `registers`.

```rust
// -----------------------------------------------------------------------------
// RAW ACCESS
// -----------------------------------------------------------------------------

// Get or set the raw value of the register directly. Nothing fancy:
registers.cr().set(registers.cr().get() + 1);


// -----------------------------------------------------------------------------
// READ
// -----------------------------------------------------------------------------

// `range` will contain the value of the RANGE field, e.g. 0, 1, 2, or 3.
// The type annotation is not necessary, but provided for clarity here.
let range: u8 = registers.cr().read(Control::RANGE);

// Or one can read `range` as a enum and `match` over it.
let range = registers.cr().read_as_enum(Control::RANGE);
match range {
    Some(Control::RANGE::Value::VeryHigh) => { /* ... */ }
    Some(Control::RANGE::Value::High) => { /* ... */ }
    Some(Control::RANGE::Value::Low) => { /* ... */ }

    None => unreachable!("invalid value")
}

// `en` will be 0 or 1
let en: u8 = registers.cr().read(Control::EN);


// -----------------------------------------------------------------------------
// MODIFY
// -----------------------------------------------------------------------------

// Write a value to a bitfield without altering the values in other fields:
registers.cr().modify(Control::RANGE.val(2)); // Leaves EN, INT unchanged

// Named constants can be used instead of the raw values:
registers.cr().modify(Control::RANGE::VeryHigh);

// Enum values can also be used:
registers.cr().modify(Control::RANGE::Value::VeryHigh.into())

// Another example of writing a field with a raw value:
registers.cr().modify(Control::EN.val(0)); // Leaves RANGE, INT unchanged

// For one-bit fields, the named values SET and CLEAR are automatically
// defined:
registers.cr().modify(Control::EN::SET);

// Write multiple values at once, without altering other fields:
registers.cr().modify(Control::EN::CLEAR + Control::RANGE::Low); // INT unchanged

// Any number of non-overlapping fields can be combined:
registers.cr().modify(Control::EN::CLEAR + Control::RANGE::High + CR::INT::SET);

// In some cases (such as a protected register) .modify() may not be appropriate.
// To enable updating a register without coupling the read and write, use
// modify_no_read():
let original = registers.cr().extract();
registers.cr().modify_no_read(original, Control::EN::CLEAR);


// -----------------------------------------------------------------------------
// WRITE
// -----------------------------------------------------------------------------

// Same interface as modify, except that all unspecified fields are overwritten to zero.
registers.cr().write(Control::RANGE.val(1)); // implictly sets all other bits to zero

// -----------------------------------------------------------------------------
// BITFLAGS
// -----------------------------------------------------------------------------

// For one-bit fields, easily check if they are set or clear:
let txcomplete: bool = registers.s().is_set(Status::TXCOMPLETE);

// -----------------------------------------------------------------------------
// MATCHING
// -----------------------------------------------------------------------------

// You can also query a specific register state easily with `matches_all` or
// `any_matching_bits_set` or `matches_any`:

// Doesn't care about the state of any field except TXCOMPLETE and MODE:
let ready: bool = registers.s().matches_all(Status::TXCOMPLETE:SET +
                                            Status::MODE::FullDuplex);

// This is very useful for awaiting for a specific condition:
while !registers.s().matches_all(Status::TXCOMPLETE::SET +
                                 Status::RXCOMPLETE::SET +
                                 Status::TXINTERRUPT::CLEAR) {}

// Or for checking whether any interrupts are enabled:
let any_ints = registers.s().any_matching_bits_set(Status::TXINTERRUPT + Status::RXINTERRUPT);

// Or for checking whether any completion states are cleared:
let any_cleared = registers.s().matches_any(&[Status::TXCOMPLETE::CLEAR, Status::RXCOMPLETE::CLEAR]);

// Or for checking if a multi-bit field matches one of several modes:
let sub_word_size = registers.s().matches_any(&[Size::Halfword, Size::Word]);

// Or for checking if any of several fields exactly match in the register:
let not_supported_mode = registers.s().matches_any(&[Status::Mode::HalfDuplex, Status::Mode::VARSYNC, Status::MODE::NOPARITY]);

// Also you can read a register with set of enumerated values as a enum and `match` over it:
let mode = registers.cr().read_as_enum(Status::MODE);

match mode {
    Some(Status::MODE::Value::FullDuplex) => { /* ... */ }
    Some(Status::MODE::Value::HalfDuplex) => { /* ... */ }

    None => unreachable!("invalid value")
}

// -----------------------------------------------------------------------------
// LOCAL COPY
// -----------------------------------------------------------------------------

// More complex code may want to read a register value once and then keep it in
// a local variable before using the normal register interface functions on the
// local copy.

// Create a copy of the register value as a local variable.
let local = registers.cr().extract();

// Now all the functions for a ReadOnly register work.
let txcomplete: bool = local.is_set(Status::TXCOMPLETE);

// -----------------------------------------------------------------------------
// In-Memory Registers
// -----------------------------------------------------------------------------

// In some cases, code may want to edit a memory location with all of the
// register features described above, but the actual memory location is not a
// fixed MMIO register but instead an arbitrary location in memory. If this
// location is then shared with the hardware (i.e. via DMA) then the code
// must do volatile reads and writes since the value may change without the
// software knowing. To support this, the library includes an `InMemoryRegister`
// type.

let control: InMemoryRegister<u32, Control::Register> = InMemoryRegister::new(0)
control.write(Contol::BYTE_COUNT.val(0) +
              Contol::ENABLE::Yes +
              Contol::LENGTH.val(10));
```

Note that `modify` performs exactly one volatile load and one volatile store,
`write` performs exactly one volatile store, and `read` performs exactly one
volatile load. Thus, you are ensured that a single call will set or query all
fields simultaneously.

## Performance

Examining the binaries while testing this interface, everything compiles
down to the optimal inlined bit twiddling instructions--in other words, there is
zero runtime cost, as far as an informal preliminary study has found.

## Nice type checking

This interface helps the compiler catch some common types of bugs via type checking.

If you define the bitfields for e.g. a control register, you can give them a
descriptive group name like `Control`. This group of bitfields will only work
with a register whose data type is `Control::Register`. For instance, if we have
the bitfields and registers as defined above,

```rust
// This line compiles, because registers.cr is associated with the Control group
// of bitfields.
registers.cr().modify(Control::RANGE.val(1));

// This line will not compile, because registers.s is associated with the Status
// group, not the Control group.
let range = registers.s().read(Control::RANGE);
```

## Naming conventions

There are several related names in the register definitions. Below is a
description of the naming convention for each:

```rust
use tock_registers::{mmio32_register_map, register_bitfields, Read, Write};

mmio32_register_map! {
    registers {
        // The register name in the struct should be a lowercase version of the
        // register abbreviation, as written in the datasheet:
        0 => cr: Control::Register { Read, Write },
    }
}

register_bitfields! [
    u8,

    // The name should be the long descriptive register name,
    // camelcase, without the word 'register'.
    Control [
        // The field name should be the capitalized abbreviated
        // field name, as given in the datasheet.
        RANGE OFFSET(4) NUMBITS(3) [
            // Each of the field values should be camelcase,
            // as descriptive of their value as possible.
            VeryHigh = 0,
            High = 1,
            Low = 2
        ]
    ]
]
```

## Debug trait

By default, if you print the value of a register, you will get the raw value as a number.

How ever, you can use the `debug` method to get a more human readable output.

This is implemented in `LocalRegisterCopy` and in `Read` registers.

Example:

```rust
use tock_registers::Read;

// Create a copy of the register value as a local variable.
let local = registers.cr().extract();

println!("cr: {:#?}", local.debug());
```

For example, if the value of the `Control` register is `0b0000_0100`, the output will be:

```rust
cr: Control {
    RANGE: VeryHigh,
    EN: 0,
    INT: 1
}
```

Similarly it works directly on the register:

```rust
use tock_registers::Read;

println!("cr: {:#?}", registers.cr.debug());
```
> Do note this will issue a read to the register once.

License
-------

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
