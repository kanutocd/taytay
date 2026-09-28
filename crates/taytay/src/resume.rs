use crate::{TaytayError, model::Artifact};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResumeState {
    pub artifact_id: String,
    pub path: String,
    pub size: u64,
    pub checksum_sha256: Option<String>,
    pub api_url: String,
    pub offset: u64,
}

impl ResumeState {
    pub fn new(artifact: &Artifact, api_url: impl Into<String>, offset: u64) -> Self {
        Self {
            artifact_id: artifact.id.0.clone(),
            path: artifact.path.clone(),
            size: artifact.size,
            checksum_sha256: artifact.checksum_sha256.clone(),
            api_url: api_url.into(),
            offset,
        }
    }
    pub fn validate_for(&self, artifact: &Artifact, api_url: &str) -> Result<(), TaytayError> {
        if self.artifact_id != artifact.id.0
            || self.path != artifact.path
            || self.size != artifact.size
            || self.checksum_sha256 != artifact.checksum_sha256
            || self.api_url != api_url
        {
            return Err(TaytayError::Protocol(
                "resume state does not match artifact identity".into(),
            ));
        }
        if self.offset > self.size {
            return Err(TaytayError::Protocol(
                "resume offset exceeds artifact size".into(),
            ));
        }
        Ok(())
    }
}

pub fn save(path: impl AsRef<Path>, state: &ResumeState) -> Result<(), TaytayError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, serde_json::to_vec(state)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(temp, path)?;
    Ok(())
}
pub fn load(path: impl AsRef<Path>) -> Result<ResumeState, TaytayError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
pub fn path_for(directory: impl AsRef<Path>, artifact_id: &str) -> PathBuf {
    directory
        .as_ref()
        .join(format!("{artifact_id}.resume.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArtifactId, SourceId};
    use std::time::{SystemTime, UNIX_EPOCH};
    #[test]
    fn state_is_atomic_and_identity_bound() {
        let root = std::env::temp_dir().join(format!(
            "taytay-resume-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let artifact = Artifact {
            id: ArtifactId::new("a"),
            source: SourceId::new("s"),
            path: "/tmp/a".into(),
            media_type: "x".into(),
            captured_at: None,
            metadata: serde_json::json!({}),
            size: 10,
            checksum_sha256: Some("sum".into()),
        };
        let path = path_for(&root, "a");
        let state = ResumeState::new(&artifact, "https://api", 4);
        save(&path, &state).unwrap();
        assert_eq!(load(&path).unwrap(), state);
        assert!(state.validate_for(&artifact, "https://other").is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
