use crate::{
    TaytayError,
    ledger::Ledger,
    model::{Artifact, ArtifactId, UploadJob},
};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub struct Spool {
    root: PathBuf,
    quota_bytes: u64,
    ledger: Ledger,
}

impl Spool {
    pub fn open(root: impl AsRef<Path>, quota_bytes: u64) -> Result<Self, TaytayError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        let ledger = Ledger::open(root.join("ledger.jsonl"))?;
        Ok(Self {
            root,
            quota_bytes,
            ledger,
        })
    }

    pub fn publish(
        &self,
        id: ArtifactId,
        source: crate::SourceId,
        media_type: String,
        bytes: &[u8],
        captured_at: Option<String>,
        metadata: serde_json::Value,
    ) -> Result<UploadJob, TaytayError> {
        let used = self.used_bytes()?;
        let available = self.quota_bytes.saturating_sub(used);
        if bytes.len() as u64 > available {
            return Err(TaytayError::QuotaExceeded {
                requested: bytes.len() as u64,
                available,
            });
        }
        let final_path = self.root.join(format!("{}.artifact", id.0));
        if final_path.exists() {
            return Err(TaytayError::DuplicateArtifact(id.to_string()));
        }
        let temp = self.root.join(format!(".{}.part", id.0));
        let mut file = File::create(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temp, &final_path)?;
        let checksum = hex(&Sha256::digest(bytes));
        let artifact = Artifact {
            id,
            source,
            path: final_path.to_string_lossy().into_owned(),
            media_type,
            captured_at,
            metadata,
            size: bytes.len() as u64,
            checksum_sha256: Some(checksum),
        };
        let job = UploadJob::new(artifact);
        if let Err(e) = self.ledger.insert(job.clone()) {
            let _ = fs::remove_file(final_path);
            return Err(e);
        }
        Ok(job)
    }

    pub fn verify(&self, job: &UploadJob) -> Result<(), TaytayError> {
        let mut file = File::open(&job.artifact.path)?;
        let mut hasher = Sha256::new();
        let mut buf = [0; 64 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        let actual = hex(&hasher.finalize());
        let expected = job.artifact.checksum_sha256.clone().unwrap_or_default();
        if actual == expected {
            Ok(())
        } else {
            Err(TaytayError::ChecksumMismatch { expected, actual })
        }
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn pause(&self, id: &ArtifactId) -> Result<UploadJob, TaytayError> {
        self.ledger.pause(id)
    }

    pub fn resume(&self, id: &ArtifactId) -> Result<UploadJob, TaytayError> {
        self.ledger.resume(id)
    }
    pub fn cleanup_completed(&self) -> Result<usize, TaytayError> {
        let jobs = self.ledger.completed();
        let mut removed = 0;
        for job in jobs {
            if Path::new(&job.artifact.path).exists() {
                fs::remove_file(&job.artifact.path)?;
                removed += 1;
            }
        }
        Ok(removed)
    }
    fn used_bytes(&self) -> Result<u64, TaytayError> {
        Ok(fs::read_dir(&self.root)?
            .filter_map(Result::ok)
            .filter_map(|e| e.metadata().ok())
            .filter(|m| m.is_file())
            .map(|m| m.len())
            .sum())
    }

    pub fn bytes_used(&self) -> Result<u64, TaytayError> {
        self.used_bytes()
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        std::env::temp_dir().join(format!(
            "taytay-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn publication_is_checksum_verified_and_recovered() {
        let root = temp_root();
        let spool = Spool::open(&root, 1024).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("one"),
                crate::SourceId::new("test"),
                "application/octet-stream".into(),
                b"payload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        spool.verify(&job).unwrap();
        drop(spool);
        let reopened = Spool::open(&root, 1024).unwrap();
        assert_eq!(reopened.ledger().pending().len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn publication_enforces_quota() {
        let root = temp_root();
        let spool = Spool::open(&root, 2).unwrap();
        assert!(matches!(
            spool.publish(
                ArtifactId::new("one"),
                crate::SourceId::new("test"),
                "x".into(),
                b"123",
                None,
                serde_json::json!({})
            ),
            Err(TaytayError::QuotaExceeded { .. })
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cleanup_removes_only_completed_artifacts() {
        let root = temp_root();
        let spool = Spool::open(&root, 1024).unwrap();
        let mut job = spool
            .publish(
                ArtifactId::new("one"),
                crate::SourceId::new("test"),
                "x".into(),
                b"payload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        assert_eq!(spool.cleanup_completed().unwrap(), 0);
        job.transition(crate::ArtifactState::Uploading).unwrap();
        job.transition(crate::ArtifactState::Completed).unwrap();
        spool.ledger().update(job).unwrap();
        assert_eq!(spool.cleanup_completed().unwrap(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
