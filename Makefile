# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# ------------------------------------------------------------------------------
# Developer tests.
# These actions are for use during tock-registers development.
# ------------------------------------------------------------------------------

# The default action: a very quick test intended to catch *most* issues and
# only take a few seconds.
.PHONY: test
test: miri_quick stable_checks
	@printf '%s%s\n%s\n%s%s\n' "$$(tput bold)" \
		'**********************' \
		'* Quick test passed! *' \
		'**********************' "$$(tput sgr0)"

# Runs all the tests that this Makefile can run. Takes longer, but useful
# before sending PRs. Note that the license checker is only run in CI, as it
# depends on the main Tock repository.
.PHONY: full
full: miri_full stable_checks
	@printf '%s%s\n%s\n%s%s\n' "$$(tput bold)" \
		'*********************' \
		'* Full test passed! *' \
		'*********************' "$$(tput sgr0)"

# ------------------------------------------------------------------------------
# Actions used by CI jobs.
# The CI jobs are divided based on which toolchain they run, to avoid redundant
# toolchain downloads while also preventing any one job from downloading
# multiple different toolchains.
# ------------------------------------------------------------------------------

# CI job for tests run using the nightly toolchain.
.PHONY: ci_nightly
ci_nightly: miri_full
	@printf '%s%s\n%s\n%s%s\n' "$$(tput bold)" \
		'***************************' \
		'* CI-nightly test passed! *' \
		'***************************' "$$(tput sgr0)"

# Main CI job, uses the stable toolchain.
.PHONY: ci_stable
ci_stable: stable_checks
	@printf '%s%s\n%s\n%s%s\n' "$$(tput bold)" \
		'**************************' \
		'* CI-stable test passed! *' \
		'**************************' "$$(tput sgr0)"

# Note that the license checker is *not* in this file, as that program is in
# the Tock repository.

# ------------------------------------------------------------------------------
# Test groups.
# These are groups referred to by other tests, to reduce duplication in the
# above targets. You can invoke these directly, but won't get the "X tests
# passed" output.
# ------------------------------------------------------------------------------

.PHONY: miri_full
miri_full: miri_32 miri_64

.PHONY: stable_checks
stable_checks: test_doc doc test_all build_all no_default_features \
               expand_macros_test clippy format_check

# ------------------------------------------------------------------------------
# Toolchain targets.
# Rustup currently lacks the locking needed for concurrent use:
# https://github.com/rust-lang/rustup/issues/988. In particular, running
# concurrent cargo commands with a missing toolchain results in parallel rustup
# instances installing the same toolchain, corrupting that toolchain. To
# mitigate that issue, every target that uses the main (MSRV) toolchain should
# depend transitively on the `toolchain` target, so that the toolchain is
# installed before it is invoked concurrently.
# ------------------------------------------------------------------------------
.PHONY: toolchain
toolchain:
	cargo -V

# The same as `toolchain`, but for the nightly toolchain.
.PHONY: nightly_toolchain
nightly_toolchain:
	cd nightly && cargo -V

# ------------------------------------------------------------------------------
# Individual checks.
# These should all depend on one of the toolchain targets. The developer
# actions and CI job actions depend on these.
# ------------------------------------------------------------------------------

.PHONY: build_all
build_all: toolchain
	RUSTFLAGS="-D warnings" cargo build --all-targets --workspace

.PHONY: clippy
clippy: toolchain
	RUSTFLAGS="-D warnings" cargo clippy --all --all-targets --workspace

# `cargo doc` seems to invalidate the Cargo cache for other commands (and
# vice-versa), so we set a different target directory to stop that
# invalidation.
.PHONY: doc
doc: toolchain
	RUSTDOCFLAGS="-D warnings" cargo doc --target-dir=target_doc --workspace

# Tests the expand_macros binary.
.PHONY: expand_macros_test
expand_macros_test: toolchain
	RUSTFLAGS="-D warnings" cargo run -p tock-registers-expand-macros \
		--release -- expand_macros/test/src/unexpanded.rs \
		> expand_macros/test/src/lib.rs
	RUSTFLAGS="-D warnings" cargo test \
		--manifest-path=expand_macros/test/Cargo.toml

.PHONY: format_check
format_check: toolchain
	cargo fmt --all --check

# Runs all tests in Miri on a 32-bit target.
.PHONY: miri_32
miri_32: nightly_toolchain
	cd nightly && RUSTFLAGS="-D warnings" \
		cargo miri test --all-targets --manifest-path=../Cargo.toml \
			--target=i686-unknown-linux-gnu --workspace

# Runs all tests in Miri on a 64-bit target.
.PHONY: miri_64
miri_64: nightly_toolchain
	cd nightly && RUSTFLAGS="-D warnings" \
		cargo miri test --all-targets --manifest-path=../Cargo.toml \
			--target=x86_64-unknown-linux-gnu --workspace

# A quick Miri smoke test.
.PHONY: miri_quick
miri_quick: nightly_toolchain
	cd nightly && RUSTFLAGS="-D warnings" \
		cargo miri test --manifest-path=../Cargo.toml --test=mmio

.PHONY: no_default_features
no_default_features: toolchain
	RUSTFLAGS="-D warnings" cargo build --no-default-features

.PHONY: test_all
test_all: toolchain
	RUSTFLAGS="-D warnings" cargo test --all-targets --workspace

.PHONY: test_doc
test_doc: toolchain
	RUSTFLAGS="-D warnings" cargo test --doc --workspace
