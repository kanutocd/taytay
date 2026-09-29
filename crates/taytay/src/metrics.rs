use crate::operations::CounterSnapshot;

pub fn prometheus(snapshot: CounterSnapshot) -> String {
    format!(
        "# TYPE taytay_uploads_total counter\ntaytay_uploads_total {}\n# TYPE taytay_retries_total counter\ntaytay_retries_total {}\n# TYPE taytay_bytes_transferred_total counter\ntaytay_bytes_transferred_total {}\n# TYPE taytay_source_errors_total counter\ntaytay_source_errors_total {}\n",
        snapshot.uploads, snapshot.retries, snapshot.bytes_transferred, snapshot.source_errors
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exports_redacted_metric_names() {
        let text = prometheus(CounterSnapshot {
            uploads: 1,
            retries: 2,
            bytes_transferred: 3,
            source_errors: 4,
        });
        assert!(text.contains("taytay_uploads_total 1"));
        assert!(!text.contains("token"));
    }
}
