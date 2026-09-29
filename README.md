<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/branding/taytay-logo-dark.png">
    <img src="docs/branding/taytay-logo.png" alt="Taytay bridge logo" width="96" height="96">
  </picture>
</p>

# Taytay

[![CI](https://github.com/kanutocd/taytay/actions/workflows/ci.yml/badge.svg)](https://github.com/kanutocd/taytay/actions/workflows/ci.yml)
[![Documentation](https://github.com/kanutocd/taytay/actions/workflows/docs.yml/badge.svg)](https://github.com/kanutocd/taytay/actions/workflows/docs.yml)
[![Docs](https://img.shields.io/badge/docs-GitHub%20Pages-2563eb)](https://kanutocd.github.io/taytay/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)

Taytay is a Rust crate for Linux edge applications that need to deliver local
or device-produced artifacts to Lunsaran reliably. It provides durable local
publication, checksums, resumable jobs, bounded retries, and an uploader
boundary for Lunsaran and Brutus/TUS.

It is intended for OEMs, Rust integrators, and teams building edge workers for
cameras, NVRs, files, drones, or LiDAR. Taytay is not a hosted camera platform:
Lunsaran owns identity and project authorization, while Brutus owns TUS byte
transfer and provider execution.

*Taytay* is a Hiligaynon word for “bridge”, describing the crate’s role between
edge sources and the Lunsaran platform.

## Add Taytay

```toml
[dependencies]
lunsaran-taytay = "0.1.0"
```

The crate gives integrators:

- typed source, artifact, spool, job, uploader, and error contracts;
- atomic publication, SHA-256 verification, quota enforcement, and restart
  recovery;
- bounded scheduling, retry classification, cancellation, and resume state; and
- a `lunsaran-entregar` implementation behind the `ArtifactUploader` boundary.

## Documentation

Read the [Taytay documentation site](https://kanutocd.github.io/taytay/) for
the [quick start](https://kanutocd.github.io/taytay/quickstart/),
[crate guide](https://kanutocd.github.io/taytay/crate-guide/),
[architecture](https://kanutocd.github.io/taytay/architecture/),
[durability](https://kanutocd.github.io/taytay/durability/), and
[upload integration](https://kanutocd.github.io/taytay/uploads/).

Taytay keeps source credentials on the edge and never receives or persists
customer storage-provider credentials. See the [security policy](SECURITY.md)
and [security guide](https://kanutocd.github.io/taytay/security/) before
deployment.

## Policies

- [Apache-2.0 license](LICENSE)
- [Contributing](CONTRIBUTING.md)
- [Security reporting](SECURITY.md)
