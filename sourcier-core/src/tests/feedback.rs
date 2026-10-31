//! Integration tests for the `rt-feedback` feature.

#![cfg(feature = "rt-feedback")]

use std::sync::Arc;

use crate::{FeedbackSnapshot, RuntimeFeedback, SourceFilesMap};

#[test]
fn finalization_reports_into_the_shared_context() {
    let feedback = Arc::new(RuntimeFeedback::new());
    let mut files = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
    files.add_file("one.rs", b"0123456789").expect("fits");
    files.add_file("two.rs", b"01").expect("fits");
    files.finalize().expect("fits");

    assert_eq!(
        feedback.snapshot(),
        FeedbackSnapshot {
            total_files: 2,
            total_bytes: 12,
            max_file_size: 10,
            usage_count: 1,
        }
    );
}

#[test]
fn multiple_maps_share_one_context() {
    let feedback = Arc::new(RuntimeFeedback::new());

    let mut first = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
    first.add_file("a.rs", b"aaaa").expect("fits");
    first.finalize().expect("fits");

    let mut second = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
    second.add_file("b.rs", b"bb").expect("fits");
    second.add_file("c.rs", b"ccc").expect("fits");
    second.finalize().expect("fits");

    let snapshot = feedback.snapshot();
    assert_eq!(snapshot.total_files, 2, "last finalization wins for totals");
    assert_eq!(snapshot.total_bytes, 5);
    assert_eq!(snapshot.max_file_size, 3);
    assert_eq!(snapshot.usage_count, 2);
}

#[test]
fn u16_maps_report_into_the_shared_context() {
    // The feedback path is generic over the id type; exercise it with u16
    // ids as well so every instantiation of the reporting code runs.
    let feedback = Arc::new(RuntimeFeedback::new());
    let mut files = SourceFilesMap::<u16>::with_feedback(Arc::clone(&feedback));
    files.add_file("a.rs", b"alpha").expect("fits");
    files.finalize().expect("fits");

    let snapshot = feedback.snapshot();
    assert_eq!(snapshot.total_files, 1);
    assert_eq!(snapshot.total_bytes, 5);
    assert_eq!(snapshot.max_file_size, 5);
    assert_eq!(snapshot.usage_count, 1);
}

#[test]
fn maps_without_feedback_do_not_panic() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("a.rs", b"a").expect("fits");
    files.finalize().expect("fits");
    assert_eq!(files.len(), 1);
}

#[test]
fn empty_finalization_reports_zeroed_totals() {
    let feedback = Arc::new(RuntimeFeedback::new());
    let mut files = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
    files.finalize().expect("no files");

    assert_eq!(
        feedback.snapshot(),
        FeedbackSnapshot {
            total_files: 0,
            total_bytes: 0,
            max_file_size: 0,
            usage_count: 1,
        }
    );
}

#[test]
fn failed_finalizations_do_not_report() {
    let feedback = Arc::new(RuntimeFeedback::new());
    let mut files = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
    files.add_file("a.rs", b"a").expect("fits");
    assert_eq!(files.finalize(), Ok(()));
    assert_eq!(files.finalize(), Err(crate::MapError::AlreadyFinalized));
    assert_eq!(feedback.snapshot().usage_count, 1);
}

#[test]
fn feedback_is_shareable_across_threads() {
    let feedback = Arc::new(RuntimeFeedback::new());
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let feedback = Arc::clone(&feedback);
            std::thread::spawn(move || {
                let mut files = SourceFilesMap::<u8>::with_feedback(Arc::clone(&feedback));
                files
                    .add_file(format!("t{i}.rs"), vec![0_u8; 10])
                    .expect("fits");
                files.finalize().expect("fits");
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("thread finishes");
    }
    assert_eq!(feedback.snapshot().usage_count, 4);
}
