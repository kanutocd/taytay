# Contributing to Taytay

Taytay welcomes focused improvements to the public crate, reference service,
source boundaries, documentation, and deterministic tests.

Before changing code:

1. Inspect the relevant implementation, execution path, tests, and invariants.
2. Keep source adapters and upload policy behind the existing typed boundaries.
3. Update public documentation when behavior or contracts change.
4. Add a `CHANGELOG.md` entry for user-visible or architectural changes.

Run the complete local checks:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
scripts/quality.sh
scripts/docs-quality.sh
```

For upload changes, also run:

```bash
scripts/acceptance-mock.sh
```

Do not include credentials, signed URLs, media payloads, customer metadata, or
generated `site/` output in commits. Keep commits small and coherent. See the
[security policy](SECURITY.md) before reporting a vulnerability.
