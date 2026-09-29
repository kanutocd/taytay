# Taytay

[![CI](https://github.com/kanutocd/taytay/actions/workflows/ci.yml/badge.svg)](https://github.com/kanutocd/taytay/actions/workflows/ci.yml)
[![Documentation](https://github.com/kanutocd/taytay/actions/workflows/docs.yml/badge.svg)](https://github.com/kanutocd/taytay/actions/workflows/docs.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-2563eb)](https://kanutocd.github.io/taytay/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)

Taytay is a Rust Linux edge bridge for durable, resumable Lunsaran uploads. It
turns completed output from cameras, NVRs, files, and field sources into
checksummed local jobs, then transfers them through Lunsaran and Brutus/TUS.

Taytay is for OEMs, Rust integrators, and teams building Linux edge workers. It
is a reusable crate and reference service boundary—not a hosted camera
platform. Lunsaran owns identity and organization/project authorization;
Brutus owns TUS byte transfer and provider execution.

## Use Taytay

Add the crate to an integrator:

```toml
[dependencies]
taytay = "0.0.1"
```

Start with the [consumer documentation](https://kanutocd.github.io/taytay/),
especially the [quick start](https://kanutocd.github.io/taytay/quickstart/),
[Rust crate guide](https://kanutocd.github.io/taytay/crate-guide/), and
[architecture guide](https://kanutocd.github.io/taytay/architecture/).

The crate provides:

- typed source, artifact, spool, job, uploader, and error contracts;
- atomic publication, SHA-256 verification, quota enforcement, and restart
  recovery;
- bounded scheduling, retry classification, cancellation, and durable resume
  state; and
- a `lunsaran-entregar` integration behind the `ArtifactUploader` boundary.

## Reference service

The binary validates TOML configuration and reports local readiness:

```bash
cargo run -- --check-config config/taytay.toml.example
cargo run -- --status config/taytay.toml.example
```

For deployment guidance, see [operations](docs/operations.md) and the example
configuration at [`config/taytay.toml.example`](config/taytay.toml.example).

## Scope

Taytay keeps source adapters behind small typed traits, so an integrator can
add filesystem/NVR, V4L2, RTSP, ONVIF, drone, or LiDAR sources without coupling
the queue to a vendor SDK. Real device compatibility and hosted Lunsaran
acceptance require external hardware or services.

Taytay never receives or persists customer storage-provider credentials. Source
credentials remain on the edge and must not appear in logs, metadata, metrics,
or diagnostics.

## Development

```bash
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
scripts/quality.sh
scripts/docs-quality.sh
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and
[CHANGELOG.md](CHANGELOG.md) for release notes.

## Project policies

- [Apache-2.0 license](LICENSE)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)
- [Documentation site](https://kanutocd.github.io/taytay/)
