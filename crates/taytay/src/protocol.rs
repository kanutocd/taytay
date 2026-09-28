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
