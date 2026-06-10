# `tock-registers-codegen` crate

[`tock-registers-macros`](../macros) is the procedural macro crate for
tock-registers. This crate, `tock-registers-codegen`, is a library crate that
contains the implementation of tock-registers' procedural macros.

Projects can use this crate to run the logic of tock-registers' procedural
macros in a context other than a procedural macro. For example, the [macro
expansion tool](../expand_macros) uses this crate to implement the code
expansion.

Note that the generated code is not stable; the version of the
`tock-registers-codegen` crate must exactly match the version of the
`tock-registers` crate that the generated code is compiled against.

## Dependency tree

```
$ cargo tree -e no-dev
tock-registers-codegen v0.10.1 (/home/ryan/tock/registers/codegen)
├── proc-macro2 v1.0.106
│   └── unicode-ident v1.0.24
├── quote v1.0.45
│   └── proc-macro2 v1.0.106 (*)
└── syn v2.0.117
    ├── proc-macro2 v1.0.106 (*)
    ├── quote v1.0.45 (*)
    └── unicode-ident v1.0.24
```
