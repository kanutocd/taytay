#!/usr/bin/env bash
set -euo pipefail

# Exercises the published lunsaran-entregar client and its deterministic local
# mock. This deliberately does not contact Lunsaran, Brutus, or object storage.
# The filtered suite covers happy path, checksum propagation, transient and
# stale-offset recovery, expiry classification, and cancellation.
cargo test --workspace --all-features entregar_uploader -- --nocapture
