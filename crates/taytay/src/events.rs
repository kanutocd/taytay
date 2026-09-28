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
}
