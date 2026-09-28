use serde::Deserialize;
use std::{fs, path::Path};

use crate::TaytayError;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub spool_dir: String,
    #[serde(default = "default_quota")]
    pub quota_bytes: u64,
    #[serde(default = "default_workers")]
    pub upload_workers: usize,
    pub lunsaran_base_url: String,
    pub organization_id: String,
    pub project_id: String,
    pub token_file: String,
}

fn default_quota() -> u64 {
    10 * 1024 * 1024 * 1024
}
fn default_workers() -> usize {
    2
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TaytayError> {
        let raw = fs::read_to_string(path)?;
        let config: Self =
            toml::from_str(&raw).map_err(|e| TaytayError::Configuration(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), TaytayError> {
        if self.spool_dir.is_empty()
            || self.lunsaran_base_url.is_empty()
            || self.organization_id.is_empty()
            || self.project_id.is_empty()
            || self.token_file.is_empty()
        {
            return Err(TaytayError::Configuration("spool_dir, lunsaran_base_url, organization_id, project_id, and token_file are required".into()));
        }
        if self.quota_bytes == 0 || self.upload_workers == 0 {
            return Err(TaytayError::Configuration(
                "quota_bytes and upload_workers must be positive".into(),
            ));
        }
        if self.token_file.contains("..") {
            return Err(TaytayError::Configuration(
                "token_file must not contain parent traversal".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_zero_workers() {
        let c = Config {
            spool_dir: "spool".into(),
            quota_bytes: 1,
            upload_workers: 0,
            lunsaran_base_url: "https://lunsaran".into(),
            organization_id: "o".into(),
            project_id: "p".into(),
            token_file: "token".into(),
        };
        assert!(c.validate().is_err());
    }
}
