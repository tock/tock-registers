// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Convert hand-written `#[repr(C)]` MMIO register structs in `chips/` to the
//! `register_structs!` macro from `tock-registers`, one module at a time.
//!
//! # Why the output can be trusted
//!
//! 1. Every token emitted for a field -- doc comments, attributes,
//!    visibility, name and type -- is a byte-exact slice of the input file.
//!    The tool computes only the offsets and which fields become padding.
//! 2. `register_structs!` expands to `test_fields!`, which asserts every
//!    field's start offset, alignment and end offset plus the struct's total
//!    size at compile time. A miscomputed offset is a build failure.
//! 3. `test_fields!` never calls `offset_of!`, so it cannot see a dropped
//!    field, a reordering, a same-size type swap or an alignment change. The
//!    tool therefore emits a temporary module holding the original struct and
//!    comparing it field by field, runs `cargo check`, and removes that
//!    module only once the compiler has accepted it.

use std::ops::Range;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use layout::Block;

mod collapse;
mod crate_scan;
mod emit;
mod imports;
mod layout;
mod num;
mod select;
mod sizes;
mod verify;

struct Converted {
    plain: String,
    checked: String,
    predicates: Vec<String>,
}

impl Converted {
    fn convert(
        item: &syn::ItemStruct,
        resolver: &sizes::Resolver,
        scan: &crate_scan::CrateScan,
        recursion_limit: u64,
    ) -> Result<Converted, String> {
        let plan = select::plan(item, scan)?;

        let mut plain = String::new();
        let mut checked = String::new();
        let mut predicates = Vec::new();

        fn check_block(
            block: &Block,
            decisions: &Vec<bool>,
            recursion_limit: u64,
        ) -> Result<(), String> {
            // When every register is byte-wide the block's alignment is 1, and
            // `test_fields!` expands its size check to `$size % 1`, which
            // `clippy::modulo_one` denies. The lint fires inside the macro
            // expansion, where an `#[allow]` on the invocation does not reach
            // it, so such a block cannot pass `make clippy`.
            if block.align == 1 {
                return Err(
                    "every register is byte-wide, so the block's alignment is 1 and \
                `test_fields!` expands its size check to `% 1`, which `clippy::modulo_one` \
                denies inside the macro; this needs a fix in tock-registers"
                        .to_string(),
                );
            }

            // `test_fields!` recurses once per entry, so a long block exceeds
            // rustc's limit before it ever gets to check the offsets.
            let entries = block.placed.len() as u64;
            let allowed = crate_scan::max_entries(recursion_limit);
            if entries > allowed {
                let needed = [256u64, 512, 1024, 2048, 4096]
                    .into_iter()
                    .find(|l| crate_scan::max_entries(*l) >= entries)
                    .unwrap_or(8192);
                return Err(format!(
                    "block has {entries} entries but the crate's recursion limit of \
                {recursion_limit} allows {allowed}; add `#![recursion_limit = \"{needed}\"]` \
                to the crate root and re-run"
                ));
            }

            if block
                .placed
                .iter()
                .zip(decisions)
                .all(|(_, collapsed)| *collapsed)
            {
                // `test_fields!` divides by the maximum alignment of the typed
                // fields, so an all-padding struct fails to const-evaluate.
                return Err("no typed fields would remain after padding collapse".to_string());
            }
            Ok(())
        }

        match plan {
            select::Plan::NoCfg(fields) => {
                let block = layout::place(&fields, resolver, &mut Vec::new())?;
                let decisions = collapse::decide(&block.placed, scan, block.align);

                check_block(&block, &decisions, recursion_limit)?;

                let invocation = emit::invocation(item, &block, &decisions, None, resolver);
                let check = emit::layout_check(item, &block, None, resolver);

                plain.push_str(&invocation);
                checked.push_str(&invocation);
                checked.push('\n');
                checked.push_str(&check);
            }
            select::Plan::WithCfg { cfg, with, without } => {
                let block = layout::place(&with, resolver, &mut Vec::new())?;
                let decisions = collapse::decide(&block.placed, scan, block.align);
                check_block(&block, &decisions, recursion_limit)?;

                predicates.push(cfg.clone());

                let invocation = emit::invocation(item, &block, &decisions, Some(&cfg), resolver);
                let check = emit::layout_check(item, &block, Some(&cfg), resolver);

                plain.push_str(&invocation);
                checked.push_str(&invocation);
                checked.push('\n');
                checked.push_str(&check);

                plain.push('\n');
                checked.push('\n');

                let block = layout::place(&without, resolver, &mut Vec::new())?;
                let decisions = collapse::decide(&block.placed, scan, block.align);
                check_block(&block, &decisions, recursion_limit)?;

                let cfg = format!("not({cfg})");
                predicates.push(cfg.clone());

                let invocation = emit::invocation(item, &block, &decisions, Some(&cfg), resolver);
                let check = emit::layout_check(item, &block, Some(&cfg), resolver);

                plain.push_str(&invocation);
                checked.push_str(&invocation);
                checked.push('\n');
                checked.push_str(&check);

                plain.push_str("\n\n");
                checked.push_str("\n\n");
            }
        }

        Ok(Converted {
            plain,
            checked,
            predicates,
        })
    }
}

/// Replace each struct's source range, back to front so earlier
/// offsets stay valid.
///
/// * `source` - original file source
/// * `edit` - List of edit range, plain, and checked, ordered from
///   latest to earliest in the file.
/// * `with_check` - whether to include a conversion validation module
fn splice(source: &str, edits: &[(Range<usize>, String, String)], with_check: bool) -> String {
    let mut out = source.to_string();
    for (range, plain, checked) in edits.iter() {
        let text = if with_check { checked } else { plain };
        out.replace_range(range.clone(), text.trim_end_matches('\n'));
    }
    out
}

#[derive(Parser)]
#[command(
    about = "Convert manual #[repr(C)] register structs to register_structs!",
    long_about = None,
)]
struct Args {
    /// The module to convert.
    path: PathBuf,

    /// Print the converted module instead of rewriting it. Implies --no-verify.
    #[arg(long)]
    stdout: bool,

    /// Skip the layout-equivalence check.
    #[arg(long)]
    no_verify: bool,

    /// Leave the layout-equivalence module in the file instead of removing
    /// it once `cargo check` accepts it. Useful for inspecting the proof.
    #[arg(long, conflicts_with = "no_verify")]
    keep_layout_check: bool,

    /// Treat this directory as the crate root instead of searching upward
    /// for a Cargo.toml. Intended for tests.
    #[arg(long, value_name = "DIR")]
    crate_root: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    run(&args).unwrap_or_else(|message| {
        eprintln!("error: {message}");
        ExitCode::FAILURE
    })
}

fn run(args: &Args) -> Result<ExitCode, String> {
    let source =
        std::fs::read_to_string(&args.path).map_err(|e| format!("{}: {e}", args.path.display()))?;
    let file = syn::parse_file(&source).map_err(|e| format!("{}: {e}", args.path.display()))?;

    // Find target items to convert
    let target_items = select::filter_target_items(&file);

    let crate_root = match &args.crate_root {
        Some(dir) => dir.clone(),
        None => crate_scan::crate_root(&args.path)
            .ok_or_else(|| format!("no Cargo.toml above {}", args.path.display()))?,
    };
    let scan = crate_scan::CrateScan::run(&crate_root);
    let recursion_limit = crate_scan::recursion_limit(&crate_root);
    let resolver = sizes::Resolver::new(&source, &file);

    // (byte range, text without the check module, text with it)
    let mut edits: Vec<(Range<usize>, String, String)> = Vec::new();
    let mut features: Vec<String> = Vec::new();

    let conversions = target_items.into_iter().filter_map(|(item, span)| {
        Converted::convert(item, &resolver, &scan, recursion_limit)
            .ok()
            .map(|converted| (converted, span))
    });

    // collect coversions
    for (converted, span) in conversions {
        for predicate in &converted.predicates {
            if let Some(f) = verify::feature_of(predicate)
                && !features.contains(&f)
            {
                features.push(f);
            }
        }
        edits.push((span.byte_range(), converted.plain, converted.checked));
    }

    if edits.is_empty() {
        eprintln!("no convertible register structs in {}", args.path.display());
    }

    // a converted module will need `register_structs!` imported.
    if !edits.is_empty() {
        if let Some((range, text)) = imports::ensure_import(&source, &file) {
            edits.push((range, text.clone(), text));
        }
    }

    // `splice` requires edits to be ordered from latest to earliest
    // in the file.
    edits.sort_by_key(|(range, _, _)| range.start);
    edits.reverse();

    // Splice in the edits
    let mut failed = false;
    let plain = splice(&source, &edits, false);
    if args.stdout {
        print!("{plain}");
    } else {
        let verifying = !args.no_verify && !edits.is_empty();
        if !verifying {
            std::fs::write(&args.path, plain)
                .map_err(|e| format!("{}: {e}", args.path.display()))?;
        } else {
            std::fs::write(&args.path, splice(&source, &edits, true))
                .map_err(|e| format!("{}: {e}", args.path.display()))?;
            let outcomes = verify::check_all(&crate_root, &features);
            eprintln!("verifying layout equivalence ({})", crate_root.display());
            for outcome in &outcomes {
                eprintln!(
                    "  {:<24} ... {}",
                    outcome.label,
                    if outcome.ok { "PASS" } else { "FAIL" }
                );
            }
            if outcomes.iter().all(|o| o.ok) {
                eprintln!(
                    "  {}/{} configurations verified",
                    outcomes.len(),
                    outcomes.len()
                );
                if args.keep_layout_check {
                    eprintln!("  layout check kept in {}", args.path.display());
                } else {
                    std::fs::write(&args.path, plain)
                        .map_err(|e| format!("{}: {e}", args.path.display()))?;
                }
            } else {
                failed = true;
                for outcome in outcomes.iter().filter(|o| !o.ok) {
                    eprintln!("\n--- {} ---\n{}", outcome.label, outcome.output);
                }
                eprintln!(
                    "the layout check is still in {} so you can iterate; `git checkout` to undo",
                    args.path.display()
                );
            }
        }
    }

    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
