---
title: Operations
description: Configure, deploy, and observe Taytay
---

# Operations

## Configuration

The reference binary reads TOML:

```toml
spool_dir = "/var/lib/taytay/spool"
quota_bytes = 10737418240
upload_workers = 2
lunsaran_base_url = "https://lunsaran.example"
organization_id = "org-example"
project_id = "project-example"
token_file = "/etc/taytay/lunsaran.token"
```

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

This covers the happy path, transient PATCH failure, stale-offset recovery,
and pre-flight cancellation against a local mock. It does not replace a hosted
Lunsaran/Brutus acceptance environment or hardware testing.
