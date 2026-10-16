<!-- Licensed under the Apache License, Version 2.0 or the MIT License. -->
<!-- SPDX-License-Identifier: Apache-2.0 OR MIT                        -->
<!-- Copyright Tock Contributors 2026.                                 -->

structs2regs
============

Converts hand-written `#[repr(C)]` MMIO register structs in `chips/` to the
`register_structs!` macro from `tock-registers`, one module at a time.

```
cargo run --manifest-path tools/Cargo.toml -p structs2regs -- chips/nrf52/src/nvmc.rs
```

The module is rewritten in place, then `cargo check` is run over the chip
crate to prove the conversion preserved the layout. `git checkout` is the
undo.

Why the output can be trusted
-----------------------------

The tool is built so that a reviewer does not have to trust its arithmetic.

**1. It does not write code, it moves code.** Every token emitted for a field
-- doc comments, attributes, visibility, name and type -- is a byte-exact
slice of the input file, taken from its `syn` span. The tool computes exactly
two things: the hex offsets, and which fields become padding. Field order is
never changed and no field is ever dropped.

**2. The offsets are checked by the compiler.** `register_structs!` expands to
`test_fields!`, which asserts every field's start offset, natural alignment
and end offset, plus the struct's total size, as `const` assertions. A
miscomputed offset is a build failure naming the field:

```
error[E0080]: evaluation panicked: Invalid end offset for field ovr
              (expected 0x064 but actual value differs)
```

**3. Everything else is checked by a generated proof.** `test_fields!` never
calls `offset_of!`, so on its own it cannot see a dropped field, a
reordering, a same-size type swap, or an alignment change. (That last one is
real: macro padding is always `[u8; N]` with alignment 1, so erasing
`_reserved: [u32; 4]` can quietly reduce a struct's alignment while its size
stays correct.) So the tool emits a temporary module holding the original
struct beside the converted one:

```rust
#[allow(dead_code, non_snake_case)]
mod __structs2regs_check_RegisterRC {
    use super::*;
    #[repr(C)]
    struct Original { val: ReadOnly<u32>, reserved0: u32, clear: WriteOnly<u32> }
    const _: () = {
        assert!(core::mem::size_of::<Original>() == core::mem::size_of::<super::RegisterRC>());
        assert!(core::mem::align_of::<Original>() == core::mem::align_of::<super::RegisterRC>());
        assert!(core::mem::offset_of!(Original, val) == core::mem::offset_of!(super::RegisterRC, val));
        // ... one per field, collapsed padding included
    };
}
```

It runs `cargo check`, and removes the module only once the compiler has
accepted it. Pass `--keep-layout-check` to read the proof yourself.

These assertions cover *every* field because of one rule: **collapsing never
renames and never merges**. `(0x014 => _protctrl)` still generates a field
called `_protctrl`, just typed `[u8; 4]`.

What becomes padding
--------------------

A field gives up its type only when all four hold:

- it is private;
- its type is a bare fixed-width integer or an array of them;
- its name begins with `reserved`, `res`, `rsvd`, `pad`, `padding`, `unused`
  or `spare`;
- its identifier appears nowhere else in the chip crate.

All four are needed, and each was added because of real code. The type test
alone rescues 121 fields across `chips/` that the other tests would have
demoted -- `keyr: WriteOnly<u32>` in `stm32f4xx/src/flash.rs` and
`events_ch: [AdcEventChRegisters; 8]` in `nrf52/src/adc.rs` are unused today
but are plainly registers. The name test rescues a few more that are declared
as bare integers: `parameter` and `version` in `sam4l/src/gpio.rs`. Going the
other way, `_protctrl` in `sam4l/src/pm.rs` is underscore-prefixed but is a
register, while `reserved0` in `sam4l/src/gpio.rs` has no underscore and is
padding -- so neither the underscore nor its absence means anything on its
own.

The unused test is sound in the direction that matters. Every way to touch a
field in Rust -- `x.f`, `Foo { f: .. }`, `let Foo { f, .. }`,
`offset_of!(Foo, f)` -- needs the identifier as a literal token; there is no
reflection, `concat_idents!` is unstable, and no `chips/` crate depends on a
proc macro that synthesizes field access. Counting over the whole crate is
deliberately broader than the module a private field is visible in, so an
unrelated name collision only causes a field to keep its type. The two
constructs that read fields without naming them, struct-update syntax and
derived code, both need the struct to be constructible -- and such structs are
refused outright.

Compiler-inserted alignment padding has no name in the source, so it is
emitted as `_implicit_paddingN` and listed in the report.

What is refused
---------------

Refusal is always a structural property of the struct's own syntax, decided
before any offset is computed, so the tool never abandons a half-converted
struct. A refused struct is left exactly as it was found and the exit status
is non-zero, so a bulk pass cannot miss it.

| Refused when | Because |
| --- | --- |
| the struct is generic | the macro grammar accepts at most a lifetime |
| it has a `repr` beyond plain `#[repr(C)]` | the macro emits its own, and an added alignment interacts with padding |
| it has a `#[derive]` | derived code reads fields without naming them |
| it is built with a struct literal anywhere in the crate | padding becomes `[u8; N]`, which `_reserved: 0` cannot name |
| two or more distinct cfg predicates sit on its fields | the configurations are not mechanically separable |
| a field type is not in the size table | later offsets would be unknown |
| every register is byte-wide | see *Known limitations* |
| the block is longer than the crate's recursion limit allows | see *Known limitations* |

Structs that are not MMIO blocks at all are skipped silently rather than
reported: a `#[repr(C)]` struct qualifies only if some field is, or is an
array of, `ReadOnly`/`ReadWrite`/`WriteOnly`/`Aliased`. `InMemoryRegister` is
excluded on purpose, since it marks in-memory DMA structures.

cfg-gated fields
----------------

`test_fields!` discards cfg attributes on entries, so one invocation can only
assert one configuration's `@END`. A struct with exactly one cfg predicate is
therefore emitted as a gated pair, each copy an ordinary fully-checked
invocation:

```rust
#[cfg(feature = "stm32f429")]
register_structs! { FlashRegisters { /* ... */ (0x020 => @END), } }

#[cfg(not(feature = "stm32f429"))]
register_structs! { FlashRegisters { /* ... */ (0x01C => @END), } }
```

Verification then runs `cargo check` once per configuration and reports the
coverage.

Offsets that are not numbers
----------------------------

Most array lengths are literals, but some are named constants. The tool folds
file-local `const` definitions and integer arithmetic, including
`size_of::<uN>()`. What it cannot fold it carries through verbatim, since the
macro takes an `expr` in the offset position:

```rust
(0x2000 => enable: [ReadWrite<u32>; MAX_BIT_REGS]),
(0x2000 + 4 * MAX_BIT_REGS => _reserved3),
```

`test_fields!` evaluates those expressions at compile time like any other, so
nothing is lost. A constant with a cfg attribute or more than one definition
is never folded, because it has no single value.

Known limitations
-----------------

Both are properties of `tock-registers` 0.10.0, not of this tool, and both
are detected up front rather than discovered at build time.

**Block length.** `test_fields!` recurses once per entry, so a block is capped
at `recursion_limit / 2 - 2` entries -- 62 by default. Several chip crates
already raise the limit for this reason (`chips/apollo3/src/lib.rs`,
`chips/earlgrey/src/lib.rs`). The tool reads the crate's current limit and, if
a block is too long, names the value to set:

```
skipped    IomuxcRegisters   iomuxc.rs:13
    block has 123 entries but the crate's recursion limit of 128 allows 62;
    add `#![recursion_limit = "256"]` to the crate root and re-run
```

**All-byte-wide blocks.** If every register is `u8` the block's alignment is
1, and `test_fields!` expands its size check to `$size % 1`, which
`clippy::modulo_one` denies. The lint fires inside the expansion, where an
`#[allow]` on the invocation does not reach it, so the result cannot pass
`make clippy`. Fixing this upstream (the macro already carries an `#[allow]`
for a sibling case) would unblock `chips/qemu_virt_chip/src/uart.rs`.

Options
-------

| Flag | Effect |
| --- | --- |
| `--struct NAME` | convert only the named struct; repeatable |
| `--stdout` | print the converted module instead of rewriting it; implies `--no-verify` |
| `--no-verify` | skip the `cargo check` layout proof |
| `--keep-layout-check` | leave the proof module in the file so you can read it |
| `--crate-root DIR` | override the crate root; used by the tests |

Tests
-----

```
cargo test --manifest-path tools/Cargo.toml -p structs2regs
```

`tests/fixtures/<case>/src/input.rs` holds a shape taken from real code in
`chips/`; `expected.rs` and `report.txt` are the snapshots. Recording the
report is what keeps the refusal messages honest. `UPDATE_FIXTURES=1`
rewrites the snapshots after an intentional change.
