// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

/// Macro that defines registers and register blocks (peripherals). See `README.md` for a
/// high-level overview; this documentation is an in-depth explanation of `register_map!`'s
/// capabilities.
///
/// # The simplest example: a scalar primitive register.
/// A basic register definition looks like:
/// ```
/// # fn main() {}
/// use tock_registers::{register_map, Mmio32, Read, Write};
/// register_map! {
///     #[bus(Mmio32)]
///     ctrl: u8 { Read, Write },
/// }
/// ```
/// This declares an MMIO register (for 32-bit systems) called `ctrl`, whose data type is `u8`,
/// which can be read from or written to. This definition generates a module that contains the
/// following items:
/// ```
/// # fn main() {}
/// # use tock_registers::{Mmio32, Read, Write};
/// mod ctrl {
///     use super::*;
///     pub trait Interface: tock_registers::Register<DataType = u8> + Read + Write {}
///     pub trait Bus {}
///     impl Bus for Mmio32 {}
///     #[derive(Clone, Copy)]
///     pub struct Real<B: Bus = Mmio32>(B);
/// }
/// ```
/// Note that many details, such as trait bounds and trait sealing, are omitted for clarity. `Real`
/// implements several traits: [`Span`](crate::Span), [`Read`], [`Write`], and `Interface`.
///
/// To use the generated register type, use `Real::new` to construct a handle that points to the
/// register, then use the [`Read`] and/or [`Write`] traits to access the register.
///
/// To unit test code that uses the generated register, modify the code that you want to test so
/// that it takes a `C: ctrl::Interface` rather than a `Real`. Then, in the unit test, you can pass
/// in a [`FakeRegister`](crate::FakeRegister) with whatever behavior the test case needs. See
/// `doc/UnitTesting.md` in the tock-registers repository for more information.
///
/// # mmio32_register_map and mmio64_register_map
/// Instead of writing `#[bus(Mmio32)]` or `#[bus(Mmio64)]` every time you call `register_map!`,
/// you can use [`mmio32_register_map!`](crate::mmio32_register_map) or
/// [`mmio64_register_map!`](crate::mmio64_register_map). The rest of the examples on this page
/// will use `mmio32_register_map` for brevity.
///
/// # Bitfield registers
/// A register's type can be a bitfield:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, register_bitfields, Read, Write};
/// register_bitfields! [u8,
///     Control [OFF 0, ON 1],
/// ];
/// mmio32_register_map![ctrl: Control::Register { Read, Write }];
/// ```
///
/// # Register arrays
/// You can define register arrays as well:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, Read, Write};
/// mmio32_register_map![buttons: [u8; 8] { Read }];
/// // You can nest array types as well.
/// mmio32_register_map![led_grid: [[u8; 8]; 8] { Read, Write }];
/// ```
/// In the generated module, the `Interface` trait will depend on
/// [`RegisterArray`](crate::RegisterArray) instead of `Register`, and the `RegisterArray::Element`
/// type will be `Read` and/or `Write`.
///
/// # Register references
/// Registers can be references to other registers:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, Read};
/// // The original register definition.
/// mmio32_register_map![button: u8 { Read }];
/// // A clone of the register.
/// mmio32_register_map![button2: button];
/// // An array of buttons:
/// mmio32_register_map![button_array: [button; 8]];
/// ```
/// The data type and operations that a register has are inherited from the register it refers to
/// (so `button2` implements `Read`, and `button_array` is an array of readable registers). These
/// references work across crates (the register type being referred to is the path to the generated
/// register module).
///
/// # Register blocks
/// You can declare a block of registers:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, Read, Write};
/// mmio32_register_map! {
///     uart {
///         0 => status: u8 { Read },
///         1 => ctrl: u16 { Read, Write },
///         3 => buffer: u8 { Read, Write },
///     }
/// }
/// ```
/// The leading integers (0, 1, 3) are address offsets (the value added to the base address to get
/// the register's address). The generated module looks like:
/// ```
/// # fn main() {}
/// # use tock_registers::{Mmio32, Read, Write};
/// mod uart {
///     use super::*;
///     pub trait Interface: Copy {
///         type status: tock_registers::Register<DataType = u8> + Read;
///         fn status(self) -> Self::status;
///         type ctrl: tock_registers::Register<DataType = u16> + Read + Write;
///         fn ctrl(self) -> Self::ctrl;
///         type buffer: tock_registers::Register<DataType = u8> + Read + Write;
///         fn buffer(self) -> Self::buffer;
///     }
///     pub trait Bus {}
///     impl Bus for Mmio32 {}
///     #[derive(Clone, Copy)]
///     pub struct Real<B: Bus>(B);
/// }
/// ```
/// As before, `Real` implements `Interface`. The generated code includes tests that verify the
/// offsets are correct (the registers must not overlap or have gaps between them).
///
/// # Register blocks with references
/// The fields of register blocks can be arrays, references, and arrays of references:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, register_bitfields, Read, Write};
/// register_bitfields! [u8,
///     Control [OFF 0, INPUT 1, OUTPUT 2],
/// ];
/// mmio32_register_map! {
///     gpio_pin {
///         0 => control: Control::Register { Read, Write },
///         1 => value: u8 { Read, Write },
///     }
/// }
/// mmio32_register_map! {
///     pinmux {
///         0 => status: u8 { Read },
///         // `pins` is an array of references to gpio_pin register blocks. This results in
///         // gpio_pin::control and gpio_pin::value being interleaved for a total of 32 bytes.
///         1 => pins: [gpio_pin; 16],
///     }
/// }
/// ```
///
/// # Padding fields
/// Sometimes, register blocks have gaps in them (usually for alignment). To handle this, you must
/// insert a padding field where the gap is:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, Read, Write};
/// mmio32_register_map! {
///     uart {
///         0 => status: u8 { Read },
///         // Padding is specified by replacing the field definition with a _
///         1 => _,
///         // In this case, the padding is inferred to be 3 bytes, because the next field is at
///         // offset 4.
///         4 => ctrl: u16 { Read, Write },
///         // You can also have padding with a specified size. This size is in bytes.
///         6 => _: 1,
///         7 => buffer: u8 { Read, Write },
///     }
/// }
/// ```
///
/// # Aliased fields
/// Sometimes, the offset tests are too restrictive. In that case, you can mark a register as
/// `#[aliased]`, which removes it from the offset calculation logic entirely. This allows you to
/// overlap registers:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, register_bitfields, Read, Write};
/// register_bitfields! [u8,
///     Control [OFF 0, ON 1],
/// ];
/// mmio32_register_map! {
///     /// Uart with an unusual property: the "control" register and "status" register are located
///     /// at the same offset. Writes go to the control register, while reads go to the status
///     /// register.
///     uart {
///         0 => control: Control::Register { Write },
///         #[aliased]
///         0 => status: u8 { Read },
///     }
/// }
/// ```
/// One word of warning: the offset calculation logic also calculates the total size of a register
/// block. That size is used when a register block is embedded into another register type (like
/// another register block or array). Marking a register `#[aliased]` therefore makes it not affect
/// the register's block size. If a `#[aliased]` register extends beyond the end of the
/// non-`#[aliased]` registers, then that register will exist outside the register block's
/// boundaries! That may or may not be what you wanted. If in doubt, put a padding field at the end
/// to make sure the total size is correct.
///
/// # Visibility
/// You can specify the visibility of the generated modules:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, Read};
/// mmio32_register_map![pub button: u8 { Read }];     // pub mod button { ... }
/// mmio32_register_map![pub(crate) button2: button];  // pub(crate) mod button2 { ... }
/// mmio32_register_map![button_array: [button; 8]];   // mod button2 { ... }
/// ```
///
/// # Specifying multiple buses
/// There are some peripherals (such as LiteX peripherals) that support multiple bus types. For
/// those peripherals, you can specify multiple buses. For example (using Mmio32 and Mmio64 because
/// tock-registers does not provide the LiteX bus types):
/// ```
/// # fn main() {}
/// use tock_registers::{register_map, Mmio32, Mmio64, Read, Write};
/// register_map! {
///     #[buses(Mmio32, Mmio64)]
///     rng {
///         0 => ctrl: u8 { Read, Write },
///         1 => random_byte: u8 { Read },
///     }
/// }
/// ```
///
/// Using `#[buses]` removes the default `Bus` type from the definition of the `Real` struct, so
/// you will have to specify the bus any time you use `Real`.
///
/// Sometimes, such as for LiteX peripherals or peripherals that use `usize` registers, the offsets
/// might depend on which bus is used. In that case, you can specify an array of offsets. The
/// offsets apply to each bus in the order the buses are specified:
/// ```
/// # fn main() {}
/// use tock_registers::{register_map, Mmio32, Mmio64, Read, Write};
/// register_map! {
///     #[buses(Mmio32, Mmio64)]
///     dma_rng {
///         0 => address: usize { Read, Write },
///         [4, 8] => length: u32 { Read, Write },
///         // Padding can also move around, and have per-bus length.
///         [8, 12] => _: [4, 0],
///         // Because of the previous padding, this is at offset 12 on both buses.
///         12 => status: u8 { Read },
///     }
/// }
/// ```
///
/// # Doc comments
/// `register_map!` supports doc comments:
/// ```
/// # fn main() {}
/// use tock_registers::{mmio32_register_map, Read, Write};
/// mmio32_register_map! {
///     /// This is an outer doc comment, which will be copied to the generated `uart` module.
///     uart {
///         /// This doc comment will be copied onto `Interface::status()`.
///         0 => status: u8 { Read },
///         /// This doc comment will be copied onto `Interface::buffer()`.
///         1 => buffer: u8 { Read, Write },
///     }
/// }
/// ```
///
/// # Multiple declarations
///
/// Instead of writing multiple `register_map!` invocations, you can instead put multiple
/// definitions in a single `register_map!` call (comma-separated):
///
/// ```
/// # fn main() {}
/// use tock_registers::{register_map, Mmio32, Mmio64, Read, Write};
/// register_map! {
///     // This inner buses attribute sets the default buses for all declarations in this
///     // register_map! call.
///     #![buses(Mmio32, Mmio64)]
///     //! Inner doc comments are copied onto every declaration.
///     // Both `button` and `led` support both 32-bit and 64-bit MMIO.
///     button: u8 { Read },
///     /// You can still have outer doc comments per-declaration.
///     led: u8 { Read, Write },
///     // But `touch` only supports 32-bit MMIO.
///     #[bus(Mmio32)]
///     touch: u8 { Read },
/// }
/// ```
///
/// [`Read`]: trait@crate::Read
/// [`Write`]: trait@crate::Write
#[macro_export]
macro_rules! register_map {
    {$($arguments:tt)*} => {
        $crate::internal::register_map!($crate $($arguments)*);
    }
}
