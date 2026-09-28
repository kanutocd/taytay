use std::io;

#[derive(Debug, thiserror::Error)]
pub enum TaytayError {
    #[error("configuration: {0}")]
    Configuration(String),
    #[error("artifact {0} already exists")]
    DuplicateArtifact(String),
    #[error("artifact is not ready: {0}")]
    ArtifactNotReady(String),
    #[error("invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("disk quota exceeded: requested {requested} bytes, available {available} bytes")]
    QuotaExceeded { requested: u64, available: u64 },
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
