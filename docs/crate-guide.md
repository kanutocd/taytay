---
title: Rust crate guide
description: Taytay's public traits and data contracts
---

# Rust crate guide

The public API is deliberately small. Integrators normally compose four
pieces: an adapter, a spool, a scheduler, and an uploader.

## Source adapters

Implement `SourceAdapter` when your source can be polled for completed
artifacts:

```rust
pub trait SourceAdapter: Send {
    fn source_id(&self) -> &SourceId;
    fn poll(&mut self) -> Result<Vec<Artifact>, TaytayError>;
}
```

An adapter should return only stable, complete artifacts. It should not upload,
write the ledger, or decide organization/project authorization. Use
`ArtifactReader` when reading needs a separate source-specific implementation.

## Artifact and job identity

`ArtifactId` must remain stable across process restarts and retries. The
artifact carries its source, local path, media type, capture time, metadata,
size, and optional SHA-256 checksum. `UploadJob` adds lifecycle state, attempts,
server identity, committed offset, and the last failure.

The normal state flow is:

```text
Published -> Uploading -> Completed -> Retained
         \-> Failed -> Uploading
```

`Retained` is the cleanup boundary, not an instruction to delete immediately.

## Uploaders

Implement `ArtifactUploader` for a custom transport. The callback reports
progress while the returned future performs the transfer:

```rust
pub trait ArtifactUploader: Send + Sync {
    fn upload<'a>(
        &'a self,
        artifact: &Artifact,
        job: &UploadJob,
        progress: Box<dyn FnMut(UploadProgress) + Send + 'a>,
    ) -> UploadFuture<'a>;
}
```

Return `UploadError::Retryable` for transient transport or service failures,
`Unauthorized` when new-session authorization is rejected, `Expired` when a
credential or session can be refreshed, `Permanent` for invalid input or
protocol failures, and `Cancelled` when work should stop.

The `EntregarUploader` implementation is the supported public-client path for
Lunsaran. It supplies a stable job-derived idempotency key and keeps resume
state under the configured edge spool.

## Composition rule

Keep policy above the traits. A source adapter should not know TUS offsets. An
uploader should not know whether bytes came from ONVIF, RTSP, or a mounted NVR
share. This separation is what lets one durability and recovery test suite
cover many source types.
