use crate::{
    TaytayError,
    model::UploadJob,
    retry::{RetryPolicy, bounded_channel},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, SyncSender, TrySendError},
};
use std::time::Duration;

#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub async fn cancelled(&self) {
        while !self.is_cancelled() {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
}

pub struct JobQueue {
    sender: SyncSender<UploadJob>,
    receiver: Receiver<UploadJob>,
    token: CancellationToken,
    policy: RetryPolicy,
}
impl JobQueue {
    pub fn new(capacity: usize, policy: RetryPolicy) -> Result<Self, TaytayError> {
        let (sender, receiver) = bounded_channel(capacity)?;
        Ok(Self {
            sender,
            receiver,
            token: CancellationToken::default(),
            policy,
        })
    }
    pub fn submit(&self, job: UploadJob) -> Result<(), Box<UploadJob>> {
        if self.token.is_cancelled() {
            return Err(Box::new(job));
        }
        match self.sender.try_send(job) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(job) | TrySendError::Disconnected(job)) => Err(Box::new(job)),
        }
    }
    pub fn receive(&self) -> Option<UploadJob> {
        if self.token.is_cancelled() {
            None
        } else {
            self.receiver.try_recv().ok()
        }
    }
    pub fn cancel(&self) {
        self.token.cancel();
    }
    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
    pub fn retry_delay(&self, attempt: u32) -> Option<Duration> {
        self.policy
            .should_retry(attempt)
            .then(|| self.policy.delay(attempt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_stops_new_work() {
        let q = JobQueue::new(
            1,
            RetryPolicy {
                max_attempts: 3,
                base_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(2),
            },
        )
        .unwrap();
        q.cancel();
        assert!(q.is_cancelled());
        assert!(q.receive().is_none());
        assert_eq!(q.retry_delay(1), Some(Duration::from_secs(1)));
    }
}
