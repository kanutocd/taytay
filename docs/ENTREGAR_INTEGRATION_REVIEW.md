# Entregar integration review for Taytay

Reviewed against the current workspace crates:

- `crates/entregar` — publishable Rust library, version `0.1.0`.
- `apps/entregar-cli` — human-facing binary, version `0.1.0`, installs `entregar`.
- `docs/ENTREGAR_CLIENT_CONFORMANCE.md` — shared wire and behavior contract.

## Integration decision

Taytay should be a standalone Rust application that depends on the **`entregar` library**. It should not depend on or invoke `entregar-cli`.

```text
Taytay source adapters
    → Taytay durable spool and job ledger
    → entregar::Client::upload_file_with_progress
    → Lunsaran POST /v1/upload-sessions
    → Brutus TUS POST/HEAD/PATCH
```

`entregar` is responsible for one file transfer. Taytay remains responsible for discovering artifacts, durable scheduling, source health, device credentials, organization/project configuration, and process recovery.

## Public library surface Taytay can consume

- `entregar::Config` — API URL, bearer credential, timeout, chunk size, and retry limit.
- `entregar::Client` — control-plane handshake and TUS transfer.
- `entregar::UploadOptions` — explicit project, content type, idempotency key, and resume-state path.
- `entregar::UploadProgress` — byte progress callback.
- `entregar::UploadResult` — session/asset identity and transferred byte count.
- `entregar::Error` — configuration, file, HTTP/API, TUS, response, and resume failures.

The Taytay integration should use an explicit `project_id` and a stable job-derived `idempotency_key`. It should not use `Client::default_project_id()` in a daemon because a project named `Default` is ambiguous across organizations and can change independently of device configuration.

## Taytay wrapper boundary

Taytay should have a small upload port so source and queue code do not depend on `entregar` types everywhere:

```rust
#[async_trait]
pub trait ArtifactUploader {
    async fn upload(
        &self,
        artifact: &Artifact,
        job: &UploadJob,
        progress: Box<dyn Fn(UploadProgress) + Send>,
    ) -> Result<UploadReceipt, UploadError>;
}
```

The production implementation wraps `entregar::Client`. A fake implementation drives deterministic queue and recovery tests without HTTP. The wrapper should:

- map Taytay's job UUID to the idempotency key;
- place each job's resume record in the Taytay-managed state directory;
- pass project and content type explicitly;
- convert `UploadResult` into a Taytay receipt;
- classify retryable, expired, unauthorized, permanent, and cancellation outcomes;
- avoid logging `Error` values if a future error variant can contain remote data.

## Credential and state handling

Taytay loads a Lunsaran-issued device/workload credential through its own protected configuration or secret-manager integration and passes it to `Config`. A human user token must not be used as the device identity. The credential must not be placed in command arguments, job records, source metadata, logs, metrics, or uploaded files. Credential scope, expiry, rotation, revocation, and audit remain Lunsaran responsibilities.

`entregar`'s resume state is an implementation detail of the wrapped upload attempt. Taytay's ledger remains the source of truth for job lifecycle, while the library state allows TUS offset recovery. A job must not be deleted until the library reports completion and the ledger transaction is durable.

## Current strengths

- Explicit Lunsaran session creation with `Idempotency-Key`.
- TUS `POST`, `HEAD`, and `PATCH` flow.
- Server-offset revalidation after TUS/API errors.
- Bounded chunk memory.
- Resumable state excludes the bearer/session token.
- TUS URL validation rejects userinfo, query, and fragment components.
- Library is independent of database, provider, and API handler crates.

## Gaps to resolve before Taytay production use

These are review findings, not assumptions that the existing library already satisfies them:

1. **Request retry classification:** transport-level `reqwest` failures currently are not retried by the upload loop, although transient network failures are expected on edge devices.
2. **TUS response validation:** creation, `HEAD`, and `PATCH` responses should validate the TUS version and ensure returned offsets are monotonic and do not exceed `Upload-Length`.
3. **Atomic resume-state writes:** state updates should use a temporary file plus atomic rename and restrictive permissions where supported.
4. **File identity binding:** resume state currently binds API URL, path, and size. Taytay should also bind a content identity such as a checksum or stable inode/mtime tuple before reusing a state record.
5. **Credential-bearing configuration:** API URLs should reject query/fragment/userinfo components before requests are sent; secret values should be represented by non-debuggable wrappers in future API revisions.
6. **Cancellation and pause:** the current library surface has no cancellation token or pause control. Taytay must stop scheduling new work and allow in-flight transfer cancellation through a planned library extension or task boundary.
7. **Metadata/checksum propagation:** `UploadOptions` exposes content type but the implementation currently sends no checksum. Taytay should not claim end-to-end checksum verification until the library and server contract support it.
8. **CLI/documentation drift:** the CLI currently exposes project, resume, JSON, quiet, and color options. Documented chunk, retry, metadata, pause, and cancellation flags are not yet all CLI options.
9. **Result naming:** the current API response's `id` is the upload/session identity. Keep Taytay's receipt naming explicit until the platform exposes a distinct asset identifier.
10. **Library test depth:** the fixture covers a successful transfer, but the conformance matrix still needs deterministic tests for malformed offsets, transient transport errors, expiry, cancellation, and token redaction.

## Recommended Taytay dependency policy

- Pin a reviewed `entregar` version rather than using a floating dependency.
- Keep the library behind Taytay's `ArtifactUploader` port.
- Do not fork or duplicate TUS logic in Taytay.
- Upgrade `entregar` only after the shared conformance fixtures pass.
- Treat `entregar-cli` as a separate human workflow, not a runtime dependency.

## Acceptance criteria for the Taytay integration

- A fake uploader can exercise every queue transition without network access.
- A real `entregar` adapter uploads a completed artifact through local Lunsaran and Brutus.
- Restarting Taytay resumes both the durable job and TUS offset without duplicate sessions.
- Offline operation preserves artifacts until a later successful upload.
- Project, organization, and idempotency context comes from Taytay configuration and the authenticated Lunsaran device credential, never from source metadata.
- When Lunsaran is unavailable, Taytay queues locally and does not make authorization decisions for new sessions.
- No source credential, API token, TUS session token, signed URL, or provider identity is emitted to logs or persisted in the Taytay job ledger.
