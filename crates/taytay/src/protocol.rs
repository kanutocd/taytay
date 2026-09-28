use crate::{TaytayError, model::Artifact};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct UploadSession {
    pub upload_url: String,
    pub expires_at: String,
    pub organization_id: String,
    pub project_id: String,
    pub chunk_size: u64,
    pub checksum_algorithm: Option<String>,
}
impl UploadSession {
    pub fn validate_for(&self, organization_id: &str, project_id: &str) -> Result<(), TaytayError> {
        if self.organization_id != organization_id
            || self.project_id != project_id
            || self.upload_url.is_empty()
            || self.chunk_size == 0
        {
            return Err(TaytayError::Protocol(
                "upload session is missing scope or transfer capabilities".into(),
            ));
        }
        Ok(())
    }
}
pub trait LunsaranClient: Send + Sync {
    fn create_upload_session(
        &self,
        artifact: &Artifact,
        organization_id: &str,
        project_id: &str,
        idempotency_key: &str,
    ) -> Result<UploadSession, TaytayError>;
    fn report_state(&self, artifact: &Artifact, state: &str) -> Result<(), TaytayError>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TusOffset {
    pub offset: u64,
    pub length: u64,
}
pub trait TusClient: Send + Sync {
    fn create(&self, session: &UploadSession, artifact: &Artifact) -> Result<String, TaytayError>;
    fn head(&self, upload_url: &str) -> Result<TusOffset, TaytayError>;
    fn patch(
        &self,
        upload_url: &str,
        offset: u64,
        chunk: &[u8],
        checksum_sha256: Option<&str>,
    ) -> Result<TusOffset, TaytayError>;
}

pub fn validate_offset(
    previous: u64,
    returned: &TusOffset,
    sent: u64,
    length: u64,
) -> Result<(), TaytayError> {
    let expected = previous.saturating_add(sent);
    if returned.offset != expected
        || returned.offset < previous
        || returned.offset > length
        || returned.length != length
    {
        return Err(TaytayError::Protocol(format!(
            "invalid TUS offset response: expected {expected}, got {}",
            returned.offset
        )));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedRequest {
    pub method: String,
    pub path: String,
    pub authorization: String,
    pub idempotency_key: Option<String>,
    pub body: serde_json::Value,
}
pub fn create_session_request(
    base_path: &str,
    token: &str,
    artifact: &Artifact,
    organization_id: &str,
    project_id: &str,
    idempotency_key: &str,
) -> AuthenticatedRequest {
    AuthenticatedRequest {
        method: "POST".into(),
        path: format!("{base_path}/v1/upload-sessions"),
        authorization: format!("Bearer {token}"),
        idempotency_key: Some(idempotency_key.into()),
        body: serde_json::json!({"organization_id": organization_id, "project_id": project_id, "artifact_id": artifact.id, "media_type": artifact.media_type, "size": artifact.size, "checksum_sha256": artifact.checksum_sha256}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_is_scoped_and_authenticated() {
        let a = Artifact {
            id: crate::ArtifactId::new("a"),
            source: crate::SourceId::new("s"),
            path: "local".into(),
            media_type: "video/mp4".into(),
            captured_at: None,
            metadata: serde_json::json!({}),
            size: 4,
            checksum_sha256: None,
        };
        let r = create_session_request("https://lunsaran", "secret", &a, "org", "project", "a");
        assert_eq!(r.authorization, "Bearer secret");
        assert_eq!(r.body["organization_id"], "org");
        assert_eq!(r.body["project_id"], "project");
    }
    #[test]
    fn rejects_non_monotonic_offset() {
        let result = validate_offset(
            4,
            &TusOffset {
                offset: 3,
                length: 7,
            },
            2,
            7,
        );
        assert!(result.is_err());
    }
}
