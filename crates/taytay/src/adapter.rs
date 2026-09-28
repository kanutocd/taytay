use crate::{
    TaytayError,
    model::{Artifact, SourceId},
};

pub trait SourceAdapter: Send {
    fn source_id(&self) -> &SourceId;
    fn poll(&mut self) -> Result<Vec<Artifact>, TaytayError>;
}

pub trait ArtifactReader: Send + Sync {
    fn open(&self, artifact: &Artifact) -> Result<Box<dyn std::io::Read + Send>, TaytayError>;
}

pub mod field;
pub mod filesystem;
pub mod onvif;
pub mod rtsp;
pub mod v4l2;
