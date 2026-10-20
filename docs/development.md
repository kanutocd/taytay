---
title: Development
description: Test and document Taytay integrations
---

# Development

## Toolchain and checks

Taytay targets stable Rust and declares its minimum supported toolchain in the
workspace manifest. Run the focused checks before sending an integration or
adapter change:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
scripts/quality.sh
scripts/coverage.sh
```

The coverage command depends on the local Rust/LLVM instrumentation toolchain;
report toolchain failures separately from test failures.

## Test an integrator boundary

The published `lunsaran-entregar-mock` crate provides deterministic local
control-plane and TUS behavior. Taytay’s acceptance script exercises normal
transfer, transient PATCH failure, stale offsets, checksum propagation, and
cancellation without network access to production services.

The cross-repository harness is deliberately opt-in because it requires a
provisioned Lunsaran API/PostgreSQL/Brutus environment and an enrolled device
credential. Run `scripts/acceptance-cross-repo.sh` with the documented
`TAYTAY_CROSS_REPO=1` environment when that disposable environment is
available.

When the live gate is provisioned, its acceptance matrix includes session
authorization, TUS completion, checksum propagation, durable restart recovery,
idempotency replay, project scoping, invalid credentials, and expired/revoked
credential rejection.

When adding an adapter, test at least:

- incomplete-file suppression;
- stable identifiers and metadata;
- reconnect or source-disconnect behavior;
- checksum and size preservation; and
- restart after publication.

When adding upload behavior, test:

- session scope validation;
- idempotent retries;
- authoritative offset recovery;
- authorization and expiry handling;
- checksum propagation and server-side checksum enforcement; and
- redaction of credentials and signed URLs.

For durable operator controls, use `Spool::pause` and `Spool::resume`. The
paused state is persisted and omitted from `Ledger::pending()` until resumed.

## Documentation workflow

Install Zensical in a local virtual environment:

```bash
python3 -m venv .venv-zensical
source .venv-zensical/bin/activate
python -m pip install zensical
```

Build the site from the repository root:

```bash
scripts/docs-quality.sh
```

This builds the Zensical site and lints Markdown structure, fenced code blocks,
trailing whitespace, and local links in one command.

Keep examples aligned with the current public API. Private planning documents
belong in the companion `taytay-saas` repository and should not be added to
this site’s navigation.
