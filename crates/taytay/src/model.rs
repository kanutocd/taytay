use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
/// Stable identity for a source artifact.
pub struct ArtifactId(pub String);

impl ArtifactId {
    /// Creates an artifact identity from a caller-owned stable value.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Stable identity for the device or source that produced an artifact.
pub struct SourceId(pub String);

impl SourceId {
    /// Creates a source identity from a configured source name.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Immutable reference to a completed source artifact.
///
/// The path must refer to a locally durable file. `checksum_sha256`, when
/// present, binds the artifact identity to its bytes and is verified before
/// upload.
pub struct Artifact {
    /// Stable artifact identity used for deduplication and idempotency.
    pub id: ArtifactId,
    /// Source identity supplied by the adapter.
    pub source: SourceId,
    /// Local path of the published artifact.
    pub path: String,
    /// MIME type or registered media type of the artifact.
    pub media_type: String,
    /// Optional source capture timestamp in an agreed serialized format.
    pub captured_at: Option<String>,
    /// Source-specific metadata; never put credentials or bearer tokens here.
    pub metadata: serde_json::Value,
    /// Number of bytes expected at `path`.
    pub size: u64,
    /// Lowercase hexadecimal SHA-256 digest of the artifact bytes.
    pub checksum_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Durable lifecycle state of an artifact upload job.
pub enum ArtifactState {
    /// Published locally and eligible for scheduling.
    Published,
    /// Claimed by an uploader.
    Uploading,
    /// Explicitly withheld from scheduling without deleting local bytes.
    Paused,
    /// Confirmed by the remote upload service.
    Completed,
    /// Failed with a recorded diagnostic and eligible for policy-driven retry.
    Failed,
    /// Completed and retained according to local retention policy.
    Retained,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Durable state tying an artifact to its upload lifecycle and remote offset.
pub struct UploadJob {
    /// Artifact being uploaded.
    pub artifact: Artifact,
    /// Current durable lifecycle state.
    pub state: ArtifactState,
    /// Number of upload attempts started for this job.
    pub attempts: u32,
    /// Remote asset or upload URL, when known.
    pub server_url: Option<String>,
    /// Last server-confirmed byte offset.
    pub offset: u64,
    /// Redacted or operator-safe last failure description.
    pub last_error: Option<String>,
}

impl UploadJob {
    /// Creates a new job in the [`ArtifactState::Published`] state.
    pub fn new(artifact: Artifact) -> Self {
        Self {
            artifact,
            state: ArtifactState::Published,
            attempts: 0,
            server_url: None,
            offset: 0,
            last_error: None,
        }
    }

    /// Applies a permitted lifecycle transition.
    ///
    /// The mutation is in memory; persist the job through [`crate::ledger::Ledger`]
    /// after a successful transition.
    pub fn transition(&mut self, next: ArtifactState) -> Result<(), crate::TaytayError> {
        let valid = matches!(
            (&self.state, &next),
            (
                ArtifactState::Published,
                ArtifactState::Uploading | ArtifactState::Failed
            ) | (
                ArtifactState::Uploading,
                ArtifactState::Uploading | ArtifactState::Completed | ArtifactState::Failed
            ) | (
                ArtifactState::Published | ArtifactState::Failed,
                ArtifactState::Paused
            ) | (ArtifactState::Paused, ArtifactState::Published)
                | (
                    ArtifactState::Failed,
                    ArtifactState::Uploading | ArtifactState::Retained
                )
                | (ArtifactState::Completed, ArtifactState::Retained)
                | (ArtifactState::Retained, ArtifactState::Retained)
        );
        if valid {
            self.state = next;
            Ok(())
        } else {
            Err(crate::TaytayError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", next),
            })
        }
    }

    /// Marks a job failed and records an operator-visible error description.
    pub fn fail(&mut self, error: impl Into<String>) -> Result<(), crate::TaytayError> {
        self.transition(ArtifactState::Failed)?;
        self.last_error = Some(error.into());
        Ok(())
    }
}
