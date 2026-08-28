# Safe DMA API example

This example demonstrates how a safe DMA abstraction can be developed on top of
`register_map!`. It is divided into several modules, each of which behaves like a crate:

- safe_dma.rs is the crate exposing the safe DMA API. Depends on tock-registers.
- chip_unsafe.rs is the unsafe chip crate from Tock's crate hierarchy. Depends on the safe DMA
  crate.
- chip_fake.rs is a unit testing support crate. Depends on the unsafe chip crate.
- chip_safe.rs is the main chip crate, which contains the bulk of the peripheral's driver. It
  depends on chip_unsafe and has a dev-dependency on chip-fake.
- main.rs represents a board file, instantiating the Real version of the driver.
