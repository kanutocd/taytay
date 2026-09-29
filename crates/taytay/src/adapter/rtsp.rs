//! RTSP stream-health, timestamp, and bounded segmentation primitives.

use crate::{TaytayError, model::SourceId};
#[derive(Clone, Debug, Eq, PartialEq)]
/// Connection state of a supervised RTSP stream.
pub enum StreamState {
    /// No connection has been attempted.
    Disconnected,
    /// Connection setup is in progress.
    Connecting,
    /// Packets are arriving normally.
    Streaming,
    /// The stream was lost and should be reconnected.
    Reconnecting,
}
#[derive(Clone, Debug, Eq, PartialEq)]
/// Codec and timestamp state observed from an RTSP stream.
pub struct StreamMetadata {
    /// Codec identifier.
    pub codec: String,
    /// Nanoseconds per time-base unit; must be nonzero.
    pub time_base_nanos: u64,
    /// Last accepted presentation timestamp.
    pub last_timestamp_nanos: Option<u64>,
}
impl StreamMetadata {
    /// Accepts a monotonic timestamp or returns a protocol error.
    pub fn observe(&mut self, timestamp_nanos: u64) -> Result<(), TaytayError> {
        if self.time_base_nanos == 0
            || self
                .last_timestamp_nanos
                .is_some_and(|last| timestamp_nanos < last)
        {
            return Err(TaytayError::Protocol(
                "RTSP timestamp is invalid or regressed".into(),
            ));
        }
        self.last_timestamp_nanos = Some(timestamp_nanos);
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Bounds for an in-memory RTSP segment.
pub struct SegmentPolicy {
    /// Maximum segment bytes.
    pub max_bytes: usize,
    /// Maximum segment duration.
    pub max_duration_seconds: u64,
}
/// Accumulates packets until a byte or duration bound is reached.
pub struct SegmentBuffer {
    policy: SegmentPolicy,
    bytes: Vec<u8>,
    duration_seconds: u64,
}
impl SegmentBuffer {
    /// Creates a bounded segment buffer.
    pub fn new(policy: SegmentPolicy) -> Result<Self, TaytayError> {
        if policy.max_bytes == 0 || policy.max_duration_seconds == 0 {
            return Err(TaytayError::Configuration(
                "RTSP segment limits must be positive".into(),
            ));
        }
        Ok(Self {
            policy,
            bytes: Vec::new(),
            duration_seconds: 0,
        })
    }
    /// Adds a packet and returns a completed segment when bounded.
    pub fn push(&mut self, packet: &[u8], duration_seconds: u64) -> Option<Vec<u8>> {
        self.bytes.extend_from_slice(packet);
        self.duration_seconds = self.duration_seconds.saturating_add(duration_seconds);
        (self.bytes.len() >= self.policy.max_bytes
            || self.duration_seconds >= self.policy.max_duration_seconds)
            .then(|| {
                self.duration_seconds = 0;
                std::mem::take(&mut self.bytes)
            })
    }
}
/// Tracks an RTSP URI and reconnect state without performing I/O itself.
pub struct RtspAdapter {
    source: SourceId,
    uri: String,
    state: StreamState,
    reconnects: u32,
}
impl RtspAdapter {
    /// Creates an adapter for an `rtsp://` URI.
    pub fn new(source: SourceId, uri: impl Into<String>) -> Result<Self, TaytayError> {
        let uri = uri.into();
        if !uri.starts_with("rtsp://") {
            return Err(TaytayError::Configuration(
                "RTSP URI must use rtsp://".into(),
            ));
        }
        Ok(Self {
            source,
            uri,
            state: StreamState::Disconnected,
            reconnects: 0,
        })
    }
    /// Marks the stream as connecting.
    pub fn begin(&mut self) {
        self.state = StreamState::Connecting;
    }
    /// Marks the stream healthy.
    pub fn healthy(&mut self) {
        self.state = StreamState::Streaming;
    }
    /// Records a loss and enters reconnecting state.
    pub fn lost(&mut self) {
        self.state = StreamState::Reconnecting;
        self.reconnects += 1;
    }
    /// Returns the current stream state.
    pub fn state(&self) -> &StreamState {
        &self.state
    }
    /// Returns the configured source identity.
    pub fn source_id(&self) -> &SourceId {
        &self.source
    }
    /// Returns the configured RTSP URI.
    pub fn uri(&self) -> &str {
        &self.uri
    }
    /// Returns the number of recorded reconnects.
    pub fn reconnects(&self) -> u32 {
        self.reconnects
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stream_loss_enters_reconnect_state() {
        let mut adapter = RtspAdapter::new(SourceId::new("cam"), "rtsp://camera/stream").unwrap();
        adapter.begin();
        adapter.healthy();
        adapter.lost();
        assert_eq!(adapter.state(), &StreamState::Reconnecting);
        assert_eq!(adapter.reconnects(), 1);
    }
    #[test]
    fn segment_buffer_bounds_output() {
        let mut s = SegmentBuffer::new(SegmentPolicy {
            max_bytes: 3,
            max_duration_seconds: 60,
        })
        .unwrap();
        assert!(s.push(b"ab", 1).is_none());
        assert_eq!(s.push(b"c", 1), Some(b"abc".to_vec()));
        let mut metadata = StreamMetadata {
            codec: "h264".into(),
            time_base_nanos: 1,
            last_timestamp_nanos: None,
        };
        metadata.observe(2).unwrap();
        assert!(metadata.observe(1).is_err());
    }
}
