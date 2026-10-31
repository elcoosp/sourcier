//! Error types shared across the crate.
//!
//! Both error enums are [`non_exhaustive`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
//! so that new variants can be introduced in minor releases without a
//! semver-breaking change. Match arms outside of this crate should therefore
//! always contain a catch-all arm.

use thiserror::Error;

/// Errors produced while building or querying a [`SourceFilesMap`](crate::map::SourceFilesMap).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum MapError {
    /// The file id type ran out of identifiers.
    ///
    /// Each [`FileId`](crate::file_id::FileId) implementation supports a fixed
    /// number of files (`u8` supports 255, `u16` supports 65,535). Adding or
    /// finalizing beyond that limit fails with this error.
    #[error("cannot register more than {max} files for this file id type")]
    TooManyFiles {
        /// Maximum number of files supported by the id type.
        max: usize,
    },

    /// [`finalize`](crate::map::SourceFilesMap::finalize) was called on an
    /// already finalized map.
    #[error("the source files map has already been finalized")]
    AlreadyFinalized,

    /// The map was finalized and can no longer be mutated.
    #[error("cannot modify the source files map after finalization")]
    Finalized,

    /// A file exceeds the 4 GiB line offset limit of the view feature.
    ///
    /// Line offsets are stored as `u32` values to keep the index compact,
    /// which supports files of up to `u32::MAX` bytes.
    #[error("file content of {size} bytes exceeds the 4 GiB line offset limit")]
    FileTooLarge {
        /// Size of the offending file in bytes.
        size: u64,
    },
}

/// Errors produced when constructing positions with the checked
/// [`try_new`](crate::position::AbsolutePosition::try_new) constructors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum PositionError {
    /// The requested span is not a valid, ordered, one-based region.
    ///
    /// A valid span satisfies all of the following:
    ///
    /// - `start_line >= 1` and `end_line >= 1`
    /// - `start_col >= 1` and `end_col >= 1`
    /// - `start_line < end_line`, or (`start_line == end_line` and
    ///   `start_col <= end_col`)
    #[error(
        "invalid position range: start {start_line}:{start_col} must not come after end {end_line}:{end_col} and all lines and columns must be at least 1"
    )]
    InvalidRange {
        /// One-based start line of the invalid span.
        start_line: u16,
        /// One-based start column of the invalid span.
        start_col: u8,
        /// One-based end line of the invalid span.
        end_line: u16,
        /// One-based end column of the invalid span.
        end_col: u8,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_error_display_is_stable() {
        let err = MapError::TooManyFiles { max: 255 };
        assert_eq!(
            err.to_string(),
            "cannot register more than 255 files for this file id type"
        );
        assert_eq!(
            MapError::AlreadyFinalized.to_string(),
            "the source files map has already been finalized"
        );
        assert_eq!(
            MapError::Finalized.to_string(),
            "cannot modify the source files map after finalization"
        );
        assert_eq!(
            MapError::FileTooLarge { size: 1 << 32 }.to_string(),
            "file content of 4294967296 bytes exceeds the 4 GiB line offset limit"
        );
    }

    #[test]
    fn position_error_display_is_stable() {
        let err = PositionError::InvalidRange {
            start_line: 2,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        };
        assert_eq!(
            err.to_string(),
            "invalid position range: start 2:1 must not come after end 1:1 and all lines and columns must be at least 1"
        );
    }

    #[test]
    fn errors_implement_std_error() {
        let err: &dyn std::error::Error = &MapError::AlreadyFinalized;
        assert_eq!(
            err.to_string(),
            "the source files map has already been finalized"
        );
        let err: &dyn std::error::Error = &PositionError::InvalidRange {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        };
        assert!(
            err.to_string()
                .starts_with("invalid position range: start 1:1")
        );
    }
}
