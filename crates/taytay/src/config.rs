//! Configuration and protected secret loading for an edge process.

use serde::Deserialize;
use std::{fs, path::Path};

use crate::TaytayError;

#[derive(Clone, Deserialize)]
/// A secret value whose debug representation is always redacted.
pub struct Secret(String);
impl Secret {
    /// Returns the secret for the narrow operation that needs it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}
/// Loads a non-empty secret file and rejects group/other-readable Unix modes.
pub fn load_secret_file(path: impl AsRef<Path>) -> Result<Secret, TaytayError> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(TaytayError::Configuration(
                "credential file permissions must not grant access to group or other users".into(),
            ));
        }
    }
    let value = fs::read_to_string(path)?.trim().to_owned();
    if value.is_empty() {
        return Err(TaytayError::Configuration(
            "credential file is empty".into(),
        ));
    }
    Ok(Secret(value))
}
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
/// Runtime configuration for the reference edge process.
pub struct Config {
    /// Local artifact spool and ledger directory.
    pub spool_dir: String,
    /// Maximum number of bytes retained in the local spool.
    #[serde(default = "default_quota")]
    pub quota_bytes: u64,
    /// Maximum number of concurrent upload workers.
    #[serde(default = "default_workers")]
    pub upload_workers: usize,
    /// Lunsaran control-plane origin.
    pub lunsaran_base_url: String,
    /// Organization scope for newly created upload sessions.
    pub organization_id: String,
    /// Project scope for newly created upload sessions.
    pub project_id: String,
    /// Protected file containing the device/workload credential.
    pub token_file: String,
}

fn default_quota() -> u64 {
    10 * 1024 * 1024 * 1024
}
fn default_workers() -> usize {
    2
}

impl Config {
    /// Loads TOML configuration and validates security-sensitive fields.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TaytayError> {
        let raw = fs::read_to_string(path)?;
        let config: Self =
            toml::from_str(&raw).map_err(|e| TaytayError::Configuration(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    /// Validates required values, quota/worker bounds, token path, and URL safety.
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
        validate_base_url(&self.lunsaran_base_url)?;
        Ok(())
    }
}

/// Validates an HTTP(S) origin without userinfo, query, or fragment data.
pub fn validate_base_url(value: &str) -> Result<(), TaytayError> {
    if !(value.starts_with("https://") || value.starts_with("http://"))
        || value.contains('?')
        || value.contains('#')
        || value
            .split_once("://")
            .is_some_and(|(_, rest)| rest.contains('@'))
    {
        return Err(TaytayError::InvalidUrl(
            "base URL must be an HTTP(S) origin without userinfo, query, or fragment".into(),
        ));
    }
    Ok(())
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

    #[test]
    fn rejects_credential_bearing_url() {
        let c = Config {
            spool_dir: "spool".into(),
            quota_bytes: 1,
            upload_workers: 1,
            lunsaran_base_url: "https://user:pass@example.test".into(),
            organization_id: "o".into(),
            project_id: "p".into(),
            token_file: "token".into(),
        };
        assert!(c.validate().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn secret_loader_rejects_broad_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let path = std::env::temp_dir().join(format!("taytay-secret-{}", std::process::id()));
        fs::write(&path, "device-token").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(load_secret_file(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}
