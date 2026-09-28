use crate::{
    model::{ArtifactState, UploadJob},
    spool::Spool,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    pub completed_for: Duration,
    pub failed_for: Duration,
}
impl RetentionPolicy {
    pub fn keep(&self, state: &ArtifactState, age: Duration) -> bool {
        match state {
            ArtifactState::Completed | ArtifactState::Retained => age < self.completed_for,
            ArtifactState::Failed => age < self.failed_for,
            _ => true,
        }
    }
}

#[derive(Debug, Default)]
pub struct Counters {
    pub uploads: AtomicU64,
    pub retries: AtomicU64,
    pub bytes_transferred: AtomicU64,
    pub source_errors: AtomicU64,
}
impl Counters {
    pub fn record_upload(&self, bytes: u64) {
        self.uploads.fetch_add(1, Ordering::Relaxed);
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn record_retry(&self) {
        self.retries.fetch_add(1, Ordering::Relaxed);
    }
    pub fn record_source_error(&self) {
        self.source_errors.fetch_add(1, Ordering::Relaxed);
    }
    pub fn snapshot(&self) -> CounterSnapshot {
        CounterSnapshot {
            uploads: self.uploads.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            bytes_transferred: self.bytes_transferred.load(Ordering::Relaxed),
            source_errors: self.source_errors.load(Ordering::Relaxed),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CounterSnapshot {
    pub uploads: u64,
    pub retries: u64,
    pub bytes_transferred: u64,
    pub source_errors: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthSnapshot {
    pub ready: bool,
    pub pending_jobs: usize,
    pub completed_jobs: usize,
    pub spool_bytes: u64,
    pub quota_bytes: u64,
    pub generated_at: SystemTime,
}
impl HealthSnapshot {
    pub fn from_spool(spool: &Spool, quota_bytes: u64) -> Result<Self, crate::TaytayError> {
        let pending = spool.ledger().pending();
        let completed = spool.ledger().completed();
        let spool_bytes = spool.bytes_used()?;
        Ok(Self {
            ready: quota_bytes > 0 && spool_bytes <= quota_bytes,
            pending_jobs: pending.len(),
            completed_jobs: completed.len(),
            spool_bytes,
            quota_bytes,
            generated_at: SystemTime::now(),
        })
    }
}

pub fn record_failure(
    spool: &Spool,
    mut job: UploadJob,
    error: impl Into<String>,
) -> Result<UploadJob, crate::TaytayError> {
    job.fail(error)?;
    spool.ledger().update(job.clone())?;
    Ok(job)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArtifactId, SourceId};
    use std::time::Duration;
    #[test]
    fn retention_and_counters_are_deterministic() {
        let policy = RetentionPolicy {
            completed_for: Duration::from_secs(10),
            failed_for: Duration::from_secs(20),
        };
        assert!(policy.keep(&ArtifactState::Completed, Duration::from_secs(9)));
        assert!(!policy.keep(&ArtifactState::Failed, Duration::from_secs(20)));
        let c = Counters::default();
        c.record_upload(4);
        c.record_retry();
        c.record_source_error();
        assert_eq!(
            c.snapshot(),
            CounterSnapshot {
                uploads: 1,
                retries: 1,
                bytes_transferred: 4,
                source_errors: 1
            }
        );
    }
    #[test]
    fn failure_is_persisted() {
        let root = std::env::temp_dir().join(format!("taytay-ops-{}", std::process::id()));
        let spool = Spool::open(&root, 100).unwrap();
        let job = spool
            .publish(
                ArtifactId::new("a"),
                SourceId::new("s"),
                "x".into(),
                b"x",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        let failed = record_failure(&spool, job, "offline").unwrap();
        assert_eq!(failed.state, ArtifactState::Failed);
        assert_eq!(failed.last_error.as_deref(), Some("offline"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
