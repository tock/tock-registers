// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

// TODO: Figure out if we're merging this into the default branch or into a separate feature
//       branch.
// TODO: Determine how to split this into PRs.
// TODO: Figure out tock-registers version numbering (interacts with deprecation strategy).
// TODO: Update the top-level crate doc comment. It should probably match or be similar to
//       the README.
// TODO: Improve parse error handling. There might be three classes of errors:
//       1. Errors which immediately terminate parsing (e.g. unexpected token)
//       2. Errors which prevent generating code, but allow parsing to continue (e.g. register
//          reference must be to a module)
//       3. Errors where we can still generate code (e.g. multiple #[aliased] attributes)
// TODO: Implement a arm64_secure_vm feature (see the TODO in src/mmio.rs).
// TODO: Implement a RegisterArray iterator.
// TODO: Implement UnimplementedRegister, add to operation documentation.
// TODO: Implement macro that automatically provides the type defs for Interface trait impls.
// TODO: Re-evaluate which `syn` features we need (is full necessary?).

use tock_registers_codegen::Env::ProcMacro;

#[proc_macro]
pub fn register_map(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(out) | Err(out)) = tock_registers_codegen::register_map(input.into(), ProcMacro);
    out.into()
}
