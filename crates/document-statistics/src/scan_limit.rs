//! A bound on how much source content one statistics computation may scan,
//! and the typed error raised when a scan would exceed it.
//!
//! "Scanned" means bytes of [`crate::SourceItem::Text`] and
//! [`crate::items::MathItem::source`] content actually passed through
//! word/math counting (see [`crate::Statistics::scanned_bytes`]) — not bytes
//! touched only to compute the cheap [`crate::Statistics::content_hash`]
//! fingerprint used for cache-key identity. That distinction is what makes
//! it possible to prove a cache is doing something: a cache hit can skip the
//! expensive scan entirely while still checking identity cheaply.

use std::fmt;

/// Maximum number of source bytes a single call to
/// [`crate::Statistics::compute_bounded`] (or a [`crate::cache::ProjectCache`]
/// update built on top of it) is allowed to scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScanLimit {
    max_bytes: usize,
}

impl ScanLimit {
    /// No cap at all: every scan is accepted, however large.
    pub const UNBOUNDED: ScanLimit = ScanLimit {
        max_bytes: usize::MAX,
    };

    /// A cap of exactly `max_bytes` scanned source bytes.
    pub const fn new(max_bytes: usize) -> Self {
        ScanLimit { max_bytes }
    }

    /// The configured cap, in bytes.
    pub const fn max_bytes(self) -> usize {
        self.max_bytes
    }
}

impl Default for ScanLimit {
    /// One mebibyte. Generous for a single document's rendered text and math
    /// source, and small enough that a runaway input is rejected instead of
    /// scanned indefinitely.
    fn default() -> Self {
        ScanLimit::new(1 << 20)
    }
}

/// A scan was rejected because it would exceed its [`ScanLimit`].
///
/// No partial [`crate::Statistics`] is ever produced alongside this error:
/// scanning stops the moment the running byte count would exceed the limit,
/// and the caller gets only this error, never a truncated count silently
/// passed off as complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanTooLarge {
    /// Running scanned-byte count at the point the limit was exceeded.
    /// Always strictly greater than `limit`.
    pub scanned: usize,
    /// The [`ScanLimit`] that was exceeded, in bytes.
    pub limit: usize,
}

impl ScanTooLarge {
    pub(crate) fn new(scanned: usize, limit: ScanLimit) -> Self {
        ScanTooLarge {
            scanned,
            limit: limit.max_bytes(),
        }
    }
}

impl fmt::Display for ScanTooLarge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "source scan exceeded the {}-byte limit (reached {} bytes)",
            self.limit, self.scanned
        )
    }
}

impl std::error::Error for ScanTooLarge {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_limit_is_one_mebibyte() {
        assert_eq!(ScanLimit::default().max_bytes(), 1 << 20);
    }

    #[test]
    fn unbounded_is_usize_max() {
        assert_eq!(ScanLimit::UNBOUNDED.max_bytes(), usize::MAX);
    }

    #[test]
    fn error_reports_both_scanned_and_limit() {
        let err = ScanTooLarge::new(150, ScanLimit::new(100));
        assert_eq!(err.scanned, 150);
        assert_eq!(err.limit, 100);
        let msg = err.to_string();
        assert!(msg.contains("150"));
        assert!(msg.contains("100"));
    }
}
