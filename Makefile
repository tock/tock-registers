# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2026.

# The Miri test takes longer than all the other tests combined, and is mostly
# sequential (cargo runs individual test binaries sequentially). Therefore, to
# speed up `make test`, we split the Miri test and other tests into separate
# targets (miri_test and fast_tests) so they run in parallel.
.PHONY: test
test: miri_test fast_test

.PHONY: fast_test
fast_test:
	+RUSTFLAGS="-D warnings" cargo build --no-default-features
	+RUSTFLAGS="-D warnings" cargo build --all-targets --workspace
	+RUSTFLAGS="-D warnings" cargo test --all-targets --workspace
	+RUSTFLAGS="-D warnings" cargo clippy --all --all-targets --workspace
	+RUSTDOCFLAGS="-D warnings" cargo doc --workspace
	+cargo fmt --all --check

.PHONY: miri_test
miri_test:
	+cd nightly && RUSTFLAGS="-D warnings" \
		cargo miri test --all-targets --manifest-path=../Cargo.toml --workspace
