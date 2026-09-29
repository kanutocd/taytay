//! Durable source-to-Lunsaran edge bridge primitives.
//!
//! Taytay is the reusable Linux-side portion of a Lunsaran integration. It
//! turns completed source artifacts into durable, checksummed, resumable jobs.
//! The crate deliberately stops at the edge boundary: it does not implement
//! user authorization, provider credentials, or a cloud storage adapter.
//!
//! A typical integration composes [`adapter::SourceAdapter`], [`spool::Spool`],
//! and [`upload::ArtifactUploader`]. The source produces an [`Artifact`], the
//! spool publishes it atomically, and an uploader transfers it while the
//! ledger preserves recovery state across process restarts.
//!
//! ```
//! use lunsaran_taytay::{Artifact, ArtifactId, SourceId, UploadJob};
//!
//! let artifact = Artifact {
//!     id: ArtifactId::new("camera-clip-001"),
//!     source: SourceId::new("front-door"),
//!     path: "/var/lib/taytay/spool/camera-clip-001.artifact".into(),
//!     media_type: "video/mp4".into(),
//!     captured_at: None,
//!     metadata: serde_json::json!({"camera": "front-door"}),
//!     size: 0,
//!     checksum_sha256: None,
//! };
//! let job = UploadJob::new(artifact);
//! assert_eq!(job.attempts, 0);
//! ```

#![warn(missing_docs)]

/// Source-adapter traits and protocol-specific adapter building blocks.
pub mod adapter;
/// Configuration loading and protected secret-file handling.
pub mod config;
/// Device/workload credential lifecycle primitives.
pub mod credential;
/// Redacted diagnostic formatting helpers.
pub mod diagnostics;
/// Errors returned by Taytay's durable edge operations.
pub mod error;
/// Event deduplication and event-to-capture policy.
pub mod events;
/// Atomic durable job ledger.
pub mod ledger;
/// Redacted Prometheus-compatible metrics formatting.
pub mod metrics;
/// Artifact and upload-job domain models.
pub mod model;
/// Retention, health, counters, and failure persistence.
pub mod operations;
/// Lunsaran session and TUS protocol contracts.
pub mod protocol;
/// Atomic TUS resume-state persistence.
pub mod resume;
/// Bounded retry and backpressure primitives.
pub mod retry;
/// Cancellation-aware bounded job scheduling.
pub mod scheduler;
/// Atomic artifact publication and checksum verification.
pub mod spool;
/// Upload ports and the published `lunsaran-entregar` integration.
pub mod upload;

pub use error::TaytayError;
pub use model::{Artifact, ArtifactId, ArtifactState, SourceId, UploadJob};
