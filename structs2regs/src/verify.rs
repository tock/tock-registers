// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2026.

//! Running `cargo check` over the crate so the compiler, not this tool,
//! decides whether a conversion preserved the layout.

use std::path::Path;
use std::process::Command;

pub struct Outcome {
    pub label: String,
    pub ok: bool,
    pub output: String,
}

/// Check the crate once per configuration.
///
/// `features` holds the feature names a converted struct is gated on. The
/// default build covers every `not(feature = ..)` branch; one additional run
/// per feature covers the positive branches.
pub fn check_all(crate_root: &Path, features: &[String]) -> Vec<Outcome> {
    let mut outcomes = vec![run(crate_root, None)];
    for feature in features {
        outcomes.push(run(crate_root, Some(feature)));
    }
    outcomes
}

fn run(crate_root: &Path, feature: Option<&str>) -> Outcome {
    let label = match feature {
        None => "default features".to_string(),
        Some(f) => format!("--features {f}"),
    };
    let mut cmd = Command::new("cargo");
    cmd.arg("check")
        .arg("--manifest-path")
        .arg(crate_root.join("Cargo.toml"));
    if let Some(f) = feature {
        cmd.arg("--features").arg(f);
    }
    match cmd.output() {
        Ok(out) => Outcome {
            label,
            ok: out.status.success(),
            output: String::from_utf8_lossy(&out.stderr).into_owned(),
        },
        Err(e) => Outcome {
            label,
            ok: false,
            output: format!("could not run cargo: {e}"),
        },
    }
}

/// The feature name in a `feature = "x"` predicate, if that is its shape.
pub fn feature_of(predicate: &str) -> Option<String> {
    let rest = predicate.strip_prefix("feature")?.trim_start();
    let rest = rest.strip_prefix('=')?.trim();
    let inner = rest.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_string())
}
