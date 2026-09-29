//! Linux V4L2 format negotiation and bounded segmenting primitives.

use crate::{TaytayError, model::SourceId};

#[derive(Clone, Debug, Eq, PartialEq)]
/// A camera format candidate returned by V4L2 enumeration.
pub struct V4l2Format {
    /// Device node.
    pub device: String,
    /// FourCC or negotiated pixel format.
    pub pixel_format: String,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Frames per second.
    pub fps: u32,
}
/// Selects the highest-capacity supported format in preferred pixel order.
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
/// Bounds for an in-memory camera segment.
pub struct SegmentPolicy {
    /// Maximum segment bytes.
    pub max_bytes: usize,
    /// Maximum segment duration.
    pub max_duration_seconds: u64,
}
/// Accumulates frames until a byte or time bound is reached.
pub struct Segmenter {
    policy: SegmentPolicy,
    bytes: Vec<u8>,
    elapsed_seconds: u64,
}
impl Segmenter {
    /// Creates a segmenter with positive byte and duration limits.
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
    /// Adds a frame and returns a completed segment when a bound is reached.
    pub fn push(&mut self, frame: &[u8], elapsed_seconds: u64) -> Option<Vec<u8>> {
        self.bytes.extend_from_slice(frame);
        self.elapsed_seconds = self.elapsed_seconds.saturating_add(elapsed_seconds);
        if self.bytes.len() >= self.policy.max_bytes
            || self.elapsed_seconds >= self.policy.max_duration_seconds
        {
            self.elapsed_seconds = 0;
            Some(std::mem::take(&mut self.bytes))
        } else {
            None
        }
    }
    /// Flushes a final partial segment, if any.
    pub fn finish(&mut self) -> Option<Vec<u8>> {
        (!self.bytes.is_empty()).then(|| std::mem::take(&mut self.bytes))
    }
}

/// Tracks a configured V4L2 device's connection state.
pub struct V4l2Adapter {
    source: SourceId,
    format: V4l2Format,
    connected: bool,
}
impl V4l2Adapter {
    /// Creates a disconnected adapter for a validated format.
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
    /// Marks the device connected.
    pub fn connect(&mut self) {
        self.connected = true;
    }
    /// Marks the device disconnected.
    pub fn disconnect(&mut self) {
        self.connected = false;
    }
    /// Returns whether the adapter is connected.
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    /// Returns the source identity.
    pub fn source_id(&self) -> &SourceId {
        &self.source
    }
    /// Returns the negotiated format.
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
        assert!(s.push(b"d", 1).is_none());
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
