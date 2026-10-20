//! Bounded executable upload runtime for the reference edge process.

use crate::{
    TaytayError,
    config::{Config, load_secret_file},
    model::{ArtifactState, UploadJob},
    retry::RetryPolicy,
    spool::Spool,
    upload::{EntregarUploader, UploadError, upload_with_uploader},
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use uuid::Uuid;

/// Summary returned after the runtime drains the jobs visible at startup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeReport {
    /// Jobs completed during this runtime invocation.
    pub completed: usize,
    /// Jobs that reached a terminal failure during this invocation.
    pub failed: usize,
}

/// Runs the configured Entregar upload workers against pending durable jobs.
///
/// The runtime intentionally drains the pending snapshot and then exits. A
/// service manager can invoke it again after new source artifacts are
/// published; an interrupted upload remains in the ledger and is recovered on
/// the next invocation.
pub async fn run(config: &Config) -> Result<RuntimeReport, TaytayError> {
    let token = load_secret_file(&config.token_file)?;
    let project_id = Uuid::parse_str(&config.project_id)
        .map_err(|_| TaytayError::Configuration("project_id must be a UUID".into()))?;
    let _organization_id = Uuid::parse_str(&config.organization_id)
        .map_err(|_| TaytayError::Configuration("organization_id must be a UUID".into()))?;
    let spool = Arc::new(Spool::open(&config.spool_dir, config.quota_bytes)?);
    let pending = spool.ledger().pending();
    if pending.is_empty() {
        return Ok(RuntimeReport {
            completed: 0,
            failed: 0,
        });
    }

    let client = lunsaran_entregar::Client::new(lunsaran_entregar::ClientConfig::new(
        config.lunsaran_base_url.clone(),
        token.expose(),
    ))
    .map_err(|error| {
        TaytayError::Configuration(format!("invalid Entregar client configuration: {error}"))
    })?;
    let uploader = Arc::new(EntregarUploader::new(
        client,
        project_id,
        std::path::Path::new(&config.spool_dir).join("resume"),
    ));
    let jobs = Arc::new(Mutex::new(VecDeque::from(pending)));
    let workers = config
        .upload_workers
        .min(jobs.lock().expect("job queue poisoned").len());
    let policy = RetryPolicy {
        max_attempts: 3,
        base_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
    };
    let mut handles = Vec::with_capacity(workers);
    for _ in 0..workers {
        let jobs = Arc::clone(&jobs);
        let spool = Arc::clone(&spool);
        let uploader = Arc::clone(&uploader);
        handles.push(tokio::spawn(async move {
            let mut report = RuntimeReport {
                completed: 0,
                failed: 0,
            };
            loop {
                let job = jobs.lock().expect("job queue poisoned").pop_front();
                let Some(job) = job else { break };
                if process_job(&spool, &uploader, job, policy).await {
                    report.completed += 1;
                } else {
                    report.failed += 1;
                }
            }
            report
        }));
    }

    let mut report = RuntimeReport {
        completed: 0,
        failed: 0,
    };
    for handle in handles {
        let worker = handle
            .await
            .map_err(|error| TaytayError::Protocol(format!("upload worker stopped: {error}")))?;
        report.completed += worker.completed;
        report.failed += worker.failed;
    }
    if report.failed > 0 {
        return Err(TaytayError::Protocol(format!(
            "{} upload job(s) failed after bounded retries",
            report.failed
        )));
    }
    Ok(report)
}

async fn process_job(
    spool: &Spool,
    uploader: &EntregarUploader,
    initial_job: UploadJob,
    policy: RetryPolicy,
) -> bool {
    let artifact_id = initial_job.artifact.id.clone();
    for attempt in 1..=policy.max_attempts {
        let job = spool
            .ledger()
            .get(&artifact_id)
            .unwrap_or(initial_job.clone());
        match upload_with_uploader(spool, uploader, job).await {
            Ok(_) => return true,
            Err(error) => {
                if !error.retryable() || !policy.should_retry(attempt) {
                    let _ = record_failure(spool, &artifact_id, &error);
                    return false;
                }
                let _ = record_failure(spool, &artifact_id, &error);
                tokio::time::sleep(policy.delay(attempt)).await;
            }
        }
    }
    false
}

fn record_failure(
    spool: &Spool,
    artifact_id: &crate::ArtifactId,
    error: &UploadError,
) -> Result<(), TaytayError> {
    let Some(mut job) = spool.ledger().get(artifact_id) else {
        return Err(TaytayError::ArtifactNotReady(artifact_id.to_string()));
    };
    if !matches!(job.state, ArtifactState::Failed) {
        job.transition(ArtifactState::Failed)?;
    }
    job.last_error = Some(safe_error(error));
    spool.ledger().update(job)
}

fn safe_error(error: &UploadError) -> String {
    match error {
        UploadError::Unauthorized(_) => "upload authorization failed".into(),
        UploadError::Expired(_) => "upload session expired".into(),
        UploadError::Retryable(_) => "transient upload failure".into(),
        UploadError::Permanent(_) => "permanent upload failure".into(),
        UploadError::Cancelled => "upload cancelled".into(),
        UploadError::Paused => "upload paused".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArtifactId, SourceId};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[tokio::test]
    async fn configured_runtime_drains_pending_job() {
        let server = lunsaran_entregar_mock::MockServer::start().await.unwrap();
        let root = std::env::temp_dir().join(format!(
            "taytay-runtime-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let token_file = root.join("token");
        fs::create_dir_all(&root).unwrap();
        fs::write(&token_file, "test-token").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&token_file, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let spool = Spool::open(&root, 1024).unwrap();
        spool
            .publish(
                ArtifactId::new("runtime-one"),
                SourceId::new("test"),
                "text/plain".into(),
                b"runtime-upload",
                None,
                serde_json::json!({}),
            )
            .unwrap();
        drop(spool);

        let config = Config {
            spool_dir: root.to_string_lossy().into_owned(),
            quota_bytes: 1024,
            upload_workers: 1,
            lunsaran_base_url: server.base_url().to_string(),
            organization_id: Uuid::nil().to_string(),
            project_id: Uuid::nil().to_string(),
            token_file: token_file.to_string_lossy().into_owned(),
        };
        let report = run(&config).await.unwrap();
        assert_eq!(
            report,
            RuntimeReport {
                completed: 1,
                failed: 0
            }
        );
        assert_eq!(server.offset().await, 14);
        server.shutdown();
        fs::remove_dir_all(root).unwrap();
    }
}
