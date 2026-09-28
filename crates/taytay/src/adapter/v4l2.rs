use crate::{TaytayError, model::SourceId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V4l2Format {
    pub device: String,
    pub pixel_format: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

pub struct V4l2Adapter {
    source: SourceId,
    format: V4l2Format,
    connected: bool,
}
impl V4l2Adapter {
    pub fn new(source: SourceId, format: V4l2Format) -> Result<Self, TaytayError> {
        if format.width == 0 || format.height == 0 || format.fps == 0 {
            return Err(TaytayError::Configuration(
                "V4L2 dimensions and fps must be positive".into(),
            ));
        }
        Ok(Self {
            source,
            format,
            connected: false,
        })
    }
    pub fn connect(&mut self) {
        self.connected = true;
    }
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    pub fn source_id(&self) -> &SourceId {
        &self.source
    }
    pub fn format(&self) -> &V4l2Format {
        &self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reconnect_is_explicit() {
        let mut a = V4l2Adapter::new(
            SourceId::new("usb"),
            V4l2Format {
                device: "/dev/video0".into(),
                pixel_format: "MJPG".into(),
                width: 1920,
                height: 1080,
                fps: 30,
            },
        )
        .unwrap();
        assert!(!a.is_connected());
        a.connect();
        assert!(a.is_connected());
        a.disconnect();
        assert!(!a.is_connected());
    }
}
