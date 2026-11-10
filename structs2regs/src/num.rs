// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Offsets and sizes that may not be known as numbers.
//!
//! Most array lengths in `chips/` are integer literals, but a handful are
//! named constants or expressions over them (`[ReadWrite<u32>;
//! NUM_ENDPOINTS]`, `[u8; 0x1000 - (MAX_INTERRUPTS + 1) * 4]`).
//! `register_structs!` accepts an `expr` in the offset position, not just a
//! literal, so a length this tool cannot fold does not block a conversion --
//! the offset keeps a symbolic part and `test_fields!` evaluates it at
//! compile time.

/// A byte offset or size: a folded constant plus any terms that resisted
/// folding.
///
/// Keeping the constant separate from the symbolic terms is what lets
/// `0x20 + 4 * MAX_BIT_REGS` and a later `+ 4` combine into
/// `0x24 + 4 * MAX_BIT_REGS` rather than trailing a chain of addends.
///
/// Every term originates as a verbatim slice of the input file and is only
/// ever composed with other terms, never rewritten.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Num {
    constant: u64,
    terms: Vec<String>,
}

impl Num {
    pub fn constant(value: u64) -> Num {
        Num {
            constant: value,
            terms: Vec::new(),
        }
    }

    /// A value the tool could not fold, carried through as written.
    pub fn symbolic(text: impl Into<String>) -> Num {
        Num {
            constant: 0,
            terms: vec![text.into()],
        }
    }

    pub fn as_const(&self) -> Option<u64> {
        self.terms.is_empty().then_some(self.constant)
    }

    /// Render in the offset position of a `register_structs!` entry.
    ///
    /// Plain constants use at least three hex digits, matching existing
    /// blocks and `tools/svd2regs/svd2regs.py`.
    pub fn render_offset(&self) -> String {
        if self.terms.is_empty() {
            return format!("0x{:03X}", self.constant);
        }
        let terms = self.terms.join(" + ");
        if self.constant == 0 {
            terms
        } else {
            format!("0x{:X} + {terms}", self.constant)
        }
    }

    pub fn add(&self, other: &Num) -> Num {
        let mut terms = self.terms.clone();
        terms.extend(other.terms.iter().cloned());
        Num {
            constant: self.constant + other.constant,
            terms,
        }
    }

    /// Multiply, as an array length multiplies its element size.
    pub fn mul(&self, other: &Num) -> Num {
        match (self.as_const(), other.as_const()) {
            (Some(a), Some(b)) => Num::constant(a * b),
            // The usual case: a known element size times a symbolic count.
            (Some(scale), None) => other.scaled(scale),
            (None, Some(scale)) => self.scaled(scale),
            // Both symbolic. Rare enough that one opaque term is fine.
            (None, None) => Num::symbolic(format!("{} * {}", self.grouped(), other.grouped())),
        }
    }

    fn scaled(&self, scale: u64) -> Num {
        if scale == 0 {
            return Num::constant(0);
        }
        if scale == 1 {
            return self.clone();
        }
        Num {
            constant: self.constant * scale,
            terms: self
                .terms
                .iter()
                .map(|t| format!("{scale} * {}", group(t)))
                .collect(),
        }
    }

    /// Render as a single factor, parenthesized if it is a sum.
    fn grouped(&self) -> String {
        let text = self.render_offset();
        group(&text)
    }
}

fn group(text: &str) -> String {
    if text.contains('+') || text.contains('-') {
        format!("({text})")
    } else {
        text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Num;

    #[test]
    fn constants_fold() {
        assert_eq!(Num::constant(4).add(&Num::constant(8)).as_const(), Some(12));
        assert_eq!(Num::constant(4).mul(&Num::constant(6)).as_const(), Some(24));
    }

    #[test]
    fn offsets_render_with_three_hex_digits_minimum() {
        assert_eq!(Num::constant(4).render_offset(), "0x004");
        assert_eq!(Num::constant(0x14D8).render_offset(), "0x14D8");
    }

    #[test]
    fn constants_keep_folding_around_a_symbolic_term() {
        // chips/sifive/src/plic.rs: MAX_BIT_REGS resists folding, but the
        // literal offsets on either side of it must still combine.
        let size = Num::constant(4).mul(&Num::symbolic("MAX_BIT_REGS"));
        assert_eq!(size.render_offset(), "4 * MAX_BIT_REGS");

        let offset = Num::constant(0x20).add(&size).add(&Num::constant(4));
        assert_eq!(offset.render_offset(), "0x24 + 4 * MAX_BIT_REGS");
        assert_eq!(offset.as_const(), None);
    }

    #[test]
    fn sums_are_parenthesized_under_multiplication() {
        let count = Num::constant(2).add(&Num::symbolic("N"));
        assert_eq!(Num::constant(4).mul(&count).render_offset(), "0x8 + 4 * N");

        let both = Num::symbolic("A").mul(&Num::symbolic("B + C"));
        assert_eq!(both.render_offset(), "A * (B + C)");
    }

    #[test]
    fn identities_do_not_clutter_the_expression() {
        let n = Num::symbolic("N");
        assert_eq!(Num::constant(1).mul(&n), n);
        assert_eq!(Num::constant(0).mul(&n).as_const(), Some(0));
        assert_eq!(Num::constant(0).add(&n), n);
    }
}
