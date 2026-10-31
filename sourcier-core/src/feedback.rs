//! Optional runtime usage statistics for finalized source file maps.
//!
//! Shared statistics are exposed through a lock-free
//! `Arc<RuntimeFeedback>`: finalizing a map updates the counters with atomic
//! operations, so multiple maps and threads can report into the same context
//! without blocking.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Lock-free usage statistics shared between source file maps.
///
/// Create one context with [`RuntimeFeedback::new`], wrap it in an `Arc` and
/// hand it to [`SourceFilesMap::with_feedback`](crate::map::SourceFilesMap::with_feedback).
/// Every successful finalization overwrites the totals of that map and
/// increments the usage counter.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
///
/// use sourcier_core::{RuntimeFeedback, SourceFilesMap};
///
/// let feedback = Arc::new(RuntimeFeedback::new());
/// let mut files = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
/// files.add_file("a.rs", b"fn a() {}").expect("capacity available");
/// files.finalize().expect("no capacity issue");
///
/// let snapshot = feedback.snapshot();
/// assert_eq!(snapshot.total_files, 1);
/// assert_eq!(snapshot.total_bytes, 9);
/// assert_eq!(snapshot.usage_count, 1);
/// ```
#[derive(Debug, Default)]
pub struct RuntimeFeedback {
    total_files: AtomicUsize,
    total_bytes: AtomicU64,
    max_file_size: AtomicUsize,
    usage_count: AtomicU64,
}

/// An immutable snapshot of [`RuntimeFeedback`] counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FeedbackSnapshot {
    /// Number of files registered by the last finalized map.
    pub total_files: usize,
    /// Total number of content bytes registered by the last finalized map.
    pub total_bytes: u64,
    /// Size in bytes of the largest file of the last finalized map.
    pub max_file_size: usize,
    /// Number of finalizations that reported into this context.
    pub usage_count: u64,
}

impl RuntimeFeedback {
    /// Creates an empty feedback context.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records the statistics of one finalization.
    pub(crate) fn record(&self, total_files: usize, total_bytes: u64, max_file_size: usize) {
        self.total_files.store(total_files, Ordering::Relaxed);
        self.total_bytes.store(total_bytes, Ordering::Relaxed);
        self.max_file_size.store(max_file_size, Ordering::Relaxed);
        self.usage_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns an immutable snapshot of the current counters.
    ///
    /// The individual counters are read independently; a snapshot taken
    /// concurrently with a finalization may mix values from two maps.
    #[inline]
    pub fn snapshot(&self) -> FeedbackSnapshot {
        FeedbackSnapshot {
            total_files: self.total_files.load(Ordering::Relaxed),
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            max_file_size: self.max_file_size.load(Ordering::Relaxed),
            usage_count: self.usage_count.load(Ordering::Relaxed),
        }
    }
}

impl Clone for RuntimeFeedback {
    fn clone(&self) -> Self {
        let snapshot = self.snapshot();
        Self {
            total_files: AtomicUsize::new(snapshot.total_files),
            total_bytes: AtomicU64::new(snapshot.total_bytes),
            max_file_size: AtomicUsize::new(snapshot.max_file_size),
            usage_count: AtomicU64::new(snapshot.usage_count),
        }
    }
}

impl PartialEq for RuntimeFeedback {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot() == other.snapshot()
    }
}

impl Eq for RuntimeFeedback {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        let feedback = RuntimeFeedback::new();
        assert_eq!(feedback.snapshot(), FeedbackSnapshot::default());
    }

    #[test]
    fn record_overwrites_totals_and_increments_usage() {
        let feedback = RuntimeFeedback::new();
        feedback.record(3, 100, 60);
        feedback.record(1, 10, 10);
        let snapshot = feedback.snapshot();
        assert_eq!(snapshot.total_files, 1);
        assert_eq!(snapshot.total_bytes, 10);
        assert_eq!(snapshot.max_file_size, 10);
        assert_eq!(snapshot.usage_count, 2);
    }

    #[test]
    fn clone_and_equality_compare_snapshots() {
        let feedback = RuntimeFeedback::new();
        feedback.record(2, 30, 20);
        let clone = feedback.clone();
        assert_eq!(feedback, clone);
        clone.record(5, 50, 50);
        assert_ne!(feedback, clone);
    }
}
