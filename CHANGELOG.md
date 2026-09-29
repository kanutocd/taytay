# Changelog

## Unreleased

### Public integrator boundary

- Added a scalable SVG Taytay logo derived from the existing PNG branding asset
  and configured it as the Zensical site logo.
- Added a Zensical documentation site focused on crate consumers, including
  quick start, architecture, durability, uploader, source-adapter, operations,
  security, and development guides with Mermaid diagrams, plus a CI build
  check for documentation changes.
- Added documentation quality checks for Zensical builds, Markdown structure,
  fenced code blocks, trailing whitespace, and local-link drift; the checks
  now run as part of the complete quality harness and documentation CI.
- Added GitHub Pages deployment for Zensical documentation builds from `main`.
- Moved repository-specific engineering instructions into the private
  `taytay-saas` harness; the public repository no longer carries `AGENTS.md`.
- Added public crate metadata and README linkage so `taytay` packages with
  discoverable release information.
- Refined Taytay as a first-party Lunsaran integrator built on the public
  `lunsaran-entregar` client library.
- Documented the Apache-2.0 public-core direction, OEM/third-party adoption
  path, scoped device credentials, and private SaaS extension boundary.
- Documented the public Taytay workspace and separate private SaaS boundary;
  the future SaaS product will consume published Taytay crates.
- Moved the private architecture, implementation, API-contract, quality, and
  `lunsaran-entregar` integration harness into the companion SaaS repository;
  public operational, security, compatibility, and release documentation stays
  with the public workspace.

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
- Added upload-session scope validation, TUS offset conformance checks, and per-chunk SHA-256 propagation.
- Added protected device-credential file loading with strict Unix permissions, empty-secret rejection, and redacted secret debug behavior.
- Added deterministic motion-event deduplication and RTSP stream-loss coverage for event-aware and network-camera state boundaries.
- Added persisted failure transitions, retention policy primitives, health snapshots, and atomic operational counters for queue and source observability.
- Added bounded V4L2/RTSP segment buffers, ONVIF profile selection, and validated drone/LiDAR sidecar metadata models.
- Added secret-free `--check-config` and `--status` CLI diagnostics for operator configuration and spool readiness.
- Documented the current implementation boundary in `IMPLEMENTATION.md` and `README.md`, including completed software-only work and external/hardware blockers.
- Implemented the secret-free CLI diagnostics described by the implementation boundary.
- Pinned and integrated `lunsaran-entregar` 0.1.0 behind the async `ArtifactUploader` port, including typed error classification, per-job resume paths, progress propagation, and a deterministic `lunsaran-entregar-mock` transfer test.
- Added public-client conformance tests for transient PATCH failures, stale offsets, and pre-flight cancellation.
- Added deterministic event-to-capture windows, ONVIF event parsing, V4L2 format negotiation, RTSP timestamp/codec validation, and field sidecar metadata association.
- Added explicit device-credential enrollment, activation, rotation, revocation, expiry, and authorization gating for new upload sessions.
- Added Prometheus-compatible redacted metrics output, CI coverage artifact retention, and a standalone mock acceptance command.
- Documented operator status, metrics, and deterministic mock-acceptance procedures.
