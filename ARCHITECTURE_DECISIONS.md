# Architecture decisions

## ADR-001: Taytay is source-independent

**Status:** Accepted

The product name and core interfaces describe an edge bridge, not a CCTV product. CCTV is the first adapter because it is available for testing; future sources must not require rewriting the queue or uploader.

## ADR-002: Completed artifacts are the first ingestion boundary

**Status:** Accepted

The first production slice consumes completed files or segments. Direct live capture is deferred until durable queueing and resumable upload behavior are proven. This reduces codec and stream-liveness complexity while preserving a path to RTSP and V4L2.

## ADR-003: TUS is the upload transport

**Status:** Accepted

Taytay uses Lunsaran's control-plane API to create scoped sessions and Brutus's TUS endpoint to transfer bytes. Taytay does not implement provider uploads or persist provider credentials.

## ADR-004: Durable local state is mandatory

**Status:** Accepted

A process-memory queue cannot guarantee delivery across network loss or restart. The spool and job ledger therefore persist artifacts, checksums, offsets, and terminal state locally.

## ADR-005: Source credentials stay at the edge

**Status:** Accepted

Camera, NVR, SMB, and vendor credentials are device-local secrets. They are loaded from protected configuration or a secret manager, redacted from logs, and never included in Lunsaran requests.

## ADR-006: No public Taytay API is required initially

**Status:** Accepted

The initial control surface is a local configuration and service interface plus outbound Lunsaran calls. A local REST API may be added later for status and administration, but it must not bypass Lunsaran authorization or expose source secrets.
