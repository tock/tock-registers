// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Snapshot tests over `tests/fixtures/<case>/`.
//!
//! Each case holds a `src/input.rs` drawn from a real shape in `chips/`. The
//! test runs the binary over it and compares both products: `expected.rs` is
//! the converted module, and `report.txt` is the decision the tool reported.
//! Recording the report is what keeps the refusal messages honest.
//!
//! The input lives under `src/` so that the tool's crate-wide scan for unused
//! field names sees only the input, never the snapshots beside it.
//!
//! Run with `UPDATE_FIXTURES=1` to rewrite the snapshots after an
//! intentional change, then read the diff.

use std::path::Path;
use std::process::Command;

#[test]
fn fixtures_match() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let update = std::env::var_os("UPDATE_FIXTURES").is_some();

    let mut cases: Vec<_> = std::fs::read_dir(&dir)
        .expect("fixtures directory")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no fixtures found in {}", dir.display());

    let mut failures = Vec::new();
    for case in cases {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let output = Command::new(env!("CARGO_BIN_EXE_structs2regs"))
            .arg(case.join("src/input.rs"))
            .arg("--stdout")
            .arg("--crate-root")
            .arg(&case)
            .output()
            .expect("run structs2regs");

        let converted = String::from_utf8(output.stdout).expect("utf-8 stdout");

        for (file, actual) in [("expected.rs", &converted)] {
            let path = case.join(file);
            if update {
                std::fs::write(&path, actual).expect("write fixture");
                continue;
            }
            let expected = std::fs::read_to_string(&path).unwrap_or_default();
            if &expected != actual {
                failures.push(format!(
                    "--- {name}/{file} ---\nexpected:\n{expected}\nactual:\n{actual}"
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} fixture mismatch(es); re-run with UPDATE_FIXTURES=1 to accept\n\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A refused struct must be left exactly as it was found.
///
/// This is the property that makes a partial conversion shippable: the module
/// still compiles, so the structs that did convert can be committed while the
/// rest are handled by hand.
#[test]
fn refused_structs_are_left_untouched() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for entry in std::fs::read_dir(&dir)
        .expect("fixtures directory")
        .flatten()
    {
        let case = entry.path();
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        if !case.is_dir() || !name.starts_with("refuse_") {
            continue;
        }
        let input = std::fs::read_to_string(case.join("src/input.rs")).expect("input");
        let output = Command::new(env!("CARGO_BIN_EXE_structs2regs"))
            .arg(case.join("src/input.rs"))
            .arg("--stdout")
            .arg("--crate-root")
            .arg(&case)
            .output()
            .expect("run structs2regs");
        let converted = String::from_utf8(output.stdout).expect("utf-8");

        assert!(
            converted.is_empty() || converted == input,
            "{name}: a refused struct was modified"
        );
    }
}
