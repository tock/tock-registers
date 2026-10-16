// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! A single pass over the chip crate, gathering the two facts that decide
//! whether a field may lose its type and whether a struct may be converted.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::visit::Visit;

pub struct CrateScan {
    /// How many times each identifier appears as a token anywhere in the crate.
    ident_uses: HashMap<String, usize>,
    /// How many times each identifier appears as a *named field declaration*.
    field_decls: HashMap<String, usize>,
    /// Structs named in a struct literal or struct pattern. Such a struct
    /// cannot be converted: `register_structs!` types padding as `[u8; N]`,
    /// which breaks `_reserved: 0` initializers, and struct-update syntax
    /// reads fields without naming them.
    literal_constructed: HashSet<String>,
}

impl CrateScan {
    /// Walk the crate's sources: `<crate root>/src` when that exists, else
    /// the directory itself. Files that fail to parse are skipped; skipping
    /// only ever makes a field look *more* used, never less.
    pub fn run(crate_root: &Path) -> CrateScan {
        let src_dir = crate_root.join("src");
        let dir = if src_dir.is_dir() {
            src_dir
        } else {
            crate_root.to_path_buf()
        };
        let mut scan = CrateScan {
            ident_uses: HashMap::new(),
            field_decls: HashMap::new(),
            literal_constructed: HashSet::new(),
        };
        for path in rust_files(&dir) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(stream) = text.parse::<proc_macro2::TokenStream>() {
                count_idents(stream, &mut scan.ident_uses);
            }
            if let Ok(file) = syn::parse_file(&text) {
                scan.visit_file(&file);
            }
        }
        scan
    }

    /// Whether an identifier appears anywhere beyond the field declarations
    /// that introduce it.
    ///
    /// Every way to touch a field in Rust -- `x.f`, `Foo { f: .. }`,
    /// `let Foo { f, .. }`, `offset_of!(Foo, f)` -- requires the identifier as
    /// a literal token, so an identifier occurring no more often than it is
    /// declared is never read. Counting over the whole crate is broader than
    /// the module scope a private field is visible in, which makes the answer
    /// conservative: a collision with an unrelated name elsewhere only causes
    /// the field to keep its type.
    pub fn is_unused(&self, ident: &str) -> bool {
        let uses = self.ident_uses.get(ident).copied().unwrap_or(0);
        let decls = self.field_decls.get(ident).copied().unwrap_or(0);
        uses <= decls
    }

    pub fn is_literal_constructed(&self, struct_name: &str) -> bool {
        self.literal_constructed.contains(struct_name)
    }
}

impl<'ast> Visit<'ast> for CrateScan {
    fn visit_fields_named(&mut self, fields: &'ast syn::FieldsNamed) {
        for field in &fields.named {
            if let Some(ident) = &field.ident {
                *self.field_decls.entry(ident.to_string()).or_default() += 1;
            }
        }
        syn::visit::visit_fields_named(self, fields);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        if let Some(seg) = node.path.segments.last() {
            self.literal_constructed.insert(seg.ident.to_string());
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_pat_struct(&mut self, node: &'ast syn::PatStruct) {
        if let Some(seg) = node.path.segments.last() {
            self.literal_constructed.insert(seg.ident.to_string());
        }
        syn::visit::visit_pat_struct(self, node);
    }
}

fn count_idents(stream: proc_macro2::TokenStream, out: &mut HashMap<String, usize>) {
    for tree in stream {
        match tree {
            proc_macro2::TokenTree::Ident(i) => *out.entry(i.to_string()).or_default() += 1,
            proc_macro2::TokenTree::Group(g) => count_idents(g.stream(), out),
            _ => {}
        }
    }
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(rust_files(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            found.push(path);
        }
    }
    found
}

/// The nearest ancestor directory holding a `Cargo.toml`.
pub fn crate_root(file: &Path) -> Option<PathBuf> {
    let mut dir = file.parent()?;
    loop {
        if dir.join("Cargo.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// The crate's macro recursion limit, as declared in its root module.
///
/// `test_fields!` recurses once per entry, so a long register block needs
/// this raised. Several chip crates already do it -- see
/// `chips/apollo3/src/lib.rs`, `chips/earlgrey/src/lib.rs`.
pub fn recursion_limit(crate_root: &Path) -> u64 {
    const DEFAULT: u64 = 128;
    let Ok(text) = std::fs::read_to_string(crate_root.join("src/lib.rs")) else {
        return DEFAULT;
    };
    let Ok(file) = syn::parse_file(&text) else {
        return DEFAULT;
    };
    file.attrs
        .iter()
        .find(|a| a.path().is_ident("recursion_limit"))
        .and_then(|a| match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => s.value().parse().ok(),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or(DEFAULT)
}

/// How many entries a `register_structs!` block may hold at a given
/// recursion limit.
///
/// Measured against tock-registers 0.10.0 on the pinned toolchain: the
/// relation is exact, and holds whether the entries are typed fields or
/// padding markers. The `@END` marker is not counted.
pub fn max_entries(recursion_limit: u64) -> u64 {
    recursion_limit / 2 - 2
}
