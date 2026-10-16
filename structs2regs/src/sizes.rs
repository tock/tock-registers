// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! The closed table mapping field types to their size and alignment.
//!
//! This table is deliberately small and exhaustive: a type it does not
//! recognize makes the whole struct ineligible rather than being guessed at.
//! If an entry here were wrong, the `test_fields!` assertions that
//! `register_structs!` generates would reject the output at compile time, so
//! this table is checked rather than trusted.

use std::collections::{HashMap, HashSet};

use crate::layout;
use crate::num::Num;

/// Size and alignment of a field type, in bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub size: Num,
    pub align: u64,
}

/// Wrapper types from `tock_registers::registers`, each `#[repr(transparent)]`
/// over an `UnsafeCell<T>`, which is itself `#[repr(transparent)]` over `T`
/// (tock-registers 0.10.0 `src/registers.rs:43,80,108,144,179`). They
/// therefore have exactly the size and alignment of `T`.
const REGISTER_WRAPPERS: &[&str] = &[
    "ReadOnly",
    "ReadWrite",
    "WriteOnly",
    "Aliased",
    "InMemoryRegister",
];

/// The subset of `REGISTER_WRAPPERS` that denotes a memory-mapped register.
///
/// `InMemoryRegister` is excluded on purpose: it marks in-memory DMA
/// structures (`chips/virtio/src/queues/split_queue.rs`), not an MMIO block.
const MMIO_WRAPPERS: &[&str] = &["ReadOnly", "ReadWrite", "WriteOnly", "Aliased"];

/// Whether a type is, or is an array of, a memory-mapped register.
pub fn is_mmio_register(ty: &syn::Type) -> bool {
    match ty {
        // Is the last segment of the path one of [MMIO_WRAPPERS]?
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .filter(|seg| MMIO_WRAPPERS.contains(&seg.ident.to_string().as_str()))
            .is_some(),
        syn::Type::Array(a) => is_mmio_register(&a.elem),
        _ => false,
    }
}

/// Scalars whose size and alignment are the same on every target Tock builds
/// for.
///
/// `usize`/`isize` are deliberately absent. `make ci-job-chips` compiles chip
/// crates for the 64-bit host while real builds target 32-bit, so a `usize`
/// field would have a different layout in the very configuration where the
/// generated assertions are evaluated.
fn scalar_layout(name: &str) -> Option<(u64, u64)> {
    match name {
        "u8" | "i8" => Some((1, 1)),
        "u16" | "i16" => Some((2, 2)),
        "u32" | "i32" => Some((4, 4)),
        "u64" | "i64" => Some((8, 8)),
        _ => None,
    }
}

pub struct Resolver<'a> {
    src: &'a str,
    structs: HashMap<String, &'a syn::ItemStruct>,
    /// File-local `const NAME: <int> = <expr>;` eligible for folding.
    consts: HashMap<String, &'a syn::Expr>,
    /// File-local `type NAME = ..;`, e.g. `Uart16550RegshiftInt` in
    /// `chips/qemu_virt_chip/src/uart.rs`.
    aliases: HashMap<String, &'a syn::Type>,
}

impl<'a> Resolver<'a> {
    pub fn new(src: &'a str, file: &'a syn::File) -> Self {
        let mut structs = HashMap::new();
        let mut aliases = HashMap::new();
        let mut consts: HashMap<String, &syn::Expr> = HashMap::new();
        let mut ambiguous: HashSet<String> = HashSet::new();

        for item in &file.items {
            match item {
                syn::Item::Struct(s) => {
                    structs.insert(s.ident.to_string(), s);
                }
                syn::Item::Type(t) if t.generics.params.is_empty() => {
                    aliases.insert(t.ident.to_string(), &*t.ty);
                }
                syn::Item::Const(c) => {
                    let name = c.ident.to_string();
                    // A constant with a cfg attribute, or defined more than
                    // once, has no single value to fold. `NUM_GPIOTE` in
                    // chips/nrf5x/src/gpio.rs has three such definitions.
                    let cfgd = c.attrs.iter().any(|a| a.path().is_ident("cfg"));
                    if cfgd || consts.contains_key(&name) {
                        ambiguous.insert(name.clone());
                    } else {
                        consts.insert(name, &c.expr);
                    }
                }
                _ => {}
            }
        }
        for name in ambiguous {
            consts.remove(&name);
        }

        Resolver {
            src,
            structs,
            consts,
            aliases,
        }
    }

    /// The exact source text a span covers.
    pub fn verbatim(&self, span: proc_macro2::Span) -> &'a str {
        &self.src[span.byte_range()]
    }

    /// `stack` carries the chain of nested struct names, for cycle detection.
    pub(crate) fn resolve_in(
        &self,
        ty: &syn::Type,
        stack: &mut Vec<String>,
    ) -> Result<Layout, String> {
        match ty {
            syn::Type::Array(a) => {
                let elem = self.resolve_in(&a.elem, stack)?;
                let len = match self.fold(&a.len, 0) {
                    Some(n) => Num::constant(n),
                    // Not foldable: carry the length through verbatim and let
                    // the compiler check the resulting offsets.
                    None => Num::symbolic(self.verbatim(syn::spanned::Spanned::span(&a.len))),
                };
                Ok(Layout {
                    size: elem.size.mul(&len),
                    align: elem.align,
                })
            }
            syn::Type::Path(p) => {
                if p.qself.is_some() {
                    return Err(format!("qualified path type `{}`", self.ty_text(ty)));
                }
                let seg = p
                    .path
                    .segments
                    .last()
                    .ok_or_else(|| "empty type path".to_string())?;
                let name = seg.ident.to_string();

                if matches!(name.as_str(), "usize" | "isize") {
                    return Err(format!(
                        "`{name}` is target-dependent; chip crates are checked on a 64-bit \
                         host but built for 32-bit targets"
                    ));
                }

                if REGISTER_WRAPPERS.contains(&name.as_str()) {
                    let inner = first_type_arg(seg)
                        .ok_or_else(|| format!("`{name}` without a value-type argument"))?;
                    let (size, align) = self.scalar_of(inner, 0).ok_or_else(|| {
                        format!(
                            "`{name}<{}, ..>` is not over a fixed-width integer",
                            self.ty_text(inner)
                        )
                    })?;
                    return Ok(Layout {
                        size: Num::constant(size),
                        align,
                    });
                }

                if let Some((size, align)) = self.scalar_of(ty, 0) {
                    return Ok(Layout {
                        size: Num::constant(size),
                        align,
                    });
                }

                // A register block nested in the same file, e.g. `Register` in
                // chips/sam4l/src/gpio.rs or `AdcEventChRegisters` in
                // chips/nrf52/src/adc.rs.
                if !matches!(seg.arguments, syn::PathArguments::None) {
                    return Err(format!("generic type `{}`", self.ty_text(ty)));
                }
                self.nested_struct(&name, stack)
            }
            _ => Err(format!("unsupported type `{}`", self.ty_text(ty))),
        }
    }

    /// Resolve a type to a fixed-width integer, following file-local `type`
    /// aliases.
    fn scalar_of(&self, ty: &syn::Type, depth: u32) -> Option<(u64, u64)> {
        if depth > 8 {
            return None;
        }
        let syn::Type::Path(path) = ty else {
            return None;
        };
        let ident = path.path.get_ident()?.to_string();
        if let Some(found) = scalar_layout(&ident) {
            return Some(found);
        }
        self.scalar_of(self.aliases.get(&ident)?, depth + 1)
    }

    fn nested_struct(&self, name: &str, stack: &mut Vec<String>) -> Result<Layout, String> {
        let Some(item) = self.structs.get(name) else {
            return Err(format!("type `{name}` is not defined in this module"));
        };
        if stack.iter().any(|n| n == name) {
            return Err(format!("type `{name}` is recursive"));
        }
        if !crate::select::has_plain_repr_c(&item.attrs) {
            return Err(format!("nested type `{name}` is not `#[repr(C)]`"));
        }
        if !item.generics.params.is_empty() {
            return Err(format!("nested type `{name}` is generic"));
        }
        let syn::Fields::Named(named) = &item.fields else {
            return Err(format!("nested type `{name}` has no named fields"));
        };
        if named
            .named
            .iter()
            .any(|f| crate::select::cfg_predicate(&f.attrs).is_some())
        {
            return Err(format!("nested type `{name}` has a cfg-gated field"));
        }

        stack.push(name.to_string());
        let fields: Vec<&syn::Field> = named.named.iter().collect();
        let block = layout::place(&fields, self, stack);
        stack.pop();
        let block = block?;

        // A nested struct's size has to be a number: it is multiplied by an
        // array length and rounded up to an alignment.
        let end = block
            .end
            .as_const()
            .ok_or_else(|| format!("nested type `{name}` has a symbolic size"))?;
        Ok(Layout {
            size: Num::constant(round_up(end, block.align)),
            align: block.align,
        })
    }

    /// Constant-fold an array length. `None` means "emit it verbatim instead".
    ///
    /// Covers integer literals, file-local constants, parentheses, the five
    /// arithmetic operators, and `size_of::<uN>()`. Method calls are not
    /// folded, so `MAX_INTERRUPTS.div_ceil(32)` stays symbolic.
    fn fold(&self, expr: &syn::Expr, depth: u32) -> Option<u64> {
        if depth > 16 {
            return None;
        }
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) => i.base10_parse::<u64>().ok(),
            syn::Expr::Paren(p) => self.fold(&p.expr, depth + 1),
            syn::Expr::Group(g) => self.fold(&g.expr, depth + 1),
            syn::Expr::Cast(c) => self.fold(&c.expr, depth + 1),
            syn::Expr::Path(p) => {
                let name = p.path.get_ident()?.to_string();
                self.fold(self.consts.get(&name)?, depth + 1)
            }
            syn::Expr::Binary(b) => {
                let l = self.fold(&b.left, depth + 1)?;
                let r = self.fold(&b.right, depth + 1)?;
                match b.op {
                    syn::BinOp::Add(_) => l.checked_add(r),
                    syn::BinOp::Sub(_) => l.checked_sub(r),
                    syn::BinOp::Mul(_) => l.checked_mul(r),
                    syn::BinOp::Div(_) => l.checked_div(r),
                    syn::BinOp::Rem(_) => l.checked_rem(r),
                    _ => None,
                }
            }
            // `size_of::<u32>()`, as used in chips/stm32u5xx/src/rsa.rs.
            syn::Expr::Call(c) => {
                let syn::Expr::Path(p) = &*c.func else {
                    return None;
                };
                let seg = p.path.segments.last()?;
                if seg.ident != "size_of" || !c.args.is_empty() {
                    return None;
                }
                let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
                    return None;
                };
                let syn::GenericArgument::Type(syn::Type::Path(tp)) = args.args.first()? else {
                    return None;
                };
                scalar_layout(&tp.path.segments.last()?.ident.to_string()).map(|(s, _)| s)
            }
            _ => None,
        }
    }

    fn ty_text(&self, ty: &syn::Type) -> String {
        self.verbatim(syn::spanned::Spanned::span(ty)).to_string()
    }
}

fn first_type_arg(seg: &syn::PathSegment) -> Option<&syn::Type> {
    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };
    args.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t),
        _ => None,
    })
}

/// Whether a type is a bare fixed-width integer, or an array of them.
///
/// Only such a field is a candidate for losing its type: anything wrapped in
/// a register type, and any nested register block, carries information about
/// the hardware that padding would discard.
pub fn is_plain_integer_storage(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Array(a) => is_plain_integer_storage(&a.elem),
        syn::Type::Path(p) => p
            .path
            .get_ident()
            .is_some_and(|i| scalar_layout(&i.to_string()).is_some()),
        _ => false,
    }
}

pub fn round_up(value: u64, align: u64) -> u64 {
    debug_assert!(align.is_power_of_two());
    value.div_ceil(align) * align
}
