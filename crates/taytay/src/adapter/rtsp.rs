use crate::{TaytayError, model::SourceId};
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StreamState {
    Disconnected,
    Connecting,
    Streaming,
    Reconnecting,
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
