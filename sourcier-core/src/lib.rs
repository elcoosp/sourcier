//! Compact and efficient source file mapping and position tracking.
//!
//! `sourcier-core` stores source files together with compact numeric ids and
//! packs positions (file id, line and column spans) into single `u64`
//! values. It is designed for tools that need cheap diagnostics-style
//! positions over many files: compilers, linters, language servers and code
//! editors.
//!
//! Most users should depend on the [`sourcier`](https://docs.rs/sourcier)
//! facade crate, which re-exports everything from here.
//!
//! # Workflow
//!
//! 1. Register files in a [`SourceFilesMap`] — or in the type-state
//!    [`SourceFilesBuilder`], which only exposes the registration API — and
//!    call [`finalize`](SourceFilesMap::finalize) to assign stable ids.
//! 2. Create [`AbsolutePosition`]s (or file-agnostic
//!    [`RelativePosition`]s) that pack a span into one `u64`.
//! 3. Query the map: paths, contents and, with the `view` feature, the exact
//!    bytes covered by a position. Byte offsets and positions convert in
//!    O(log n) with [`position_from_offset`](SourceFilesMap::position_from_offset)
//!    and [`offset_from_position`](SourceFilesMap::offset_from_position).
//!
//! ```
//! use sourcier_core::{AbsolutePosition, SourceFilePosition, SourceFilesMap};
//!
//! let mut files = SourceFilesMap::<u8>::new();
//! files.add_file("src/main.rs", b"fn main() {\n    println!(\"Hello\");\n}").expect("capacity available");
//! files.finalize().expect("no capacity issue");
//!
//! let id = files.get_id("src/main.rs").expect("file present");
//! let pos = AbsolutePosition::new(id, 2, 5, 2, 22);
//! assert_eq!(pos.start_line(), 2);
//! assert_eq!(files.view(id, &pos), Some(&b"println!(\"Hello\");"[..]));
//! ```
//!
//! # Feature flags
//!
//! | Feature       | Default | Description                                            |
//! |---------------|---------|--------------------------------------------------------|
//! | `serde`       | yes     | Serialization support for maps and positions.        |
//! | `view`        | yes     | Line offset indexes plus `view`/`view_str` lookups.  |
//! | `rt-feedback` | yes     | Lock-free shared usage statistics (`RuntimeFeedback`). |
//! | `parallel`    | no      | Multi-threaded line-offset index construction (rayon). |
//!
//! # Performance characteristics
//!
//! - Positions are single `u64` values: `Copy`, cache-friendly and free to
//!   store by value.
//! - Id, path and content lookups are O(1); line offset lookup is O(log n).
//! - Finalization is a single unstable sort (with a registration-order
//!   tiebreak) plus one O(n) pass; no content is copied and no per-path
//!   string is allocated (paths up to 30 bytes are stored inline).
//! - Path lookups run through a verified open-addressed index: per-map
//!   seeded hashing, 4 bytes per slot and hash-plus-bytes verification.
//! - Line offsets use a single `memchr`-accelerated scan and 4 bytes per
//!   line; the `parallel` feature spreads the scan across threads.
//! - Serialized map payloads carry only the raw entries and the phase flag;
//!   lookup state is derived and rebuilt on load.
//!
//! # Limits
//!
//! - File ids are one-based; `u8` supports 255 files, `u16` supports 65,535.
//! - Lines are `u16` (up to 65,535) and columns are `u8` (up to 255).
//! - Line offset indexes address files of up to 4 GiB.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Public modules.
pub mod builder;
pub mod error;
#[cfg(feature = "rt-feedback")]
#[cfg_attr(docsrs, doc(cfg(feature = "rt-feedback")))]
pub mod feedback;
pub mod file_id;
#[cfg(feature = "view")]
#[cfg_attr(docsrs, doc(cfg(feature = "view")))]
pub mod line_map;
pub mod map;
pub mod position;

// Re-export commonly used types for convenience.
pub use builder::SourceFilesBuilder;
pub use error::{MapError, PositionError};
#[cfg(feature = "rt-feedback")]
pub use feedback::{FeedbackSnapshot, RuntimeFeedback};
pub use file_id::FileId;
#[cfg(feature = "view")]
pub use line_map::LineOffsets;
pub use map::{FileRef, Iter, PositionDisplay, SourceFilesMap};
pub use position::{
    AbsolutePosition, CompactAbsolutePosition, RelativePosition, SourceFilePosition,
    StandardAbsolutePosition,
};

#[cfg(test)]
mod tests;
