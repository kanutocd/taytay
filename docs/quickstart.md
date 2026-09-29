---
title: Quick start
description: Add Taytay to a Rust edge integrator
---

# Quick start

## Add the crate

The public workspace currently publishes the `taytay` crate as the reusable
edge boundary. Pin the version selected by your integration review:

```toml
[dependencies]
taytay = "0.0.1"
```

The reference uploader uses the published `lunsaran-entregar` client. Your
application may instead implement `ArtifactUploader` for an internal transport
or a different control-plane adapter.

## Publish an artifact locally

`Spool::publish` writes the bytes to a temporary file, syncs them, atomically
publishes the final artifact, computes SHA-256, and inserts a pending ledger
job.

```rust
use taytay::{ArtifactId, SourceId, spool::Spool};

let spool = Spool::open("./var/taytay-spool", 10 * 1024 * 1024 * 1024)?;
let job = spool.publish(
    ArtifactId::new("camera-01-2026-09-29T12-00-00Z"),
    SourceId::new("camera-01"),
    "video/mp4".to_owned(),
    encoded_segment,
    Some("2026-09-29T12:00:00Z".to_owned()),
    serde_json::json!({ "camera": "camera-01" }),
)?;
```

Keep the returned `UploadJob` until the uploader reports completion. A source
adapter should not delete or mutate the published file while the job is
pending.

## Run the reference diagnostics

The example configuration is in `config/taytay.toml.example`:

```bash
cp config/taytay.toml.example /tmp/taytay.toml
install -m 600 /dev/null /tmp/lunsaran.token
printf '%s\n' "$LUNSARAN_DEVICE_TOKEN" > /tmp/lunsaran.token
sed -i 's#^token_file = .*#token_file = "/tmp/lunsaran.token"#' /tmp/taytay.toml

cargo run -- --check-config /tmp/taytay.toml
cargo run -- --status /tmp/taytay.toml
```

`--check-config` validates the file without starting transfer. `--status`
reports JSON readiness and spool counts without exposing credentials or media
metadata.

The reference binary also supports `--pause ARTIFACT_ID CONFIG` and
`--resume ARTIFACT_ID CONFIG` for durable upload control.

## Choose an integration path

| You are building… | Start with… |
| --- | --- |
| A Rust source adapter | `SourceAdapter`, `Artifact`, and `Spool` |
| A custom transport | `ArtifactUploader` and `UploadError` |
| A Lunsaran edge worker | `EntregarUploader`, credential checks, and the scheduler |
| A service deployment | the reference binary, TOML config, systemd unit, and operations guide |

For a real Lunsaran deployment you also need an organization/project scope and
a device or workload credential issued by Lunsaran. Taytay does not provision
users or provider accounts.
