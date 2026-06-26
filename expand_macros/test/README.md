# `tock-registers-expand-macros` test

We want to test that you can:

1. Run tock-registers-expand-macros to expand a source code file.
1. Build the resulting source code with tock-registers' proc_macros feature
   disabled.

This crate enables that test. The test itself is performed by the
`expand_macros_test` [Makefile](../../Makefile) action. This crate is not part
of the repository-wide workspace so that `expand_macros_test` does not race with
workspace-wide test commands.
