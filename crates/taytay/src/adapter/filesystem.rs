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

pub struct FilesystemAdapter {
    source: SourceId,
    directory: PathBuf,
    seen: std::collections::HashSet<PathBuf>,
}

impl FilesystemAdapter {
    pub fn new(source: SourceId, directory: impl AsRef<Path>) -> Self {
        Self {
            source,
            directory: directory.as_ref().into(),
            seen: Default::default(),
        }
    }
}

impl SourceAdapter for FilesystemAdapter {
    fn source_id(&self) -> &SourceId {
        &self.source
    }
    fn poll(&mut self) -> Result<Vec<Artifact>, TaytayError> {
        let mut found = Vec::new();
        for entry in fs::read_dir(&self.directory)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() || self.seen.contains(&path) {
                continue;
            }
            let metadata = entry.metadata()?;
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
