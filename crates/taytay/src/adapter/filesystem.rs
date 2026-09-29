//! Completion-safe adapter for files exported by an NVR or mounted share.

use super::SourceAdapter;
use crate::{
    TaytayError,
    model::{Artifact, ArtifactId, SourceId},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

/// Emits a file only after two consecutive stable metadata observations.
pub struct FilesystemAdapter {
    source: SourceId,
    directory: PathBuf,
    seen: std::collections::HashSet<PathBuf>,
    observed: std::collections::HashMap<PathBuf, (u64, std::time::SystemTime)>,
}

impl FilesystemAdapter {
    /// Creates an adapter for a source directory.
    pub fn new(source: SourceId, directory: impl AsRef<Path>) -> Self {
        Self {
            source,
            directory: directory.as_ref().into(),
            seen: Default::default(),
            observed: Default::default(),
        }
    }
}

impl SourceAdapter for FilesystemAdapter {
    fn source_id(&self) -> &SourceId {
        &self.source
    }
    /// Scans for newly stable files and computes their SHA-256 identities.
    fn poll(&mut self) -> Result<Vec<Artifact>, TaytayError> {
        let mut found = Vec::new();
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || self.seen.contains(&path) {
                continue;
            }
            let metadata = entry.metadata()?;
            let fingerprint = (
                metadata.len(),
                metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
            );
            if self.observed.insert(path.clone(), fingerprint) != Some(fingerprint) {
                continue;
            }
            let mut file = fs::File::open(&path)?;
            let mut hasher = Sha256::new();
            let mut buf = [0; 64 * 1024];
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            let checksum: String = hasher
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            found.push(Artifact { id: ArtifactId::new(checksum[..16].to_string()), source: self.source.clone(), path: path.to_string_lossy().into_owned(), media_type: mime_for(&path), captured_at: None, metadata: serde_json::json!({"filename": path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown")}), size: metadata.len(), checksum_sha256: Some(checksum) });
            self.seen.insert(path);
        }
        Ok(found)
    }
}

fn mime_for(path: &Path) -> String {
    match path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "mp4" => "video/mp4",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };
    #[test]
    fn emits_only_after_two_stable_observations() {
        let root = std::env::temp_dir().join(format!(
            "taytay-files-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("clip.mp4"), b"clip").unwrap();
        let mut adapter = FilesystemAdapter::new(SourceId::new("nvr"), &root);
        assert!(adapter.poll().unwrap().is_empty());
        let found = adapter.poll().unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].media_type, "video/mp4");
        fs::remove_dir_all(root).unwrap();
    }
}
