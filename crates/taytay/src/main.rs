use std::{env, process::ExitCode};
use taytay::config::Config;

fn main() -> ExitCode {
    let path = env::args().nth(1).unwrap_or_else(|| "taytay.toml".into());
    match Config::load(path) {
        Ok(config) => {
            println!(
                "taytay configuration valid; spool={}, workers={}",
                config.spool_dir, config.upload_workers
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("taytay: {error}");
            ExitCode::FAILURE
        }
    }
}
