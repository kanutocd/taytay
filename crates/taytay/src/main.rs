use lunsaran_taytay::{
    ArtifactId, config::Config, operations::HealthSnapshot, runtime, spool::Spool,
};
use std::{env, process::ExitCode};

#[tokio::main]
async fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let requested = args.next().unwrap_or_else(|| "--check-config".into());
    let first_argument = args.next().unwrap_or_else(|| "taytay.toml".into());
    let (command, path, artifact_id) = if requested.starts_with("--") {
        if requested == "--pause" || requested == "--resume" {
            (
                requested,
                args.next().unwrap_or_else(|| "taytay.toml".into()),
                Some(first_argument),
            )
        } else {
            (requested, first_argument, None)
        }
    } else {
        ("--run".into(), requested, None)
    };
    match Config::load(&path) {
        Ok(config) if command == "--check-config" => {
            println!(
                "taytay configuration valid; workers={}",
                config.upload_workers
            );
            ExitCode::SUCCESS
        }
        Ok(config) if command == "--run" => match runtime::run(&config).await {
            Ok(report) => {
                println!(
                    "{{\"completed_jobs\":{},\"failed_jobs\":{}}}",
                    report.completed, report.failed
                );
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("taytay: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(config) if command == "--status" => {
            match Spool::open(&config.spool_dir, config.quota_bytes)
                .and_then(|spool| HealthSnapshot::from_spool(&spool, config.quota_bytes))
            {
                Ok(status) => {
                    println!(
                        "{{\"ready\":{},\"pending_jobs\":{},\"completed_jobs\":{},\"spool_bytes\":{},\"quota_bytes\":{}}}",
                        status.ready,
                        status.pending_jobs,
                        status.completed_jobs,
                        status.spool_bytes,
                        status.quota_bytes
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("taytay: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(config) if command == "--pause" || command == "--resume" => {
            let Some(artifact_id) = artifact_id else {
                eprintln!("taytay: missing artifact id");
                return ExitCode::FAILURE;
            };
            let result = Spool::open(&config.spool_dir, config.quota_bytes).and_then(|spool| {
                if command == "--pause" {
                    spool.pause(&ArtifactId::new(artifact_id))
                } else {
                    spool.resume(&ArtifactId::new(artifact_id))
                }
            });
            match result {
                Ok(job) => {
                    println!(
                        "{{\"artifact_id\":\"{}\",\"state\":\"{:?}\"}}",
                        job.artifact.id, job.state
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("taytay: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Ok(_) => {
            eprintln!(
                "taytay: unknown command {command}; use CONFIG, --check-config, --status, --pause, or --resume"
            );
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("taytay: {error}");
            ExitCode::FAILURE
        }
    }
}
