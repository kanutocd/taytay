# Taytay architecture

## Purpose

Taytay is an edge bridge between arbitrary local or networked sources and Lunsaran. It is not a storage provider, workflow engine, CCTV recorder, or replacement for Brutus.

```text
source adapter
    │ completed Artifact
    ▼
local spool and durable job ledger
    │ retryable UploadJob
    ▼
Lunsaran control-plane client
    │ scoped upload session
    ▼
Brutus TUS endpoint
    │ resumable bytes
    ▼
Lunsaran-selected storage and workflows
```

## Runtime components

### Source adapters

Adapters acquire completed artifacts and source metadata. The first adapters support CCTV/NVR exports and Linux USB cameras. Planned adapters include RTSP segment capture, ONVIF recording retrieval/events, drone media and MAVLink metadata, LiDAR LAS/LAZ artifacts, V4L2 devices, filesystem shares, and vendor APIs.

The adapter contract must deliver an immutable artifact reference, media type, capture timestamps, source identity, and optional metadata. It must not know how uploads are routed or stored.

### Spool and job ledger

The spool stores artifacts until upload acknowledgement. The ledger records discovery, checksum, upload-session identity, offset progress, attempts, terminal status, and cleanup eligibility. Publication uses a temporary filename followed by an atomic rename so incomplete files are never uploaded.

The first implementation may use SQLite for durable local state and a configurable filesystem spool. The state layer must be replaceable without changing adapters or the uploader.

### Upload scheduler

The scheduler applies bounded concurrency, retry backoff, cancellation, disk limits, and network-aware operation. It requests scoped upload sessions from Lunsaran and uploads bytes to Brutus using TUS. It resumes from the server-reported offset after interruption.

### Control-plane client

The client authenticates to Lunsaran, selects organization/project context, creates upload sessions, reports lifecycle state, and receives only scoped session information. It never calls object storage directly.

### Observability

Expose health, queue depth, upload latency, retry counts, bytes transferred, and source errors. Redact credentials, tokens, signed URLs, filenames when sensitive, and media contents.

## Source protocol strategy

- Filesystem/SMB/NFS adapters for NVR-exported files.
- V4L2 for Linux-local USB cameras and capture devices.
- ONVIF Profile T/G/M for compatible camera/NVR discovery, streams, recordings, and events.
- RTSP/RTP for live media where direct capture is required.
- MAVLink for drone telemetry and mission metadata.
- LAS/LAZ (and later point-cloud ecosystem formats) for LiDAR artifacts.
- Vendor adapters only where standards are insufficient.

Source-specific behavior belongs in adapters; the spool and TUS pipeline remains common.

## Failure model

The edge must survive process restart, power loss, unavailable cameras, unavailable NVR shares, intermittent connectivity, expired sessions, partial TUS transfers, disk pressure, and duplicate source notifications. Every transition is persisted before destructive cleanup.
