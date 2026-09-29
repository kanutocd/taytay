use crate::{TaytayError, model::SourceId};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MotionEvent {
    pub id: String,
    pub source: SourceId,
    pub observed_at: String,
    pub kind: String,
    pub confidence: Option<u8>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapturePolicy {
    pub pre_event_seconds: u64,
    pub post_event_seconds: u64,
    pub minimum_confidence: Option<u8>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureWindow {
    pub start_offset_seconds: i64,
    pub end_offset_seconds: u64,
}
impl CapturePolicy {
    pub fn window_for(&self, event: &MotionEvent) -> Option<CaptureWindow> {
        if self
            .minimum_confidence
            .is_some_and(|min| event.confidence.unwrap_or(0) < min)
        {
            None
        } else {
            Some(CaptureWindow {
                start_offset_seconds: -(self.pre_event_seconds as i64),
                end_offset_seconds: self.post_event_seconds,
            })
        }
    }
}

#[derive(Default)]
pub struct EventDeduplicator {
    seen: HashMap<String, String>,
}
impl EventDeduplicator {
    pub fn accept(&mut self, event: MotionEvent) -> Result<Option<MotionEvent>, TaytayError> {
        if event.id.is_empty() || event.source.0.is_empty() {
            return Err(TaytayError::Protocol("event identity is required".into()));
        }
        if self
            .seen
            .insert(event.id.clone(), event.observed_at.clone())
            .is_some()
        {
            return Ok(None);
        }
        Ok(Some(event))
    }
    pub fn forget_before(&mut self, observed_at: &str) {
        self.seen.retain(|_, value| value.as_str() >= observed_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_events_are_dropped() {
        let mut d = EventDeduplicator::default();
        let event = MotionEvent {
            id: "e1".into(),
            source: SourceId::new("cam"),
            observed_at: "2027-01-01T00:00:00Z".into(),
            kind: "motion".into(),
            confidence: Some(90),
        };
        assert!(d.accept(event.clone()).unwrap().is_some());
        assert!(d.accept(event).unwrap().is_none());
    }
    #[test]
    fn policy_filters_confidence_and_builds_window() {
        let policy = CapturePolicy {
            pre_event_seconds: 5,
            post_event_seconds: 10,
            minimum_confidence: Some(80),
        };
        let low = MotionEvent {
            id: "low".into(),
            source: SourceId::new("cam"),
            observed_at: "t".into(),
            kind: "motion".into(),
            confidence: Some(79),
        };
        assert!(policy.window_for(&low).is_none());
        let high = MotionEvent {
            confidence: Some(80),
            ..low
        };
        assert_eq!(
            policy.window_for(&high),
            Some(CaptureWindow {
                start_offset_seconds: -5,
                end_offset_seconds: 10
            })
        );
    }
}
