use std::io;

#[derive(Debug, thiserror::Error)]
/// Errors produced by Taytay's configuration, durability, and protocol layers.
pub enum TaytayError {
    /// Invalid configuration or an unsupported operational value.
    #[error("configuration: {0}")]
    Configuration(String),
    #[error("artifact {0} already exists")]
    /// A stable artifact identity already exists in the ledger or spool.
    DuplicateArtifact(String),
    #[error("artifact is not ready: {0}")]
    /// A requested job is absent or not in a usable state.
    ArtifactNotReady(String),
    #[error("invalid state transition from {from} to {to}")]
    /// A lifecycle transition violates the durable state machine.
    InvalidTransition {
        /// Previous lifecycle state.
        from: String,
        /// Requested lifecycle state.
        to: String,
    },
    #[error("disk quota exceeded: requested {requested} bytes, available {available} bytes")]
    /// The artifact would exceed the configured local spool quota.
    QuotaExceeded {
        /// Bytes requested by the publication.
        requested: u64,
        /// Bytes remaining under quota.
        available: u64,
    },
    #[error("protocol: {0}")]
    /// A Lunsaran, TUS, source, or metadata protocol invariant failed.
    Protocol(String),
    #[error("invalid URL: {0}")]
    /// A URL failed the credential-safe URL validation rules.
    InvalidUrl(String),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    /// The bytes on disk differ from the recorded artifact digest.
    ChecksumMismatch {
        /// Digest recorded with the artifact.
        expected: String,
        /// Digest computed from the current file.
        actual: String,
    },
    #[error(transparent)]
    /// Filesystem I/O failure.
    Io(#[from] io::Error),
    #[error(transparent)]
    /// JSON serialization or deserialization failure.
    Json(#[from] serde_json::Error),
}
