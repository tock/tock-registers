# Multi-Crate Registers Test

This directory verifies that `register_layouts!` references can cross crate
boundaries. [`multi_crate_child`](child) depends on
[`multi_crate_parent`](parent).
