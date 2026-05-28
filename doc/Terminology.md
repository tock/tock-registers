# Terminology

## Bus

Conceptually, a bus is a transport used to talk to registers, such as a memory
bus or I/O port system. See [bus.rs](../src/bus.rs) for more details.

Type-wise, bus refers to an implementation of the `Address`/`Bus<T>` traits.

These definitions do differ a bit:

1. `Mmio32` and `Mmio32Nullable` ([mmio.rs](../src/mmio.rs)) are the same
   conceptual bus (32-bit memory-mapped IO), but separate bus types.
1. `BorrowedBus<T>` (in [bus.rs](../src/bus.rs)) is the same conceptual bus as
   `T`, but a distinct type.

## Register span

A contiguous span of addresses within a single bus' address space. This term is
generally used in connection to the `tock_registers::Span` trait (defined in
[bus.rs](../src/bus.rs)), which is used to nest things inside arrays and
register blocks.

## Layout and field types

A **Layout** is a top-level definition in a `tock_registers::register_map!`
invocation. Each layout is further categorized as a:

- **single register:** A layout with a single element type. Defined using
  `$name: $type ({ $operations... })?` syntax.
- **register block:** A layout consisting of a list of named fields. Defined
  using `$name { $fields... }` syntax.

Examples:

```rust
mmio32_register_map! {
    // A single register layout
    a: u8 { Read },

    // A register block layout
    b {
        // `c` is *not* a layout, it is a field of the register block
        0 => c: u8 { Read, Write },
    },

    // This is also a single register layout, even though it contains a register
    // block. This is because register_map! is not aware that the element type
    // is a register block, it merely sees that it uses the single register
    // syntax.
    d: b,
}
```

Each field in a register block can be either a register or padding:

```rust
mmio32_register_map! {
    uart {
        0 => status: u8 { Read },       // `status` is a field.
        1 => _,                         // padding is a field.
        2 => ctrl: u8 { Read, Write },  // `ctrl` is a field.
    },
}
```

A single register is categorized as a:

- **definition:** A register with a primitive element type and operation list.
- **reference:** A register that refers to a different layout and lacks an
  operation list.

Examples:

```rust
mmio32_register_map! {
    a: u8 { Read },  // definition
    b: a,            // reference (refers to `a`)
    c {
        0 => d: u8 { Read },  // definition
        1 => e: a,            // reference (refers to `a`)
    },
}
```

A single register is categorized as a *scalar* register or an *array* register
based on whether its type specification contains an array:

```rust
mmio32_register_map! {
    a: u8 { Read },                // A scalar register
    b: [u8; 2] { Read },           // An array register
    c {
        0 => d: u8 { Read },       // A scalar register field
        1 => e: [u8; 2] { Read },  // An array register field
    },

    // Like with "single" versus "block", the difference between "scalar" and
    // "array" is based only on the syntax of the register/field, not based on
    // what it points to. Therefore:

    f: b,                // A scalar register
    g: [a; 2],           // An array register
    h {
        0 => i: b,       // A scalar register field
        1 => j: [a; 2],  // An array register field
    },
    k: c,                // A scalar register
}
```

To sum it all up:

```rust
mmio32_register_map! {
    a: u8 { Read },       // A single scalar register definition layout
    b: a,                 // A single scalar register reference layout
    c: [u8; 2] { Read },  // A single array register definition layout
    d: [a; 2],            // A single array register reference layout

    // A register block layout
    e {
        0 => f: u8 { Read },       // A scalar register definition field
        1 => g: a,                 // A scalar register reference field
        2 => h: [u8; 2] { Read },  // An array register definition field
        4 => i: [a; 2],            // An array register reference field
        6 => _: 1,                 // A padding field
    }
}
```
