# Glossary

**Bus:** A transport used to talk to registers. See [bus.rs](../src/bus.rs) for
more information. In many contexts, "bus" can also refer to a particular
implementation of the `Address`/`Bus<T>` traits.

**Register span:** A contiguous span of addresses within a single bus' address
space. This term is generally used in connection to the `tock_registers::Span`
trait, which is used to nest things inside arrays and register blocks.
