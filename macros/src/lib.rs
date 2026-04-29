// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// TODO: Rename `registers!`? Also rename ast::Definition to correspond, then add the following
//       terms to the glossary: <whatever ast::Definition is renamed to>, register block, single
//       register, register definition, register reference, scalar register definition, scalar
//       register reference, array register definition, array register reference.
// TODO: Re-evaluate mod/file structure in tock-registers.
// TODO: Implement #[bus] to specify a default bus (outputs Real<B: Bus = #bus>), update the
//       src/registers_macro.rs docs. Don't forget to update expand_macros!
// TODO: Add #![no_std] to examples/tests where possible.
// TODO: Translate a Tock driver that uses register arrays.
// TODO: Implement a RegisterArray iterator.
// TODO: Rebalance file sizes.
// TODO: Move the real_* structs into a real:: module? impls would stay in the root.
// TODO: Implement a "clippy test" -> a crate that uses tock-registers with as many lints as
//       possible enabled (to verify we don't trip any of them).
// TODO: Verify that offset/size lists match the number of buses.
// TODO: Improve parse error handling. There might be three classes of errors:
//       1. Errors which immediately terminate parsing (e.g. unexpected token)
//       2. Errors which prevent generating code, but allow parsing to continue (e.g. register
//          reference must be to a module)
//       3. Errors where we can still generate code (e.g. multiple #[aliased] attributes)
// TODO: Implement UnimplementedRegister, add to operation documentation.
// TODO: Implement macro that automatically provides the type defs for Interface trait impls.
// TODO: Implement UnsafeRead/UnsafeWrite. No subtrait relationship b/c method name collisions.
// TODO: Implement a arm64_secure_vm feature (see the TODO in src/mmio.rs).
// TODO: Update the top-level crate doc comment. It should probably match or be similar to
//       the README.
// TODO: Investigate adding typestates into the API
//       (https://github.com/jrvanwhy/tock-registers/pull/6).
//       Can we use the lifetime pattern that GhostCell uses to tie state types to the particular
//       register handle? (briefly mentioned in the presentation on 2026-04-29)
// TODO: Re-evaluate which `syn` features we need (is full necessary?).

// Questions to ask the group:
// TODO: Do we want to rename something? We have the Register trait, registers module, and
//       registers! macro. Maybe we need to rename registers! ? When we do the rename, maybe rename
//       `ast::Definition` too (to distinguish it from register definitions).
// TODO: Do we want to remove the trailing commas after declarations? Easy to do, but a bit harder
//       to revert (have some Punctuated iterator code that I don't want to rewrite).

#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(out) | Err(out)) = tock_registers_codegen::registers(input.into());
    out.into()
}
