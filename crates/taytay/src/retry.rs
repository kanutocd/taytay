use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}
impl RetryPolicy {
    pub fn delay(&self, attempt: u32) -> Duration {
        self.base_delay
            .saturating_mul(2u32.saturating_pow(attempt.saturating_sub(1)))
            .min(self.max_delay)
    }
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }
}
pub fn bounded_channel<T>(
    capacity: usize,
) -> Result<(std::sync::mpsc::SyncSender<T>, std::sync::mpsc::Receiver<T>), crate::TaytayError> {
    if capacity == 0 {
        return Err(crate::TaytayError::Configuration(
            "queue capacity must be positive".into(),
        ));
    }
    Ok(std::sync::mpsc::sync_channel(capacity))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backoff_is_bounded() {
        let p = RetryPolicy {
            max_attempts: 8,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(4),
        };
        assert_eq!(p.delay(1), Duration::from_secs(1));
        assert_eq!(p.delay(5), Duration::from_secs(4));
        assert!(!p.should_retry(8));
    }
}
