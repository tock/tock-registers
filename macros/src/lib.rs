// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.
// Copyright Better Bytes 2026.

use tock_registers_codegen::Env::ProcMacro;

#[proc_macro]
pub fn register_map(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(out) | Err(out)) = tock_registers_codegen::register_map(input.into(), ProcMacro);
    out.into()
}
