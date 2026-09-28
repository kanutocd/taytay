use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ArtifactId(pub String);

impl ArtifactId {
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
pub struct SourceId(pub String);

impl SourceId {
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
pub struct Artifact {
    pub id: ArtifactId,
    pub source: SourceId,
    pub path: String,
    pub media_type: String,
    pub captured_at: Option<String>,
    pub metadata: serde_json::Value,
    pub size: u64,
    pub checksum_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ArtifactState {
    Published,
    Uploading,
    Completed,
    Failed,
    Retained,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UploadJob {
    pub artifact: Artifact,
    pub state: ArtifactState,
    pub attempts: u32,
    pub server_url: Option<String>,
    pub offset: u64,
    pub last_error: Option<String>,
}

impl UploadJob {
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
                ArtifactState::Failed,
                ArtifactState::Uploading | ArtifactState::Retained
            ) | (ArtifactState::Completed, ArtifactState::Retained)
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
}
