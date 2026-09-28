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

## ADR-007: Consume entregar as a library, not as a subprocess

**Status:** Accepted

Taytay embeds the published `entregar` library for Lunsaran session creation and TUS transfer. It does not invoke the human-oriented `entregar` binary. Taytay needs typed progress, error classification, cancellation, and durable job integration; invoking a CLI would duplicate process and credential handling.

The dependency is isolated behind Taytay's `ArtifactUploader` port so queue and source adapters remain testable with a fake uploader.

## ADR-008: Taytay owns queue state; entregar owns transfer state

**Status:** Accepted

Taytay's durable ledger records artifact and job lifecycle. Entregar's resume record is retained per job only to recover TUS offset state. Completion cleanup occurs after both layers have durably acknowledged success.

## ADR-009: Lunsaran owns Taytay identity and authorization

**Status:** Accepted

Taytay depends on Lunsaran for authentication, organization/project authorization, device enrollment, credential rotation, revocation, upload-session policy, and audit. Taytay does not embed a second user, membership, or RBAC system.

Taytay uses a dedicated scoped device/workload credential rather than a human user's long-lived API token. The credential is loaded locally and is never logged, placed in job state, or sent to a source device.

When the platform is unavailable, Taytay buffers artifacts and may resume still-valid TUS sessions. It cannot authorize new uploads locally; it waits until Lunsaran is reachable. Any local administration authentication is limited to the device and does not reproduce platform identity or policy.
