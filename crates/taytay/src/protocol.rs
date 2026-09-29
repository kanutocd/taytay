//! Minimal contracts for Lunsaran upload sessions and the TUS data plane.

use crate::{TaytayError, model::Artifact};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
/// Lunsaran-issued capabilities for one artifact upload.
pub struct UploadSession {
    /// Scoped TUS endpoint; never log this value.
    pub upload_url: String,
    /// Server expiry timestamp.
    pub expires_at: String,
    /// Organization scope returned by Lunsaran.
    pub organization_id: String,
    /// Project scope returned by Lunsaran.
    pub project_id: String,
    /// Maximum preferred PATCH chunk size.
    pub chunk_size: u64,
    /// Checksum algorithm negotiated by the session.
    pub checksum_algorithm: Option<String>,
}
impl UploadSession {
    /// Verifies that a session belongs to the requested scope and is usable.
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
/// Control-plane client required by the synchronous upload contract.
pub trait LunsaranClient: Send + Sync {
    /// Creates a scoped upload session for an artifact.
    fn create_upload_session(
        &self,
        artifact: &Artifact,
        organization_id: &str,
        project_id: &str,
        idempotency_key: &str,
    ) -> Result<UploadSession, TaytayError>;
    /// Reports a lifecycle state without exposing local credentials.
    fn report_state(&self, artifact: &Artifact, state: &str) -> Result<(), TaytayError>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
/// Server-confirmed TUS offset and declared upload length.
pub struct TusOffset {
    /// Number of bytes committed by the server.
    pub offset: u64,
    /// Total artifact length declared by the server.
    pub length: u64,
}
/// Data-plane client for creating, inspecting, and patching TUS resources.
pub trait TusClient: Send + Sync {
    /// Creates a remote TUS resource and returns its opaque URL.
    fn create(&self, session: &UploadSession, artifact: &Artifact) -> Result<String, TaytayError>;
    /// Reads the authoritative remote offset.
    fn head(&self, upload_url: &str) -> Result<TusOffset, TaytayError>;
    /// Sends one bounded chunk and returns the new authoritative offset.
    fn patch(
        &self,
        upload_url: &str,
        offset: u64,
        chunk: &[u8],
        checksum_sha256: Option<&str>,
    ) -> Result<TusOffset, TaytayError>;
}

/// Validates that a TUS response advanced exactly by the submitted chunk.
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
/// Safe representation of a Lunsaran session request for testing or transport.
pub struct AuthenticatedRequest {
    /// HTTP method.
    pub method: String,
    /// Relative API path.
    pub path: String,
    /// Bearer authorization header value.
    pub authorization: String,
    /// Idempotency key for the artifact.
    pub idempotency_key: Option<String>,
    /// JSON request body.
    pub body: serde_json::Value,
}
/// Builds a request value for a Lunsaran session transport or test fixture.
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
