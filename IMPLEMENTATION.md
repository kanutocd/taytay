# Taytay implementation plan

Statuses: **Planned**, **In progress**, **Implemented**, **Deferred**.

Dates below are target release dates for planning and sequencing, not commitments. Each milestone is cumulative: later releases retain the earlier adapters and guarantees.

## Milestone schedule

| Release | Target date | Scope | Included source adapters | Release deliverable |
| --- | --- | --- | --- | --- |
| `0.0.1` Harness preview | 2026-10-09 | Workspace and contracts | None | Compilable Rust workspace, configuration model, adapter/uploader traits, Lunsaran/TUS fixtures, security and operations documentation |
| `0.1.0` Local edge preview | 2026-11-06 | Durable local ingestion | Filesystem/NVR export | Working spool and ledger, atomic artifact publication, checksums, restart recovery, disk limits, CLI diagnostics, and file-source integration tests |
| `0.2.0` Device preview | 2026-12-04 | Direct local capture | Filesystem/NVR export, USB cameras through V4L2 | V4L2 capture adapter, configurable segmenting, device reconnect handling, local preview capture, and end-to-end upload of USB-camera segments |
| `0.3.0` Upload preview | 2027-01-15 | Production-shaped transfer | Filesystem/NVR export, V4L2 USB cameras | Authenticated Lunsaran client, scoped upload sessions, resumable TUS transfer, retry/backoff, offline queueing, and Brutus/MinIO acceptance tests |
| `0.4.0` Network-camera preview | 2027-02-26 | Network CCTV integration | Filesystem/NVR export, V4L2, RTSP | Supervised RTSP segment adapter, codec and timestamp metadata, stream reconnect behavior, and tested camera compatibility matrix |
| `0.5.0` CCTV interoperability preview | 2027-04-09 | Camera discovery and recordings | Filesystem/NVR export, V4L2, RTSP, ONVIF Profile T/G | ONVIF discovery, Profile T stream selection, Profile G recording retrieval where supported, device credential handling, and device acceptance suite |
| `0.6.0` Event-aware preview | 2027-05-21 | Event-driven capture | All previous adapters, optional ONVIF Profile M | Motion/analytics event ingestion, event-to-capture policy, deduplication, event metadata propagation, and optional MQTT event integration |
| `0.7.0` Beta candidate | 2027-07-02 | Operational hardening | CCTV/NVR export, V4L2, RTSP, ONVIF, optional Profile M | systemd packaging, upgrades and rollback, metrics and redacted logs, long-duration tests, interruption injection, disk-pressure behavior, and release documentation |
| `0.8.0` Field-source preview | 2027-08-13 | Drone and LiDAR ingestion | Drone media/file adapter, MAVLink telemetry sidecar, LiDAR LAS/LAZ artifact adapter | Field-source configuration, flight and sensor metadata association, large point-cloud queueing, checksum validation, and end-to-end resumable upload of drone and LiDAR artifacts |

A `1.0.0` date is intentionally not set until the beta candidate has completed field validation across the supported camera, NVR, Linux, and network combinations.

## Phase 1 — Workspace and contracts (Planned)

**Deliverables:** the `0.0.1` harness preview.

- Create the Rust workspace and Linux service binary.
- Define `Artifact`, `SourceId`, `UploadJob`, and lifecycle state types.
- Define adapter and uploader traits with explicit error types.
- Document supported Linux versions, configuration, and secret sources.
- Add contract fixtures for Lunsaran upload-session responses and TUS behavior.
- Add the quality and coverage scripts, report formats, and staged coverage thresholds.

## Phase 2 — Durable spool and ledger (Planned)

**Deliverables:** the durable queue used by every later adapter and the `0.1.0` local edge preview.

- Implement atomic artifact publication and disk quotas.
- Persist jobs, checksums, attempts, server URLs, offsets, and terminal state.
- Recover pending jobs after restart and prevent duplicate publication.
- Add bounded queueing, backpressure, retention, and cleanup rules.
- Test power-loss-like interruption and database/filesystem recovery.

## Phase 3 — Lunsaran and TUS upload path (Planned)

**Deliverables:** the `0.3.0` upload preview, including a working transfer path against local Lunsaran, Brutus, and MinIO/S3 fixtures.

- Implement authenticated Lunsaran control-plane client.
- Create scoped upload sessions for organization and project context.
- Implement TUS creation, `HEAD` offset recovery, `PATCH` chunk transfer, retries, and resume.
- Verify no provider credentials or signed session contents enter logs.
- Add end-to-end tests against local Lunsaran, Brutus, and MinIO/S3 fixtures.

## Phase 4 — File and local-camera source adapters (Planned)

**Deliverables:** the `0.1.0` filesystem/NVR adapter and the `0.2.0` USB-camera adapter.

### Filesystem/NVR adapter

- Consume completed files from an NVR export directory or mounted SMB/NFS share.
- Detect completion safely using atomic publication, stability checks, or producer markers.
- Extract timestamps and media metadata and compute checksums.
- Recover from share disconnects without losing queued artifacts.

### USB/V4L2 adapter

- Capture from Linux V4L2 devices, including USB webcams and capture cards.
- Negotiate supported pixel formats and resolutions.
- Segment recordings into bounded files suitable for resumable upload.
- Handle device disconnect/reconnect and preserve capture metadata.
- Keep capture and encoding policy separate from the common spool and uploader.

## Phase 5 — Network-camera protocols (Planned)

**Deliverables:** the `0.4.0` RTSP network-camera preview and `0.5.0` ONVIF interoperability preview.

- Add RTSP segment capture through a supervised GStreamer/FFmpeg integration.
- Add reconnect, timeout, clock-drift, and stream-health handling.
- Add ONVIF discovery and Profile T stream selection.
- Add Profile G recording retrieval where devices support it.
- Add a tested compatibility matrix for camera/NVR models and codecs.
- Keep all stream and camera credentials local to Taytay.

## Phase 6 — Event-aware ingestion (Planned)

**Deliverables:** the `0.6.0` event-aware preview.

- Add optional ONVIF Profile M event and metadata ingestion.
- Map motion or analytics events to configurable capture policies.
- Deduplicate repeated events and associate events with artifacts.
- Support MQTT only for lightweight event/telemetry delivery; never use it for video payloads.

## Phase 7 — Operations and security (Planned)

**Deliverables:** the `0.7.0` beta candidate operational package.

- Add systemd unit, least-privilege service account, protected directories, and rotation guidance.
- Add health/readiness endpoints or local diagnostics without exposing secrets.
- Publish CI quality and coverage artifacts with redaction checks.
- Add metrics, structured redacted logs, tracing, and disk/network alerts.
- Add upgrade, rollback, configuration validation, and offline operation guides.
- Run long-duration capture and upload tests.
- Inject network interruption, process restart, disk pressure, and source disconnects.
- Verify no loss, duplicate uploads, credential leakage, or cross-organization routing.
- Publish compatibility results for tested cameras/NVRs and supported codecs.

## Phase 8 — Drone and LiDAR source adapters (Deferred)

**Deliverables:** the `0.8.0` field-source preview. These adapters are intentionally scheduled after the CCTV/NVR, USB-camera, RTSP, and ONVIF paths have demonstrated durable ingestion.

### Drone adapter

- Ingest completed photos, video, and mission exports from onboard storage or a mounted field directory.
- Associate flight identifiers, timestamps, position, altitude, and sensor metadata with each artifact.
- Support MAVLink telemetry as a metadata sidecar or event source; MAVLink is a lightweight drone communication protocol for telemetry and onboard components. [MAVLink developer guide](https://mavlink.io/en/)
- Keep flight-control and safety-critical commands outside Taytay's upload responsibility.

### LiDAR adapter

- Ingest completed LAS/LAZ point-cloud files and related trajectory or calibration files.
- Preserve coordinate reference system, sensor identity, capture time, point count, and processing provenance as metadata.
- Stream checksums and large-file reads without loading a point cloud into memory.
- Add optional ROS 2/DDS or vendor-SDK integration only after file-based ingestion is reliable.
- Treat LAS/LAZ as artifact formats, not as a replacement for Taytay's upload protocol. LAS is maintained by the ASPRS LAS working group and LAZ provides lossless compression for LAS point clouds. [ASPRS LAS](https://github.com/ASPRSorg/LAS), [OGC LAZ](https://www.ogc.org/announcement/ogc-announces-publication-of-the-laz-1-4-community-standard/)

## Deferred

- UI and browser administration.
- Cloud-provider credentials in Taytay.
- A broad plugin marketplace.
- Bare-metal or microcontroller targets; Taytay initially targets Linux systems.
