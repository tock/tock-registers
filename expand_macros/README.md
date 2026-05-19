# tock-registers' macro expansion tool

This tool expands tock-registers' procedural macros inside a Rust file. This
tool exists for a couple reasons:

1. To support projects that have limited trust in the `proc-macro2`, `quote`,
   `syn`, and/or `unicode-ident` crates (see more below).
1. As a debugging tool, to see what the expanded tool looks like (unlike
   `cargo-expand`, this does not recursively expand the output of the procedural
   macros).

Note that this tool is relatively simple, which has a few downsides:

1. It only expands procedural macros, not tock-registers' `macro_rules!` macros.
1. It only expands top-level macro invocations. If you define a module inside
   the file and call the macro from within that `mod {}`, it will not expand
   that invocation. The same applies to other blocks as well.
1. It parses the file with `syn`, so it only recognizes syntax that `syn`
   recognizes.
1. It does a full parse then pretty print, so formatting and non-doc comments
   are lost throughout the file.
1. It does not understand crate renaming or `use` statements. It simply looks
   for the names of known tock-registers macros, possibly with a leading
   `tock_registers::` or `::tock_registers::`.

## Using this tool to avoid proc macro crates

If you want to use tock-registers but you do not wish for your final build to
depend on the `proc-macro2`/`quote`/`syn`/`unicode-ident` crates, you can:

1. Depend on the tock-registers crate with default features disabled, the
   `proc_macros` feature disabled, but `register_types` enabled.
1. Use this tool to expand the macro invocations and write the output into your
   source code tree.
1. Build your application.

This tool depends on `proc-macro2`/`quote`/`syn`/`unicode-ident`, but your final
build should not need those crates. If needed, you can audit the generated code
between the two steps.

We suggest moving your tock-registers macro invocations into their own files, as
that will minimize the impact of this tool's downsides. We're open to having the
CLI expanded to support this use case, e.g. in case you want to add a flag to
write the output to a file.
