# Changelog

## Unreleased

- Established the Taytay project harness: architecture, ADRs, implementation phases, agent instructions, and API contract boundaries.
- Added target release milestones with explicit deliverables and adapter sequencing.
- Added the USB/V4L2 camera adapter as the first direct-device source after filesystem/NVR ingestion.
- Added a deferred field-source milestone for drone media/MAVLink metadata and LiDAR LAS/LAZ ingestion.
- Added the quality and coverage harness, reusable scripts, staged thresholds, test layers, and CI artifact requirements.
- Added the stable Rust workspace, validated configuration model, artifact/job lifecycle types, source and uploader traits, atomic spool publication, SHA-256 checksums, and restart-recoverable ledger.
- Added resumable TUS offset reconciliation, bounded queue/backoff primitives, redacted URL diagnostics, and deterministic control-plane/TUS upload tests.
- Added typed filesystem/NVR, V4L2 reconnect, RTSP stream-health, ONVIF discovery, and drone/LiDAR field-file adapter primitives with deterministic protocol coverage.
- Added systemd least-privilege packaging, CI quality/coverage workflow, synthetic Lunsaran/TUS fixtures, security and operations runbooks, and source compatibility guidance.
- Reviewed the entregar library and CLI boundary; Taytay now explicitly embeds the library behind an uploader port and tracks transfer-hardening gaps.
- Recorded the identity boundary: Lunsaran owns device authentication and organization/project authorization; Taytay uses scoped device credentials and local buffering only.
- Added Taytay's `ArtifactUploader` port, typed upload error classification, atomic checksum-bound resume records, and credential-bearing URL validation pending the external `entregar` library adapter.
- Added stable filesystem completion detection, cancellation-aware bounded scheduling, retry-delay access, completed-artifact cleanup, and related recovery tests.
- Added component coverage for stable NVR file observation and cleanup only after durable upload completion.
