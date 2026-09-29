//! Secret-safe diagnostic formatting utilities.

use std::fmt;
/// Wrapper that renders sensitive text as `[REDACTED]` in all display forms.
pub struct Redacted<'a>(pub &'a str);
impl fmt::Debug for Redacted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
impl fmt::Display for Redacted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}
/// Removes query contents from a URL before it enters logs or metrics.
pub fn redact_url(value: &str) -> String {
    match value.split_once('?') {
        Some((base, _)) => format!("{base}?[REDACTED]"),
        None => value.to_owned(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_query() {
        assert_eq!(
            redact_url("https://x/upload?signature=secret"),
            "https://x/upload?[REDACTED]"
        );
    }
}
