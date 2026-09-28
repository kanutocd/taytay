use crate::{TaytayError, model::SourceId};
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamState {
    Disconnected,
    Connecting,
    Streaming,
    Reconnecting,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentPolicy {
    pub max_bytes: usize,
    pub max_duration_seconds: u64,
}
pub struct SegmentBuffer {
    policy: SegmentPolicy,
    bytes: Vec<u8>,
    duration_seconds: u64,
}
impl SegmentBuffer {
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
pub struct RtspAdapter {
    source: SourceId,
    uri: String,
    state: StreamState,
    reconnects: u32,
}
impl RtspAdapter {
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
    pub fn begin(&mut self) {
        self.state = StreamState::Connecting;
    }
    pub fn healthy(&mut self) {
        self.state = StreamState::Streaming;
    }
    pub fn lost(&mut self) {
        self.state = StreamState::Reconnecting;
        self.reconnects += 1;
    }
    pub fn state(&self) -> &StreamState {
        &self.state
    }
    pub fn source_id(&self) -> &SourceId {
        &self.source
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
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
    }
}
