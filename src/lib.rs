//! Compact and efficient source file mapping and position tracking.
//!
//! `sourcier` stores source files together with compact numeric ids and
//! packs positions (file id, line and column spans) into single `u64`
//! values. It is designed for tools that need cheap diagnostics-style
//! positions over many files: compilers, linters, language servers and code
//! editors.
//!
//! This crate is a thin facade over
//! [`sourcier-core`](https://docs.rs/sourcier-core); everything is
//! re-exported here so `sourcier::...` is the only path you ever need. See
//! the [workspace repository](https://github.com/elcoosp/sourcier) for the
//! full documentation, feature flags and limits.
//!
//! # Quick start
//!
//! ```
//! use sourcier::{AbsolutePosition, SourceFilePosition, SourceFilesMap};
//!
//! let mut files = SourceFilesMap::<u8>::new();
//! files.add_file("src/main.rs", b"fn main() {\n    println!(\"Hello\");\n}").expect("capacity available");
//! files.add_file("src/lib.rs", b"pub mod utils;").expect("capacity available");
//! files.finalize().expect("no capacity issue");
//!
//! // Ids are assigned in sorted path order, starting at 1.
//! let id = files.get_id("src/main.rs").expect("file present");
//! assert_eq!(files.get_path(id), Some("src/main.rs"));
//!
//! // Positions pack file id, lines and columns into one u64.
//! let pos = AbsolutePosition::new(id, 2, 5, 2, 22);
//! assert_eq!(pos.start_line(), 2);
//!
//! // With the default `view` feature, extract the exact bytes of a span.
//! assert_eq!(files.view(id, &pos), Some(&b"println!(\"Hello\");"[..]));
//! ```
//!
//! # Feature flags
//!
//! All features are enabled by default and forwarded to `sourcier-core`:
//!
//! - `serde` - serialization support for maps and positions.
//! - `view` - line offset indexes plus `view`/`view_str` lookups.
//! - `rt-feedback` - lock-free shared usage statistics.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use sourcier_core::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_resolves_the_core_api() {
        // The facade is a re-export surface: branch-free calls verify the
        // names resolve and behave through `sourcier::...` paths. The full
        // build/finalize/view flow is exercised by the doc examples below.
        let files = SourceFilesMap::<u8>::new();
        assert!(files.is_empty());
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn facade_resolves_the_position_api() {
        let pos = AbsolutePosition::new(1_u8, 2, 3, 4, 5);
        let _: CompactAbsolutePosition = pos;
        let _: StandardAbsolutePosition = AbsolutePosition::new(1_u16, 1, 1, 1, 1);
        assert_eq!(pos.file_id(), 1);
        assert_eq!(pos.start_line(), 2);
        assert_eq!(pos.start_column(), 3);
        assert_eq!(pos.end_line(), 4);
        assert_eq!(pos.end_column(), 5);

        let rel = RelativePosition::new(2, 3, 4, 5);
        assert_eq!(rel.source_file_id(), None);
        assert_eq!(rel.to_string(), "2:3-4:5");
    }

    #[test]
    fn facade_resolves_the_error_and_trait_api() {
        fn summarize(pos: &impl SourceFilePosition) -> u16 {
            pos.start_line()
        }

        let _: MapError = MapError::AlreadyFinalized;
        let _: PositionError = PositionError::InvalidRange {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        };

        assert_eq!(summarize(&AbsolutePosition::new(1_u8, 9, 1, 9, 1)), 9);
        assert_eq!(summarize(&RelativePosition::new(9, 1, 9, 1)), 9);

        let _: u64 = u8::to_raw(1_u8);
        assert_eq!(u8::from_index(0), Some(1));
    }

    #[cfg(feature = "view")]
    #[test]
    fn facade_resolves_the_view_api() {
        let offsets = LineOffsets::compute(b"one\ntwo").expect("fits");
        assert_eq!(offsets.line_count(), 2);
        // Interior lines end before the newline...
        assert_eq!(offsets.line_range(2), Some(4..7));
        // ...line 0 and past-the-end resolve to nothing...
        assert_eq!(offsets.line_range(0), None);
        assert_eq!(offsets.line_range(3), None);
        // ...and the final line of a file without a trailing newline runs
        // to the content length.
        let single = LineOffsets::compute(b"only").expect("fits");
        assert_eq!(single.line_range(1), Some(0..4));
    }

    #[cfg(feature = "rt-feedback")]
    #[test]
    fn facade_resolves_the_feedback_api() {
        let feedback = RuntimeFeedback::new();
        assert_eq!(feedback.snapshot().usage_count, 0);
        let _: FeedbackSnapshot = feedback.snapshot();
    }
}
