#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f Cargo.toml ]]; then
  echo "Taytay Rust workspace is not initialized yet; quality checks are defined in QUALITY.md."
  exit 0
fi

scripts/docs-quality.sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
