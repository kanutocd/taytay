#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
  echo "Taytay Rust workspace is not initialized yet; coverage policy is defined in QUALITY.md."
  exit 0
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov is required for coverage. Install it with: cargo install cargo-llvm-cov" >&2
  exit 1
fi

mkdir -p target/coverage
cargo llvm-cov --workspace --all-features --lcov --output-path target/coverage/lcov.info
cargo llvm-cov --workspace --all-features --summary-only
