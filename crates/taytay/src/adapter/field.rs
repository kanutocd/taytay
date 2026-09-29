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
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct FieldMetadata {
    pub flight_id: Option<String>,
    pub sensor_id: Option<String>,
    pub captured_at: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude_m: Option<f64>,
    pub coordinate_reference_system: Option<String>,
    pub point_count: Option<u64>,
    pub provenance: Option<String>,
}
impl FieldMetadata {
    pub fn validate(&self) -> Result<(), TaytayError> {
        if self.latitude.is_some_and(|x| !(-90.0..=90.0).contains(&x))
            || self
                .longitude
                .is_some_and(|x| !(-180.0..=180.0).contains(&x))
        {
            return Err(TaytayError::Protocol(
                "field coordinates are out of range".into(),
            ));
        }
        if self.point_count.is_some_and(|count| count == 0) {
            return Err(TaytayError::Protocol(
                "field point count must be positive".into(),
            ));
        }
        Ok(())
    }
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
                media_type: media_type_for(&p),
                captured_at: None,
                metadata: serde_json::json!({"field_source": self.source.0}),
                size: e.metadata()?.len(),
                checksum_sha256: Some(checksum),
            });
        }
        Ok(artifacts)
    }
    pub fn metadata_from_sidecar(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<FieldMetadata, TaytayError> {
        let metadata: FieldMetadata = serde_json::from_slice(&fs::read(path)?)
            .map_err(|e| TaytayError::Protocol(e.to_string()))?;
        metadata.validate()?;
        Ok(metadata)
    }
    pub fn scan_with_sidecars(&self) -> Result<Vec<Artifact>, TaytayError> {
        let mut artifacts = self.scan()?;
        for artifact in &mut artifacts {
            let sidecar = Path::new(&artifact.path).with_extension("json");
            if sidecar.exists() {
                let metadata = self.metadata_from_sidecar(sidecar)?;
                artifact.captured_at = metadata.captured_at.clone();
                artifact.metadata = serde_json::to_value(metadata)
                    .map_err(|e| TaytayError::Protocol(e.to_string()))?;
            }
        }
        Ok(artifacts)
    }
}

fn media_type_for(path: &Path) -> String {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "mp4" | "mov" => "video/mp4",
        "las" => "application/vnd.las",
        "laz" => "application/vnd.laz",
        _ => "application/octet-stream",
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_rejects_invalid_coordinates() {
        let m = FieldMetadata {
            latitude: Some(91.0),
            ..Default::default()
        };
        assert!(m.validate().is_err());
    }

    #[test]
    fn metadata_rejects_empty_point_clouds() {
        let m = FieldMetadata {
            point_count: Some(0),
            ..Default::default()
        };
        assert!(m.validate().is_err());
    }

    #[test]
    fn sidecar_metadata_is_attached_to_field_artifacts() {
        let root = std::env::temp_dir().join(format!("taytay-field-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let artifact_path = root.join("survey.laz");
        fs::write(&artifact_path, b"point-cloud").unwrap();
        fs::write(
            root.join("survey.json"),
            r#"{"flight_id":"flight-1","captured_at":"2027-01-01T00:00:00Z","point_count":42}"#,
        )
        .unwrap();

        let adapter = FieldFileAdapter::new(SourceId::new("lidar"), &root, &["laz"]);
        let artifacts = adapter.scan_with_sidecars().unwrap();
        assert_eq!(artifacts[0].media_type, "application/vnd.laz");
        assert_eq!(
            artifacts[0].captured_at.as_deref(),
            Some("2027-01-01T00:00:00Z")
        );
        assert_eq!(artifacts[0].metadata["point_count"], 42);
        fs::remove_dir_all(root).unwrap();
    }
}
