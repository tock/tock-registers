# `tock-registers-macros`

This is the procedural macro crate for tock-registers. Most of the
implementation of the procedural macros is in
[`tock-registers-codegen`](../codegen), so this is really just a wrapper crate.

## Dependency tree

```
$ cargo tree
tock-registers-macros v0.10.1 (proc-macro) (/home/ryan/tock/registers/macros)
└── tock-registers-codegen v0.10.1 (/home/ryan/tock/registers/codegen)
    ├── proc-macro2 v1.0.106
    │   └── unicode-ident v1.0.24
    ├── quote v1.0.45
    │   └── proc-macro2 v1.0.106 (*)
    └── syn v2.0.117
        ├── proc-macro2 v1.0.106 (*)
        ├── quote v1.0.45 (*)
        └── unicode-ident v1.0.24
```
