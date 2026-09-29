use crate::{TaytayError, model::SourceId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct V4l2Format {
    pub device: String,
    pub pixel_format: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}
pub fn negotiate_format(
    supported: &[V4l2Format],
    preferred_pixels: &[&str],
    min_width: u32,
    min_height: u32,
) -> Result<V4l2Format, TaytayError> {
    preferred_pixels
        .iter()
        .find_map(|pixel| {
            supported
                .iter()
                .filter(|format| {
                    format.pixel_format == *pixel
                        && format.width >= min_width
                        && format.height >= min_height
                })
                .max_by_key(|format| (format.width.saturating_mul(format.height), format.fps))
                .cloned()
        })
        .ok_or_else(|| TaytayError::Configuration("no compatible V4L2 format".into()))
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentPolicy {
    pub max_bytes: usize,
    pub max_duration_seconds: u64,
}
pub struct Segmenter {
    policy: SegmentPolicy,
    bytes: Vec<u8>,
    elapsed_seconds: u64,
}
impl Segmenter {
    pub fn new(policy: SegmentPolicy) -> Result<Self, TaytayError> {
        if policy.max_bytes == 0 || policy.max_duration_seconds == 0 {
            return Err(TaytayError::Configuration(
                "segment limits must be positive".into(),
            ));
        }
        Ok(Self {
            policy,
            bytes: Vec::new(),
            elapsed_seconds: 0,
        })
    }
    pub fn push(&mut self, frame: &[u8], elapsed_seconds: u64) -> Option<Vec<u8>> {
        self.bytes.extend_from_slice(frame);
        self.elapsed_seconds = self.elapsed_seconds.saturating_add(elapsed_seconds);
        if self.bytes.len() >= self.policy.max_bytes
            || self.elapsed_seconds >= self.policy.max_duration_seconds
        {
            Some(std::mem::take(&mut self.bytes))
        } else {
            None
        }
    }
    pub fn finish(&mut self) -> Option<Vec<u8>> {
        (!self.bytes.is_empty()).then(|| std::mem::take(&mut self.bytes))
    }
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
    #[test]
    fn segmenter_bounds_output() {
        let mut s = Segmenter::new(SegmentPolicy {
            max_bytes: 3,
            max_duration_seconds: 60,
        })
        .unwrap();
        assert!(s.push(b"ab", 1).is_none());
        assert_eq!(s.push(b"c", 1), Some(b"abc".to_vec()));
    }
    #[test]
    fn negotiates_preferred_format() {
        let supported = vec![
            V4l2Format {
                device: "/dev/video0".into(),
                pixel_format: "YUYV".into(),
                width: 1280,
                height: 720,
                fps: 30,
            },
            V4l2Format {
                device: "/dev/video0".into(),
                pixel_format: "MJPG".into(),
                width: 1920,
                height: 1080,
                fps: 30,
            },
        ];
        assert_eq!(
            negotiate_format(&supported, &["MJPG", "YUYV"], 1280, 720)
                .unwrap()
                .pixel_format,
            "MJPG"
        );
    }
}
