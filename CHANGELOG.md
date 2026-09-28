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
