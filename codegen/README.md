# `tock-registers-codegen` crate

[`tock-registers-macros`](../macros) is the procedural macro crate for
tock-registers. This crate, `tock-registers-codegen`, is a library crate that
contains the implementation of tock-registers' procedural macros.

Projects can use this crate to run the logic of tock-registers' procedural
macros in a context other than a procedural macro. For example, the [macro
expansion tool](../expand_macros) uses this crate to implement the code
expansion.

Note that the generated code is not stable; the version of the
`tock-registers-codegen` crate must be the exact same as the version of the
`tock-registers` crate.
