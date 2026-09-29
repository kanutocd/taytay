//! Traits and reusable boundaries for source integrations.
//!
//! Implementors should produce completed, immutable [`Artifact`] values and
//! leave queueing, checksums, retries, and upload routing to the common Taytay
//! pipeline.

use crate::{
    TaytayError,
    model::{Artifact, SourceId},
};

/// Poll-based source adapter that emits completed immutable artifacts.
pub trait SourceAdapter: Send {
    /// Returns the stable identity of this source.
    fn source_id(&self) -> &SourceId;
    /// Polls the source for newly completed artifacts.
    ///
    /// Implementations should be conservative: an artifact must not be
    /// returned while its producer may still be writing it.
    fn poll(&mut self) -> Result<Vec<Artifact>, TaytayError>;
}

/// Abstraction for streaming artifact bytes from local durable storage.
pub trait ArtifactReader: Send + Sync {
    /// Opens an artifact for streaming reads without loading it into memory.
    fn open(&self, artifact: &Artifact) -> Result<Box<dyn std::io::Read + Send>, TaytayError>;
}

pub mod field;
pub mod filesystem;
pub mod onvif;
pub mod rtsp;
pub mod v4l2;
