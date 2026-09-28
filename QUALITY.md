# Quality and coverage harness

## Scope

This is the required quality gate for Taytay. It applies to the Rust workspace, source adapters, spool/ledger, Lunsaran client, TUS uploader, configuration, and service packaging.

## Required checks

Run the narrowest relevant check first, then the complete workspace gate:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

The same checks are available through:

```bash
scripts/quality.sh
```

## Coverage

Line coverage is measured with `cargo llvm-cov` in CI and locally when the tool is installed:

```bash
scripts/coverage.sh
```

Coverage policy is staged because source adapters depend on hardware and network fixtures:

- `0.0.1`: coverage command and report generation established.
- `0.1.0`: at least 70% line coverage for spool, ledger, configuration, and lifecycle modules.
- `0.3.0`: at least 80% line coverage for the control-plane client, TUS uploader, retry state machine, and durable queue.
- `0.5.0`: at least 75% line coverage for source adapters, with protocol behavior covered by fixtures or integration tests where hardware is unavailable.
- `0.7.0`: at least 80% workspace line coverage, with all failure-state and security-boundary tests required regardless of percentage.

Coverage percentages are a floor, not a substitute for behavior tests. Do not exclude error handling, authorization boundaries, credential redaction, restart recovery, or offset reconciliation solely to improve the percentage.

## Test layers

### Unit tests

Use deterministic tests for state transitions, checksums, configuration validation, redaction, retry/backoff, idempotency, and TUS offset reconciliation.

### Component tests

Use temporary directories and isolated databases for the spool, ledger, cleanup, and recovery paths. Use fake source adapters and fake Lunsaran/Brutus HTTP servers.

### Protocol tests

Use fixtures for ONVIF, RTSP, V4L2, MAVLink, and source metadata. Keep real devices for compatibility and acceptance tests rather than making ordinary CI depend on them.

### End-to-end tests

Run local Lunsaran, Brutus, and MinIO/S3 fixtures to verify organization/project binding, resumability, retries, interruption recovery, duplicate prevention, and credential non-disclosure.

### Hardware acceptance

Maintain a compatibility matrix for tested cameras, NVRs, USB cameras, drones, and LiDAR devices. Hardware tests may be scheduled or manually triggered, but every supported adapter must have a deterministic fixture path for CI.

## Coverage exclusions

Only generated code, vendored code, and deliberately unreachable platform glue may be excluded. Every exclusion requires a comment and review. Do not exclude an adapter merely because a physical device is unavailable in CI; test its protocol parser and state machine with fixtures.

## CI artifacts

CI should retain, on failure and for successful main-branch runs:

- test output;
- LCOV coverage report;
- JUnit test report when supported;
- redacted integration logs;
- compatibility-test summaries.

No artifact may contain camera credentials, provider credentials, access tokens, signed URLs, media payloads, or unredacted source metadata.
