# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

.PHONY: test
test:
	+RUSTFLAGS="-D warnings" cargo build --all-targets --no-default-features
	+RUSTFLAGS="-D warnings" cargo build --all-targets
	+RUSTFLAGS="-D warnings" cargo test
	+RUSTFLAGS="-D warnings" cargo clippy
	+cargo fmt --check
	+cd nightly && \
		RUSTFLAGS="-D warnings" cargo miri test --manifest-path=../Cargo.toml
