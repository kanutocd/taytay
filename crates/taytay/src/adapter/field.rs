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

pub struct FieldFileAdapter {
    source: SourceId,
    directory: PathBuf,
    extensions: Vec<String>,
}
impl FieldFileAdapter {
    pub fn new(source: SourceId, directory: impl AsRef<Path>, extensions: &[&str]) -> Self {
        Self {
            source,
            directory: directory.as_ref().into(),
            extensions: extensions.iter().map(|x| x.to_ascii_lowercase()).collect(),
        }
    }
    pub fn scan(&self) -> Result<Vec<Artifact>, TaytayError> {
        let mut artifacts = Vec::new();
        for e in fs::read_dir(&self.directory)? {
            let e = e?;
            let p = e.path();
            if !p.is_file()
                || !self.extensions.iter().any(|x| {
                    p.extension()
                        .and_then(|v| v.to_str())
                        .is_some_and(|v| v.eq_ignore_ascii_case(x))
                })
            {
                continue;
            }
            let mut f = fs::File::open(&p)?;
            let mut h = Sha256::new();
            let mut b = [0; 64 * 1024];
            loop {
                let n = f.read(&mut b)?;
                if n == 0 {
                    break;
                }
                h.update(&b[..n]);
            }
            let checksum: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
            artifacts.push(Artifact {
                id: ArtifactId::new(&checksum[..16]),
                source: self.source.clone(),
                path: p.to_string_lossy().into(),
                media_type: "application/octet-stream".into(),
                captured_at: None,
                metadata: serde_json::json!({"field_source": self.source.0}),
                size: e.metadata()?.len(),
                checksum_sha256: Some(checksum),
            });
        }
        Ok(artifacts)
    }
}
