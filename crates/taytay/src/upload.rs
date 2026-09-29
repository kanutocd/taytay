use crate::{
    TaytayError,
    model::{ArtifactState, UploadJob},
    protocol::{LunsaranClient, TusClient},
    spool::Spool,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::pin::Pin;
use std::{
    fs::File,
    future::Future,
    io::{Read, Seek, SeekFrom},
};
use uuid::Uuid;

pub type UploadFuture<'a> =
    Pin<Box<dyn Future<Output = Result<UploadReceipt, UploadError>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadProgress {
    pub transferred: u64,
    pub total: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadReceipt {
    pub upload_id: String,
    pub transferred: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UploadError {
    Retryable(String),
    Unauthorized(String),
    Expired(String),
    Permanent(String),
    Cancelled,
}
impl UploadError {
    pub fn retryable(&self) -> bool {
        matches!(self, Self::Retryable(_) | Self::Expired(_))
    }
}
pub trait ArtifactUploader: Send + Sync {
    fn upload<'a>(
        &'a self,
        artifact: &crate::Artifact,
        job: &UploadJob,
        progress: Box<dyn FnMut(UploadProgress) + Send + 'a>,
    ) -> UploadFuture<'a>;
}

pub async fn upload_with_uploader<U: ArtifactUploader>(
    spool: &Spool,
    uploader: &U,
    mut job: UploadJob,
) -> Result<UploadJob, UploadError> {
    spool
        .verify(&job)
        .map_err(|e| UploadError::Permanent(e.to_string()))?;
    job.transition(ArtifactState::Uploading)
        .map_err(|e| UploadError::Permanent(e.to_string()))?;
    job.attempts += 1;
    let mut transferred = job.offset;
    let progress = |value: UploadProgress| {
        transferred = value.transferred;
    };
    let receipt = uploader
        .upload(&job.artifact, &job, Box::new(progress))
        .await?;
    job.server_url = Some(receipt.upload_id);
    job.offset = receipt.transferred.max(transferred);
    job.transition(ArtifactState::Completed)
        .map_err(|e| UploadError::Permanent(e.to_string()))?;
    spool
        .ledger()
        .update(job.clone())
        .map_err(|e| UploadError::Permanent(e.to_string()))?;
    Ok(job)
}

pub async fn upload_with_uploader_cancelled<U: ArtifactUploader>(
    spool: &Spool,
    uploader: &U,
    job: UploadJob,
    cancellation: &crate::scheduler::CancellationToken,
) -> Result<UploadJob, UploadError> {
    if cancellation.is_cancelled() {
        return Err(UploadError::Cancelled);
    }
    let result = upload_with_uploader(spool, uploader, job).await;
    if cancellation.is_cancelled() {
        Err(UploadError::Cancelled)
    } else {
        result
    }
}

pub struct EntregarUploader {
    client: lunsaran_entregar::Client,
    project_id: Uuid,
    resume_dir: PathBuf,
}
impl EntregarUploader {
    pub fn new(
        client: lunsaran_entregar::Client,
        project_id: Uuid,
        resume_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            client,
            project_id,
            resume_dir: resume_dir.into(),
        }
    }
    fn entregar_state_path(&self, artifact: &crate::Artifact) -> PathBuf {
        crate::resume::path_for(&self.resume_dir, &format!("{}.entregar", artifact.id.0))
    }
    fn identity_path(&self, artifact: &crate::Artifact) -> PathBuf {
        crate::resume::path_for(&self.resume_dir, &artifact.id.0)
    }
    fn prepare_identity(&self, artifact: &crate::Artifact) -> Result<(), UploadError> {
        let path = self.identity_path(artifact);
        if path.exists() {
            let state =
                crate::resume::load(&path).map_err(|e| UploadError::Permanent(e.to_string()))?;
            state
                .validate_for(artifact, "lunsaran-entregar")
                .map_err(|e| UploadError::Permanent(e.to_string()))?;
        } else {
            crate::resume::save(
                &path,
                &crate::resume::ResumeState::new(artifact, "lunsaran-entregar", 0),
            )
            .map_err(|e| UploadError::Permanent(e.to_string()))?;
        }
        Ok(())
    }
}
impl ArtifactUploader for EntregarUploader {
    fn upload<'a>(
        &'a self,
        artifact: &crate::Artifact,
        job: &UploadJob,
        mut progress: Box<dyn FnMut(UploadProgress) + Send + 'a>,
    ) -> UploadFuture<'a> {
        let artifact = artifact.clone();
        let job = job.clone();
        let state_path = self.entregar_state_path(&artifact);
        let identity = self.prepare_identity(&artifact);
        Box::pin(async move {
            identity?;
            let result = self
                .client
                .upload_file_with_progress(
                    &artifact.path,
                    lunsaran_entregar::UploadOptions {
                        project_id: self.project_id,
                        content_type: Some(artifact.media_type.clone()),
                        idempotency_key: Some(job.artifact.id.0.clone()),
                        resume_state: Some(state_path.clone()),
                    },
                    |value| {
                        progress(UploadProgress {
                            transferred: value.uploaded,
                            total: value.total,
                        })
                    },
                )
                .await
                .map_err(map_entregar_error)?;
            let _ = tokio::fs::remove_file(&state_path).await;
            let _ = tokio::fs::remove_file(self.identity_path(&artifact)).await;
            Ok(UploadReceipt {
                upload_id: result.asset_id.to_string(),
                transferred: result.bytes_uploaded,
            })
        })
    }
}

fn map_entregar_error(error: lunsaran_entregar::Error) -> UploadError {
    use lunsaran_entregar::Error;
    match error {
        Error::Api { status, .. } if status.as_u16() == 401 || status.as_u16() == 403 => {
            UploadError::Unauthorized("Lunsaran authorization failed".into())
        }
        Error::Api { status, .. }
            if status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error() =>
        {
            UploadError::Retryable("transient Lunsaran API failure".into())
        }
        Error::Api { message, .. } => UploadError::Permanent(message),
        Error::Request(_) | Error::Tus(_) => {
            UploadError::Retryable("transient upload transport failure".into())
        }
        Error::Resume(message) => UploadError::Permanent(format!("resume state: {message}")),
        Error::Configuration(message) | Error::Response(message) => UploadError::Permanent(message),
        Error::File(message) => UploadError::Permanent(message.to_string()),
    }
}

pub fn upload<C: LunsaranClient, T: TusClient>(
    spool: &Spool,
    client: &C,
    tus: &T,
    mut job: UploadJob,
    organization_id: &str,
    project_id: &str,
) -> Result<UploadJob, TaytayError> {
    spool.verify(&job)?;
    job.transition(ArtifactState::Uploading)?;
    job.attempts += 1;
    let session = client.create_upload_session(
        &job.artifact,
        organization_id,
        project_id,
        &job.artifact.id.0,
    )?;
    session.validate_for(organization_id, project_id)?;
    let upload_url = tus.create(&session, &job.artifact)?;
    let head = tus.head(&upload_url)?;
    if head.offset > job.artifact.size || head.length != job.artifact.size {
        return Err(TaytayError::Protocol(
            "TUS HEAD offset or length is invalid".into(),
        ));
    }
    let mut offset = head.offset;
    let mut file = File::open(&job.artifact.path)?;
    file.seek(SeekFrom::Start(offset))?;
    let chunk_size = session.chunk_size.max(1) as usize;
    let mut buf = vec![0; chunk_size];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let checksum = if session.checksum_algorithm.as_deref() == Some("sha256") {
            Some(hex(&Sha256::digest(&buf[..n])))
        } else {
            None
        };
        let result = tus.patch(&upload_url, offset, &buf[..n], checksum.as_deref())?;
        crate::protocol::validate_offset(offset, &result, n as u64, job.artifact.size)?;
        offset = result.offset;
    }
    job.server_url = Some(upload_url);
    job.offset = offset;
    job.transition(ArtifactState::Completed)?;
    client.report_state(&job.artifact, "completed")?;
    spool.ledger().update(job.clone())?;
    Ok(job)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{ArtifactId, SourceId},
        protocol::{TusOffset, UploadSession},
    };
    use std::{
        sync::{Arc, Mutex},
        time::{SystemTime, UNIX_EPOCH},
    };

    struct FakeControl;
    impl LunsaranClient for FakeControl {
        fn create_upload_session(
            &self,
            _: &crate::Artifact,
            _: &str,
            _: &str,
            _: &str,
        ) -> Result<UploadSession, TaytayError> {
            Ok(UploadSession {
                upload_url: "tus://fake".into(),
                expires_at: "never".into(),
                organization_id: "org".into(),
                project_id: "project".into(),
                chunk_size: 2,
                checksum_algorithm: Some("sha256".into()),
            })
        }
        fn report_state(&self, _: &crate::Artifact, _: &str) -> Result<(), TaytayError> {
            Ok(())
        }
    }
    struct FakeTus {
        bytes: Arc<Mutex<Vec<u8>>>,
    }
    struct FakeUploader;
    impl ArtifactUploader for FakeUploader {
        fn upload<'a>(
            &'a self,
            artifact: &crate::Artifact,
            _: &UploadJob,
            mut progress: Box<dyn FnMut(UploadProgress) + Send + 'a>,
        ) -> UploadFuture<'a> {
            let size = artifact.size;
            Box::pin(async move {
                progress(UploadProgress {
                    transferred: size,
                    total: size,
                });
                Ok(UploadReceipt {
                    upload_id: "upload-1".into(),
                    transferred: size,
                })
            })
        }
    }
    impl TusClient for FakeTus {
        fn create(&self, _: &UploadSession, _: &crate::Artifact) -> Result<String, TaytayError> {
            Ok("tus://fake/1".into())
        }
        fn head(&self, _: &str) -> Result<TusOffset, TaytayError> {
            Ok(TusOffset {
                offset: self.bytes.lock().unwrap().len() as u64,
                length: 7,
            })
        }
        fn patch(
            &self,
            _: &str,
            offset: u64,
            chunk: &[u8],
            _: Option<&str>,
        ) -> Result<TusOffset, TaytayError> {
            assert_eq!(offset, self.bytes.lock().unwrap().len() as u64);
            self.bytes.lock().unwrap().extend_from_slice(chunk);
            Ok(TusOffset {
                offset: offset + chunk.len() as u64,
                length: 7,
            })
        }
    }
    #[test]
    fn upload_resumes_from_server_offset() {
        let root = std::env::temp_dir().join(format!(
            "taytay-upload-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let spool = Spool::open(&root, 1024).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("one"),
                SourceId::new("test"),
                "x".into(),
                b"payload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        let bytes = Arc::new(Mutex::new(b"pa".to_vec()));
        let result = upload(
            &spool,
            &FakeControl,
            &FakeTus {
                bytes: bytes.clone(),
            },
            job,
            "org",
            "project",
        )
        .unwrap();
        assert_eq!(result.state, ArtifactState::Completed);
        assert_eq!(&*bytes.lock().unwrap(), b"payload");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn uploader_port_updates_durable_job() {
        let root = std::env::temp_dir().join(format!(
            "taytay-port-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let spool = Spool::open(&root, 1024).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("one"),
                SourceId::new("test"),
                "x".into(),
                b"payload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        let completed = upload_with_uploader(&spool, &FakeUploader, job)
            .await
            .unwrap();
        assert_eq!(completed.state, ArtifactState::Completed);
        assert_eq!(completed.server_url.as_deref(), Some("upload-1"));
        assert_eq!(spool.ledger().pending().len(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn entregar_uploader_transfers_through_public_mock() {
        let server = lunsaran_entregar_mock::MockServer::start().await.unwrap();
        let root = std::env::temp_dir().join(format!(
            "taytay-entregar-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let spool = Spool::open(&root, 1024).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("entregar-one"),
                SourceId::new("test"),
                "text/plain".into(),
                b"mock-upload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        let client = lunsaran_entregar::Client::new(lunsaran_entregar::ClientConfig::new(
            server.base_url(),
            "device-token",
        ))
        .unwrap();
        let uploader = EntregarUploader::new(client, Uuid::nil(), root.join("resume"));
        let completed = upload_with_uploader(&spool, &uploader, job).await.unwrap();
        assert_eq!(completed.state, ArtifactState::Completed);
        assert_eq!(completed.offset, 11);
        assert_eq!(server.offset().await, 11);
        server.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    async fn upload_with_mock_behavior(behavior: lunsaran_entregar_mock::MockBehavior) -> u64 {
        let server = lunsaran_entregar_mock::MockServer::start_with_behavior(behavior)
            .await
            .unwrap();
        let root = std::env::temp_dir().join(format!(
            "taytay-entregar-fault-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let spool = Spool::open(&root, 1024).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("fault"),
                SourceId::new("test"),
                "text/plain".into(),
                b"fault-upload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        let mut config = lunsaran_entregar::ClientConfig::new(server.base_url(), "device-token");
        config.retry_attempts = 1;
        let client = lunsaran_entregar::Client::new(config).unwrap();
        let uploader = EntregarUploader::new(client, Uuid::nil(), root.join("resume"));
        let completed = upload_with_uploader(&spool, &uploader, job).await.unwrap();
        let offset = server.offset().await;
        assert_eq!(completed.offset, 12);
        server.shutdown();
        std::fs::remove_dir_all(root).unwrap();
        offset
    }

    #[tokio::test]
    async fn entregar_uploader_recovers_from_transient_patch_failure() {
        assert_eq!(
            upload_with_mock_behavior(lunsaran_entregar_mock::MockBehavior {
                fail_first_patch: true,
                ..Default::default()
            })
            .await,
            12
        );
    }

    #[tokio::test]
    async fn entregar_uploader_recovers_from_stale_offset() {
        assert_eq!(
            upload_with_mock_behavior(lunsaran_entregar_mock::MockBehavior {
                return_stale_offset_once: true,
                ..Default::default()
            })
            .await,
            12
        );
    }

    #[tokio::test]
    async fn cancelled_upload_is_rejected_before_start() {
        let root = std::env::temp_dir().join(format!(
            "taytay-cancel-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let spool = Spool::open(&root, 1024).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("cancel"),
                SourceId::new("test"),
                "x".into(),
                b"x",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        let token = crate::scheduler::CancellationToken::default();
        token.cancel();
        assert_eq!(
            upload_with_uploader_cancelled(&spool, &FakeUploader, job, &token).await,
            Err(UploadError::Cancelled)
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
