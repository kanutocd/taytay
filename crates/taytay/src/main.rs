use std::{env, process::ExitCode};
use taytay::{config::Config, operations::HealthSnapshot, spool::Spool};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "--check-config".into());
    let path = args.next().unwrap_or_else(|| "taytay.toml".into());
    match Config::load(&path) {
        Ok(config) if command == "--check-config" => {
            println!(
                "taytay configuration valid; workers={}",
                config.upload_workers
            );
            ExitCode::SUCCESS
        }
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
        Ok(_) => {
            eprintln!("taytay: unknown command {command}; use --check-config or --status");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("taytay: {error}");
            ExitCode::FAILURE
        }
    }
}
