use crate::{
    TaytayError,
    model::{ArtifactState, UploadJob},
    protocol::{LunsaranClient, TusClient},
    spool::Spool,
};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

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
    let upload_url = tus.create(&session, &job.artifact)?;
    let mut offset = tus.head(&upload_url)?.offset;
    let mut file = File::open(&job.artifact.path)?;
    file.seek(SeekFrom::Start(offset))?;
    let chunk_size = session.chunk_size.max(1) as usize;
    let mut buf = vec![0; chunk_size];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let result = tus.patch(
            &upload_url,
            offset,
            &buf[..n],
            job.artifact.checksum_sha256.as_deref(),
        )?;
        offset = result.offset;
    }
    job.server_url = Some(upload_url);
    job.offset = offset;
    job.transition(ArtifactState::Completed)?;
    client.report_state(&job.artifact, "completed")?;
    spool.ledger().update(job.clone())?;
    Ok(job)
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
}
