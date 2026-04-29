# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

.PHONY: test
test: miri_test basic_test expand_macros_test

# Rustup currently lacks the locking needed for concurrent use:
# https://github.com/rust-lang/rustup/issues/988. In particular, running
# concurrent cargo commands with a missing toolchain results in parallel rustup
# instances installing the same toolchain, corrupting that toolchain. To
# mitigate that issue, every target that uses the main (MSRV) toolchain should
# depend transitively on the `toolchain` target, so that the toolchain is
# installed before it is invoked concurrently. Note that we don't need to do
# this for the nightly toolchain because the nightly toolchain is only used by
# the `miri_test` target, so this Makefile won't invoke it concurrently.
.PHONY: toolchain
toolchain:
	cargo -V

.PHONY: basic_test
basic_test: toolchain
	+RUSTFLAGS="-D warnings" cargo build --no-default-features
	+RUSTFLAGS="-D warnings" cargo build --all-targets --workspace
	+RUSTFLAGS="-D warnings" cargo test --all-targets --workspace
	+RUSTFLAGS="-D warnings" cargo clippy --all --all-targets --workspace
	+RUSTDOCFLAGS="-D warnings" cargo doc --workspace
	+cargo fmt --all --check

# Tests the expand_macros binary.
# Rust inhibits many `unused` warnings on code emitted by proc macros. However,
# this test bypasses the proc macro mechanism, so we have to do that allow
# ourselves. We also have to skip rustfmt so cango fmt --check doesn't fail.
.PHONY: expand_macros_test
expand_macros_test: toolchain
	echo '#![allow(unused)] #![cfg_attr(rustfmt, rustfmt::skip)]' \
		> tests/expanded.rs
	+RUSTFLAGS="-D warnings" cargo run -p tock-registers-expand-macros \
		--release -- tests/all_block_fields.rs >> tests/expanded.rs
	+RUSTFLAGS="-D warnings" cargo test --features=register_types \
		--no-default-features --test expanded

.PHONY: miri_test
miri_test:
	+cd nightly && RUSTFLAGS="-D warnings" \
		cargo miri test --all-targets --manifest-path=../Cargo.toml --workspace
