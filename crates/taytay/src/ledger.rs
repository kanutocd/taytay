//! Atomic JSON-lines ledger for durable upload jobs.

use crate::{
    TaytayError,
    model::{ArtifactId, UploadJob},
};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::Mutex,
};

/// Durable job index persisted by atomic replacement.
pub struct Ledger {
    path: std::path::PathBuf,
    jobs: Mutex<BTreeMap<ArtifactId, UploadJob>>,
}

impl Ledger {
    /// Opens or creates a ledger and reconstructs its in-memory index.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, TaytayError> {
        let path = path.as_ref().to_path_buf();
        let mut jobs = BTreeMap::new();
        if path.exists() {
            for line in BufReader::new(File::open(&path)?).lines() {
                let line = line?;
                if !line.is_empty() {
                    let job: UploadJob = serde_json::from_str(&line)?;
                    jobs.insert(job.artifact.id.clone(), job);
                }
            }
        } else if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self {
            path,
            jobs: Mutex::new(jobs),
        })
    }

    /// Inserts a new artifact job, rejecting duplicate identities.
    pub fn insert(&self, job: UploadJob) -> Result<(), TaytayError> {
        let mut jobs = self.jobs.lock().expect("ledger mutex poisoned");
        if jobs.contains_key(&job.artifact.id) {
            return Err(TaytayError::DuplicateArtifact(job.artifact.id.to_string()));
        }
        jobs.insert(job.artifact.id.clone(), job);
        self.persist(&jobs)
    }

    /// Replaces an existing job and durably persists the new state.
    pub fn update(&self, job: UploadJob) -> Result<(), TaytayError> {
        let mut jobs = self.jobs.lock().expect("ledger mutex poisoned");
        if !jobs.contains_key(&job.artifact.id) {
            return Err(TaytayError::ArtifactNotReady(job.artifact.id.to_string()));
        }
        jobs.insert(job.artifact.id.clone(), job);
        self.persist(&jobs)
    }

    /// Returns a snapshot of one job by identity.
    pub fn get(&self, id: &ArtifactId) -> Option<UploadJob> {
        self.jobs
            .lock()
            .expect("ledger mutex poisoned")
            .get(id)
            .cloned()
    }
    /// Returns jobs eligible for scheduling.
    pub fn pending(&self) -> Vec<UploadJob> {
        self.jobs
            .lock()
            .expect("ledger mutex poisoned")
            .values()
            .filter(|j| {
                !matches!(
                    j.state,
                    crate::model::ArtifactState::Completed
                        | crate::model::ArtifactState::Retained
                        | crate::model::ArtifactState::Paused
                )
            })
            .cloned()
            .collect()
    }

    /// Returns jobs explicitly paused by an operator.
    pub fn paused(&self) -> Vec<UploadJob> {
        self.jobs
            .lock()
            .expect("ledger mutex poisoned")
            .values()
            .filter(|j| matches!(j.state, crate::model::ArtifactState::Paused))
            .cloned()
            .collect()
    }

    /// Pauses a job and persists the transition.
    pub fn pause(&self, id: &ArtifactId) -> Result<UploadJob, TaytayError> {
        let mut job = self
            .get(id)
            .ok_or_else(|| TaytayError::ArtifactNotReady(id.to_string()))?;
        job.transition(crate::model::ArtifactState::Paused)?;
        self.update(job.clone())?;
        Ok(job)
    }

    /// Resumes a paused job and persists the transition.
    pub fn resume(&self, id: &ArtifactId) -> Result<UploadJob, TaytayError> {
        let mut job = self
            .get(id)
            .ok_or_else(|| TaytayError::ArtifactNotReady(id.to_string()))?;
        job.transition(crate::model::ArtifactState::Published)?;
        self.update(job.clone())?;
        Ok(job)
    }
    /// Returns completed or retained jobs eligible for cleanup.
    pub fn completed(&self) -> Vec<UploadJob> {
        self.jobs
            .lock()
            .expect("ledger mutex poisoned")
            .values()
            .filter(|j| {
                matches!(
                    j.state,
                    crate::model::ArtifactState::Completed | crate::model::ArtifactState::Retained
                )
            })
            .cloned()
            .collect()
    }

    fn persist(&self, jobs: &BTreeMap<ArtifactId, UploadJob>) -> Result<(), TaytayError> {
        let tmp = self.path.with_extension("tmp");
        let mut file = File::create(&tmp)?;
        for job in jobs.values() {
            writeln!(file, "{}", serde_json::to_string(job)?)?;
        }
        file.sync_all()?;
        fs::rename(tmp, &self.path)?;
        let _ = OpenOptions::new().read(true).open(&self.path)?.sync_all();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ArtifactId, SourceId, spool::Spool};

    #[test]
    fn pause_and_resume_are_durable_and_excluded_from_pending() {
        let root = std::env::temp_dir().join(format!("taytay-ledger-{}", std::process::id()));
        let spool = Spool::open(&root, 100).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("pause-me"),
                SourceId::new("camera"),
                "application/octet-stream".into(),
                b"data",
                None,
                serde_json::json!({}),
            )
            .unwrap();

        let paused = spool.pause(&job.artifact.id).unwrap();
        assert_eq!(paused.state, crate::ArtifactState::Paused);
        assert!(spool.ledger().pending().is_empty());
        assert_eq!(spool.ledger().paused().len(), 1);

        let resumed = spool.resume(&job.artifact.id).unwrap();
        assert_eq!(resumed.state, crate::ArtifactState::Published);
        assert_eq!(spool.ledger().pending().len(), 1);

        let reopened = Spool::open(&root, 100).unwrap();
        assert_eq!(
            reopened.ledger().get(&job.artifact.id).unwrap().state,
            crate::ArtifactState::Published
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
