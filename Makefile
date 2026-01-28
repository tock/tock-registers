# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

.PHONY: test
test:
	+RUSTFLAGS="-D warnings" cargo build --no-default-features --workspace
	+RUSTFLAGS="-D warnings" cargo build --all-targets --workspace
	+RUSTFLAGS="-D warnings" cargo test --all-targets --workspace
	+RUSTFLAGS="-D warnings" cargo clippy --workspace
	+RUSTDOCFLAGS="-D warnings" cargo doc --workspace
	+cargo fmt --all --check
	+cd nightly && RUSTFLAGS="-D warnings" \
		cargo miri test --all-targets --manifest-path=../Cargo.toml --workspace
