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

pub struct Ledger {
    path: std::path::PathBuf,
    jobs: Mutex<BTreeMap<ArtifactId, UploadJob>>,
}

impl Ledger {
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

    pub fn insert(&self, job: UploadJob) -> Result<(), TaytayError> {
        let mut jobs = self.jobs.lock().expect("ledger mutex poisoned");
        if jobs.contains_key(&job.artifact.id) {
            return Err(TaytayError::DuplicateArtifact(job.artifact.id.to_string()));
        }
        jobs.insert(job.artifact.id.clone(), job);
        self.persist(&jobs)
    }

    pub fn update(&self, job: UploadJob) -> Result<(), TaytayError> {
        let mut jobs = self.jobs.lock().expect("ledger mutex poisoned");
        if !jobs.contains_key(&job.artifact.id) {
            return Err(TaytayError::ArtifactNotReady(job.artifact.id.to_string()));
        }
        jobs.insert(job.artifact.id.clone(), job);
        self.persist(&jobs)
    }

    pub fn get(&self, id: &ArtifactId) -> Option<UploadJob> {
        self.jobs
            .lock()
            .expect("ledger mutex poisoned")
            .get(id)
            .cloned()
    }
    pub fn pending(&self) -> Vec<UploadJob> {
        self.jobs
            .lock()
            .expect("ledger mutex poisoned")
            .values()
            .filter(|j| {
                !matches!(
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
