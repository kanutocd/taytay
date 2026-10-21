---
title: Operations
description: Configure, deploy, and observe Taytay
---

# Operations

This guide is for operators deploying the Taytay reference binary on a Linux
edge host. It covers configuration, service execution, diagnostics, and
acceptance checks. It does not provision Lunsaran organizations, projects, or
device credentials.

## Configuration

The reference binary reads a TOML file. The following fields are required:

```toml
spool_dir = "/var/lib/taytay/spool"
quota_bytes = 10737418240
upload_workers = 2
lunsaran_base_url = "https://lunsaran.example"
organization_id = "00000000-0000-0000-0000-000000000001"
project_id = "00000000-0000-0000-0000-000000000002"
token_file = "/etc/taytay/lunsaran.token"
```

| Field | Required | Meaning and validation |
| --- | --- | --- |
| `spool_dir` | Yes | Persistent local artifact and ledger directory. |
| `quota_bytes` | No | Maximum spool size. Defaults to 10 GiB. Must be positive. |
| `upload_workers` | No | Concurrent workers. Defaults to 2. Must be positive. |
| `lunsaran_base_url` | Yes | HTTP(S) origin without credentials, query, or fragment. |
| `organization_id` | Yes | UUID used for upload-session scope. |
| `project_id` | Yes | UUID used for upload-session scope. |
| `token_file` | Yes | Protected, non-empty credential file with no parent traversal. |

Use a protected token file. Taytay rejects empty secrets, parent traversal in
the configured token path, credential-bearing base URLs, query strings, and
fragments.

## Service layout

The repository includes `packaging/taytay.service` as a starting point for an
unprivileged systemd deployment. Keep configuration and credentials owned by
root and readable only by the service account; keep the spool writable only by
Taytay.

```bash
taytay --check-config /etc/taytay/taytay.toml
taytay --status /etc/taytay/taytay.toml
systemctl enable --now taytay
systemctl status taytay
```

`--status` emits machine-readable readiness, pending/completed job counts, and
spool usage. It does not print tokens, URLs containing credentials, filenames,
or media metadata.

To run the upload workers against pending jobs, pass the configuration path as
the command. The process loads the protected token, drains the durable pending
snapshot with bounded concurrency and retries, and exits after the snapshot is
complete:

```bash
taytay /etc/taytay/taytay.toml
```

An interrupted upload remains recoverable in the ledger and is picked up by a
later invocation. Authorization failures and retry exhaustion leave a safe
failure state in the ledger and return a non-zero exit status.

Pause or resume a durable artifact by ID when an operator needs to stop retry
activity without deleting the local artifact:

```bash
taytay --pause ARTIFACT_ID /etc/taytay/taytay.toml
taytay --resume ARTIFACT_ID /etc/taytay/taytay.toml
```

These commands update the ledger atomically. A paused artifact is excluded
from pending scheduling and returns to `Published` when resumed.

## Metrics and alerts

The library exposes Prometheus-compatible counters for uploads, retries,
transferred bytes, and source errors. Keep labels low-cardinality and secret-
free: do not label metrics with artifact paths, signed URLs, device tokens, or
customer metadata.

Alert on:

- sustained spool growth or quota pressure;
- repeated authorization failures;
- retry exhaustion;
- source disconnects; and
- a worker that stops making progress.

## Acceptance

Run the deterministic published-client acceptance tests locally:

```bash
scripts/acceptance-mock.sh
```

This covers the happy path, checksum propagation, transient PATCH failure,
stale-offset recovery, expired-session classification, and pre-flight
cancellation against a local mock. It does not replace a hosted
Lunsaran/Brutus acceptance environment or hardware testing.

For a provisioned disposable Lunsaran API/PostgreSQL/Brutus environment, run
the opt-in cross-repository acceptance harness:

```bash
TAYTAY_CROSS_REPO=1 \
TAYTAY_LUNSARAN_BASE_URL=http://127.0.0.1:8080 \
TAYTAY_LUNSARAN_TOKEN_FILE=/secure/path/device.token \
TAYTAY_ORGANIZATION_ID=ORG_UUID \
TAYTAY_PROJECT_ID=PROJECT_UUID \
TAYTAY_LUNSARAN_COMPOSE_FILES="/path/lunsaran/infra/compose.yaml /path/lunsaran/infra/compose.dev.yaml" \
TAYTAY_BRUTUS_COMPOSE_FILE=/path/brutus/docker-compose.yaml \
scripts/acceptance-cross-repo.sh
```

The credential must already be enrolled for the project. The harness uses an
isolated Compose project, creates one disposable ledger artifact, verifies
completion and redacted output, and removes its stack and temporary files on
exit. Set `TAYTAY_KEEP_CROSS_REPO_STACK=1` only when inspecting a failed
environment manually. If the supplied API and Brutus stack is already running,
set `TAYTAY_USE_EXISTING_STACK=1`; this mode never stops containers.
On failure, the harness preserves its redacted output and ledger under `/tmp`
for diagnosis; remove that directory after inspection.

The live gate also replays an idempotency key, resumes a durable `Uploading`
ledger job, and verifies safe rejection of invalid, expired, revoked, and
project-ineligible credentials.
