---
title: Troubleshooting
description: Diagnose Taytay configuration, spool, and upload failures
---

# Troubleshooting

Use this page when the `taytay` reference binary does not become ready, does
not drain pending jobs, or exits with failed uploads. Run diagnostics from the
same service account and with the same configuration file used by the service.

## Configuration validation fails

Run the non-mutating check first:

```bash
taytay --check-config /etc/taytay/taytay.toml
```

Check the following fields when validation reports a configuration error:

- `spool_dir`, `lunsaran_base_url`, `organization_id`, `project_id`, and
  `token_file` are present and non-empty.
- `quota_bytes` and `upload_workers` are positive.
- `lunsaran_base_url` is an HTTP(S) origin without credentials, a query, or a
  fragment.
- `token_file` does not contain parent-directory traversal.
- The service account can read the token file and write the spool directory.

Do not paste the token, complete authorization URL, or customer metadata into
an issue or log.

## Readiness reports `ready: false`

Inspect the machine-readable status:

```bash
taytay --status /etc/taytay/taytay.toml
```

The status includes pending and completed job counts, spool usage, and the
configured quota. If spool usage is at or above the quota, increase the quota
only after confirming that the filesystem has capacity and that the retention
policy permits cleanup. Do not delete the spool to make the status ready.

If the counts are zero, the source application has not published a pending
artifact. Confirm that the source considers the file complete before calling
`Spool::publish`.

## Jobs remain pending

Run the worker against the same configuration:

```bash
taytay /etc/taytay/taytay.toml
```

An empty spool completes with a JSON report containing zero completed and
failed jobs. A pending job remains local when the worker is interrupted,
cancelled, or encounters a retryable transport failure. Re-run the worker
after checking network reachability and service logs.

## Authorization or expiry failures

An authorization failure means the credential was rejected for the requested
organization or project. An expired-session failure means the upload session
must be recreated. Verify that the credential is active, unexpired, and
enrolled for the configured project. Ask the Lunsaran operator to rotate or
re-enroll it when necessary.

Taytay keeps the artifact local when authorization fails. Do not replace the
artifact or edit its ledger entry while investigating.

## Resumed uploads fail after interruption

Taytay and Entregar use the server-confirmed TUS offset and the artifact
checksum to resume an upload. Keep the spool and ledger intact, then retry the
same job. Deleting the resume state or changing the artifact bytes breaks the
checksum-bound recovery contract.

If the failure persists, collect the redacted service output, the command
used, the status JSON, and the artifact ID. Do not collect tokens, signed URLs,
filenames that contain customer information, or media bytes.

## Paused jobs do not drain

Paused jobs are intentionally excluded from scheduling. Resume the specific
artifact, then run the worker again:

```bash
taytay --resume ARTIFACT_ID /etc/taytay/taytay.toml
taytay /etc/taytay/taytay.toml
```

Use `--pause ARTIFACT_ID` when you need to stop retry activity without deleting
the local artifact.

## Escalation checklist

Before escalating an issue, capture:

1. Taytay version and Linux version.
2. The redacted output of `--check-config` and `--status`.
3. The artifact ID and lifecycle state, but not its contents.
4. Whether the failure is reproducible with `scripts/acceptance-mock.sh`.
5. The relevant Lunsaran request status without credentials or signed URLs.
