use crate::{
    TaytayError,
    model::{ArtifactState, UploadJob},
    protocol::{LunsaranClient, TusClient},
    spool::Spool,
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

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
    fn upload(
        &self,
        artifact: &crate::Artifact,
        job: &UploadJob,
        progress: &mut dyn FnMut(UploadProgress),
    ) -> Result<UploadReceipt, UploadError>;
}

pub fn upload_with_uploader<U: ArtifactUploader>(
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
    let mut progress = |value: UploadProgress| {
        transferred = value.transferred;
    };
    let receipt = uploader.upload(&job.artifact, &job, &mut progress)?;
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
        fn upload(
            &self,
            artifact: &crate::Artifact,
            _: &UploadJob,
            progress: &mut dyn FnMut(UploadProgress),
        ) -> Result<UploadReceipt, UploadError> {
            progress(UploadProgress {
                transferred: artifact.size,
                total: artifact.size,
            });
            Ok(UploadReceipt {
                upload_id: "upload-1".into(),
                transferred: artifact.size,
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

    #[test]
    fn uploader_port_updates_durable_job() {
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
        let completed = upload_with_uploader(&spool, &FakeUploader, job).unwrap();
        assert_eq!(completed.state, ArtifactState::Completed);
        assert_eq!(completed.server_url.as_deref(), Some("upload-1"));
        assert_eq!(spool.ledger().pending().len(), 0);
        std::fs::remove_dir_all(root).unwrap();
    }
}
