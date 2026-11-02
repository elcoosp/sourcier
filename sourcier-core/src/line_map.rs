//! Compact line offset indexes for byte-accurate line and column navigation.
//!
//! A [`LineOffsets`] records the byte offset at which every line of a file
//! starts. Offsets are stored as `u32` values, which keeps the index at 4
//! bytes per line and supports files of up to 4 GiB.

use core::ops::Range;

use crate::error::MapError;

/// Byte offset of the start of each line in a source file.
///
/// Lines are one-based. A file that does not end with a newline still has a
/// final, possibly empty, last line. For an empty file the index contains a
/// single empty line.
///
/// Line endings: `\n` terminates a line. When a file uses `\r\n` endings the
/// carriage return is kept as the last byte of the line, so ranges may
/// include a trailing `\r`.
///
/// # Examples
///
/// ```
/// use sourcier_core::LineOffsets;
///
/// let lines = LineOffsets::compute(b"first\nsecond\nthird").expect("file within size limit");
/// assert_eq!(lines.line_count(), 3);
/// assert_eq!(lines.line_range(2), Some(6..12));
/// assert_eq!(lines.line_col_from_offset(6), Some((2, 1)));
/// assert_eq!(lines.offset_from_line_col(3, 1), Some(13));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineOffsets {
    offsets: Vec<u32>,
    content_length: usize,
}

impl LineOffsets {
    /// Maximum content size addressable by the compact `u32` offsets:
    /// 4 GiB (`u32::MAX` bytes).
    pub const MAX_CONTENT_LENGTH: usize = u32::MAX as usize;

    /// Computes the line offsets of `content`.
    ///
    /// Returns [`MapError::FileTooLarge`] when `content` exceeds
    /// [`MAX_CONTENT_LENGTH`](Self::MAX_CONTENT_LENGTH), the maximum size
    /// addressable by the compact `u32` offsets.
    pub fn compute(content: &[u8]) -> Result<Self, MapError> {
        let content_length = content.len();
        if content_length > Self::MAX_CONTENT_LENGTH {
            return Err(MapError::FileTooLarge {
                size: content_length as u64,
            });
        }
        Ok(Self::build(content, content_length))
    }

    /// Builds the offset index of pre-validated content.
    ///
    /// A single `memchr` pass fills the offset vector: the counting pre-pass
    /// of previous versions doubled the memory traffic of the hottest
    /// finalization phase for no gain, since amortized growth of the vector
    /// is far cheaper than a second SIMD scan of the whole content.
    fn build(content: &[u8], content_length: usize) -> Self {
        // Typical source files average 30-60 bytes per line; the heuristic
        // keeps reallocation rare without scanning the content twice.
        let mut offsets = Vec::with_capacity(content.len() / 32 + 8);
        offsets.push(0_u32);
        offsets.extend(memchr::memchr_iter(b'\n', content).map(|pos| (pos + 1) as u32));
        Self {
            offsets,
            content_length,
        }
    }

    /// Returns the number of lines (always at least 1).
    #[inline]
    pub fn line_count(&self) -> usize {
        self.offsets.len()
    }

    /// Returns the byte length of the indexed content.
    #[inline]
    pub fn content_length(&self) -> usize {
        self.content_length
    }

    /// Returns the raw byte offsets at which every line starts.
    ///
    /// The returned slice always starts with `0` and has exactly
    /// [`line_count`](Self::line_count) entries; it is exposed so callers can
    /// reuse the offsets in their own fast paths without re-scanning files.
    #[inline]
    pub fn line_starts(&self) -> &[u32] {
        &self.offsets
    }

    /// Returns the half-open byte range of `line` (one-based), excluding the
    /// terminating newline.
    #[inline]
    pub fn line_range(&self, line: usize) -> Option<Range<usize>> {
        if line == 0 || line > self.offsets.len() {
            return None;
        }
        let start = self.offsets[line - 1] as usize;
        let end = if line < self.offsets.len() {
            self.offsets[line] as usize - 1
        } else {
            self.content_length
        };
        Some(start..end)
    }

    /// Maps a byte offset to its one-based `(line, column)` position.
    ///
    /// An offset equal to the content length maps to the last column of the
    /// last line.
    #[inline]
    pub fn line_col_from_offset(&self, offset: usize) -> Option<(usize, usize)> {
        if offset > self.content_length {
            return None;
        }
        let line = self
            .offsets
            .partition_point(|&start| start as usize <= offset);
        let line_start = self.offsets[line - 1] as usize;
        Some((line, offset - line_start + 1))
    }

    /// Maps a one-based `(line, column)` position to its byte offset.
    ///
    /// A column pointing exactly at the end of the line resolves to the byte
    /// of the line terminator (or one past the last byte of the file).
    #[inline]
    pub fn offset_from_line_col(&self, line: usize, col: usize) -> Option<usize> {
        if col == 0 {
            return None;
        }
        let range = self.line_range(line)?;
        let offset = range.start + (col - 1);
        (offset <= range.end).then_some(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_offsets_without_trailing_newline() {
        let lines = LineOffsets::compute(b"ab\ncd\nef").expect("small file");
        assert_eq!(lines.line_count(), 3);
        assert_eq!(lines.line_range(1), Some(0..2));
        assert_eq!(lines.line_range(2), Some(3..5));
        assert_eq!(lines.line_range(3), Some(6..8));
        assert_eq!(lines.line_range(4), None);
        assert_eq!(lines.line_range(0), None);
    }

    #[test]
    fn computes_offsets_with_trailing_newline() {
        let lines = LineOffsets::compute(b"ab\ncd\n").expect("small file");
        assert_eq!(lines.line_count(), 3);
        assert_eq!(lines.line_range(1), Some(0..2));
        assert_eq!(lines.line_range(2), Some(3..5));
        assert_eq!(lines.line_range(3), Some(6..6));
    }

    #[test]
    fn empty_content_is_a_single_empty_line() {
        let lines = LineOffsets::compute(b"").expect("small file");
        assert_eq!(lines.line_count(), 1);
        assert_eq!(lines.line_range(1), Some(0..0));
        assert_eq!(lines.line_col_from_offset(0), Some((1, 1)));
    }

    #[test]
    fn crlf_keeps_carriage_return_in_the_line() {
        let lines = LineOffsets::compute(b"ab\r\ncd").expect("small file");
        assert_eq!(lines.line_range(1), Some(0..3));
        assert_eq!(lines.line_range(2), Some(4..6));
    }

    #[test]
    fn offset_to_line_col_round_trips() {
        let content = b"one\ntwo\nthree";
        let lines = LineOffsets::compute(content).expect("small file");
        for (offset, expected) in [
            (0, (1, 1)),
            (2, (1, 3)),
            (3, (1, 4)), // the newline itself
            (4, (2, 1)),
            (7, (2, 4)), // the newline itself
            (8, (3, 1)),
            (12, (3, 5)), // one past the last byte
        ] {
            assert_eq!(
                lines.line_col_from_offset(offset),
                Some(expected),
                "offset {offset}"
            );
        }
        // The content length maps to one past the last column of the last
        // line; anything beyond that is out of range.
        assert_eq!(lines.line_col_from_offset(13), Some((3, 6)));
        assert_eq!(lines.line_col_from_offset(14), None);

        for (line, col, expected) in [
            (1, 1, Some(0)),
            (1, 4, Some(3)),
            (2, 1, Some(4)),
            (3, 1, Some(8)),
            (3, 5, Some(12)),
            (3, 6, Some(13)), // one past the last byte of the file
            (4, 1, None),
        ] {
            assert_eq!(
                lines.offset_from_line_col(line, col),
                expected,
                "{line}:{col}"
            );
        }

        // Every offset maps to a position whose inverse is the same offset.
        for offset in 0..=content.len() {
            let (line, col) = lines.line_col_from_offset(offset).expect("offset in range");
            assert_eq!(lines.offset_from_line_col(line, col), Some(offset));
        }
    }

    #[test]
    fn line_offset_limit_matches_u32_addressing() {
        assert_eq!(LineOffsets::MAX_CONTENT_LENGTH, u32::MAX as usize);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn rejects_files_larger_than_u32_offsets() {
        // Large zeroed allocations are mmap-backed: untouched pages are
        // never committed to physical memory, so the 4 GiB limit can be
        // exercised here. The size check runs before any scan, so the
        // buffer is never actually read.
        let content = vec![0_u8; LineOffsets::MAX_CONTENT_LENGTH + 1];
        assert_eq!(
            LineOffsets::compute(&content),
            Err(MapError::FileTooLarge {
                size: content.len() as u64,
            })
        );
    }
}
