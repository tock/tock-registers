# Safe DMA API example

One thing that tock-registers does *not* provide is a safe API for performing
DMA (Direct Memory Address) operations. This example was written at a time when
the Tock project was designing abstractions for working with DMA from safe code.
However, those abstractions (e.g. `DmaFence`) are Tock-specific and may not be
directly suitable for use by other tock-registers users — and they aren't done
yet — so they're not part of tock-registers.

This example instead gives a demonstration of a safe DMA abstraction that lives
in a crate outside tock-registers. The abstraction is somewhat simplified
relative to what a real DMA access crate would look like, e.g.:

1. It only supports read-write DMA operations.
2. It only supports `'static` buffers.
3. Many APIs are simplified or incompletely implemented for brevity.
4. and many other simplifications.

Importantly, this examples demonstrates ("proves" is probably too strong a word)
that a safe DMA access crate can be built on top of tock-registers. Note that if
you are building a safe DMA crate, you will likely need to:

1. Implement suitable hardware memory fences, which are platform-specific.
2. Use [volatile atomic
operations](https://github.com/rust-lang/unsafe-code-guidelines/issues/615) to
start, stop, and check the status of DMA operations.

This example is divided into several modules, each of which behaves like a crate
in the Tock ecosystem:

- safe_dma.rs is the crate exposing the safe DMA API. Depends on tock-registers.
- chip_unsafe.rs is the unsafe chip crate from Tock's crate hierarchy. Depends on the safe DMA
  crate.
- chip_fake.rs is a unit testing support crate. Depends on the unsafe chip crate.
- chip_safe.rs is the main chip crate, which contains the bulk of the peripheral's driver. It
  depends on chip_unsafe and has a dev-dependency on chip-fake.
- main.rs represents a board file, instantiating the Real version of the driver.
