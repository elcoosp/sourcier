//! Source positions packed into a single `u64`.
//!
//! Two position types are provided:
//!
//! - [`AbsolutePosition`] carries a [`FileId`] and therefore addresses a
//!   position in a specific file of a
//!   [`SourceFilesMap`](crate::map::SourceFilesMap).
//! - [`RelativePosition`] is file-agnostic and only carries line and column
//!   information.
//!
//! All lines and columns are **one-based**. Lines are stored as `u16` and
//! columns as `u8`, so a position can address files of up to 65,535 lines
//! with columns 1 to 255. The packed representation orders the fields as
//! `file id | start line | start column | end line | end column`, which makes
//! ordering by the packed value equivalent to a natural lexicographic
//! ordering of spans.

use core::fmt;

use crate::error::PositionError;
use crate::file_id::FileId;

/// Uniform access to the components of a source position.
///
/// Implemented by [`AbsolutePosition`] and [`RelativePosition`]; useful for
/// writing generic helpers over both kinds of positions.
pub trait SourceFilePosition {
    /// Returns the source file id, or `None` for file-agnostic positions.
    fn source_file_id(&self) -> Option<u16>;

    /// Returns the one-based start line.
    fn start_line(&self) -> u16;

    /// Returns the one-based start column.
    fn start_column(&self) -> u8;

    /// Returns the one-based end line.
    fn end_line(&self) -> u16;

    /// Returns the one-based end column.
    fn end_column(&self) -> u8;
}

/// A span inside a specific source file, packed into a single `u64`.
///
/// The file id, start line, start column, end line and end column are stored
/// in the bit layout documented on [`FileId`]. The end line/column are
/// **inclusive**.
///
/// # Examples
///
/// ```
/// use sourcier_core::{AbsolutePosition, SourceFilePosition, SourceFilesMap};
///
/// let mut files = SourceFilesMap::<u8>::new();
/// files.add_file("src/main.rs", b"fn main() {}").expect("capacity available");
/// files.finalize().expect("no capacity issue");
///
/// let id = files.get_id("src/main.rs").expect("file present");
/// let pos = AbsolutePosition::new(id, 1, 4, 1, 7);
///
/// assert_eq!(pos.file_id(), id);
/// assert_eq!(pos.start_line(), 1);
/// assert_eq!(pos.start_column(), 4);
/// assert_eq!(pos.end_line(), 1);
/// assert_eq!(pos.end_column(), 7);
///
/// // Positions are cheap to copy, compare and serialize.
/// let copy = AbsolutePosition::from_raw(pos.as_raw()).expect("raw value came from a valid position");
/// assert_eq!(pos, copy);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbsolutePosition<Id: FileId>(u64, core::marker::PhantomData<Id>);

impl<Id: FileId> fmt::Debug for AbsolutePosition<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbsolutePosition")
            .field("file_id", &self.file_id())
            .field(
                "start",
                &format_args!("{}:{}", self.start_line(), self.start_column()),
            )
            .field(
                "end",
                &format_args!("{}:{}", self.end_line(), self.end_column()),
            )
            .finish()
    }
}

impl<Id: FileId> AbsolutePosition<Id> {
    /// Packs a new absolute position from its components.
    ///
    /// This constructor is infallible and performs no validation: values are
    /// encoded as-is. Use [`try_new`](Self::try_new) when the span should be
    /// validated as a well-formed, one-based region.
    #[inline]
    pub fn new(file_id: Id, start_line: u16, start_col: u8, end_line: u16, end_col: u8) -> Self {
        let file_id = file_id.to_raw();
        let encoded = (file_id << Id::FILE_ID_SHIFT)
            | ((u64::from(start_line)) << Id::START_LINE_SHIFT)
            | ((u64::from(start_col)) << Id::START_COL_SHIFT)
            | ((u64::from(end_line)) << Id::END_LINE_SHIFT)
            | (u64::from(end_col));
        Self(encoded, core::marker::PhantomData)
    }

    /// Packs a new absolute position, validating the span first.
    ///
    /// Returns [`PositionError::InvalidRange`] when any line or column is
    /// zero, or when the start does not come before (or exactly at) the end.
    #[inline]
    pub fn try_new(
        file_id: Id,
        start_line: u16,
        start_col: u8,
        end_line: u16,
        end_col: u8,
    ) -> Result<Self, PositionError> {
        let ordered = start_line < end_line || (start_line == end_line && start_col <= end_col);
        let positive = start_line >= 1 && start_col >= 1 && end_line >= 1 && end_col >= 1;
        if !(ordered && positive) {
            return Err(PositionError::InvalidRange {
                start_line,
                start_col,
                end_line,
                end_col,
            });
        }
        Ok(Self::new(file_id, start_line, start_col, end_line, end_col))
    }

    /// Rebuilds a position from its raw 64-bit representation, or returns
    /// `None` if the embedded file id cannot be represented by `Id` (this
    /// includes the never-assigned id `0`).
    #[inline]
    pub fn from_raw(raw: u64) -> Option<Self> {
        let value = (raw & Id::FILE_ID_MASK) >> Id::FILE_ID_SHIFT;
        if value == 0 {
            return None;
        }
        Id::try_from(value)
            .ok()
            .map(|_| Self(raw, core::marker::PhantomData))
    }

    /// Returns the raw 64-bit representation.
    ///
    /// The layout is stable across semver-compatible releases; see the
    /// [`FileId`] documentation for the exact bit positions.
    #[inline]
    pub const fn as_raw(&self) -> u64 {
        self.0
    }

    /// Strips the file id, keeping only the span components.
    ///
    /// The result encodes the same start and end line/column in the
    /// file-agnostic [`RelativePosition`] layout, so spans can be stored or
    /// compared without binding them to a specific file.
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::{AbsolutePosition, RelativePosition};
    ///
    /// let abs = AbsolutePosition::<u8>::new(7, 2, 5, 2, 22);
    /// assert_eq!(abs.to_relative(), RelativePosition::new(2, 5, 2, 22));
    /// ```
    #[inline]
    pub fn to_relative(self) -> RelativePosition {
        RelativePosition::new(
            self.start_line(),
            self.start_column(),
            self.end_line(),
            self.end_column(),
        )
    }

    /// Returns the file id component.
    #[inline]
    pub fn file_id(&self) -> Id {
        let value = (self.0 & Id::FILE_ID_MASK) >> Id::FILE_ID_SHIFT;
        // The id field is exactly `FILE_ID_BITS` wide, so every value that
        // can enter a position through `new` or `from_raw` converts back.
        Id::try_from(value)
            .ok()
            .expect("positions only encode valid file ids")
    }
}

impl<Id: FileId> SourceFilePosition for AbsolutePosition<Id> {
    #[inline]
    fn source_file_id(&self) -> Option<u16> {
        // The id field is at most 16 bits wide, so the value always fits.
        Some(((self.0 & Id::FILE_ID_MASK) >> Id::FILE_ID_SHIFT) as u16)
    }

    #[inline]
    fn start_line(&self) -> u16 {
        ((self.0 >> Id::START_LINE_SHIFT) & Id::LINE_MASK) as u16
    }

    #[inline]
    fn start_column(&self) -> u8 {
        ((self.0 >> Id::START_COL_SHIFT) & Id::COL_MASK) as u8
    }

    #[inline]
    fn end_line(&self) -> u16 {
        ((self.0 >> Id::END_LINE_SHIFT) & Id::LINE_MASK) as u16
    }

    #[inline]
    fn end_column(&self) -> u8 {
        ((self.0 >> Id::END_COL_SHIFT) & Id::COL_MASK) as u8
    }
}

impl<Id: FileId> fmt::Display for AbsolutePosition<Id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start_line(),
            self.start_column(),
            self.end_line(),
            self.end_column()
        )
    }
}

/// A span without a file reference, packed into the low 48 bits of a `u64`.
///
/// The layout is `start line (bits 32..48) | start column (24..32) | end line
/// (8..24) | end column (0..8)`; the high 16 bits are always zero. The end
/// line/column are **inclusive**.
///
/// # Examples
///
/// ```
/// use sourcier_core::{RelativePosition, SourceFilePosition};
///
/// let pos = RelativePosition::new(2, 5, 3, 10);
/// assert_eq!(pos.source_file_id(), None);
/// assert_eq!(pos.start_line(), 2);
/// assert_eq!(pos.end_column(), 10);
/// assert_eq!(pos.to_string(), "2:5-3:10");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RelativePosition(u64);

impl fmt::Debug for RelativePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RelativePosition")
            .field(
                "start",
                &format_args!("{}:{}", self.start_line(), self.start_column()),
            )
            .field(
                "end",
                &format_args!("{}:{}", self.end_line(), self.end_column()),
            )
            .finish()
    }
}

impl RelativePosition {
    const START_LINE_SHIFT: u32 = 32;
    const START_COL_SHIFT: u32 = 24;
    const END_LINE_SHIFT: u32 = 8;
    const LINE_MASK: u64 = 0xFFFF;
    const COL_MASK: u64 = 0xFF;

    /// Packs a new relative position from its components.
    ///
    /// This constructor is infallible and performs no validation. Use
    /// [`try_new`](Self::try_new) for a checked variant.
    ///
    /// The constructor is a constant function, so spans can be built and
    /// stored in `const` or `static` items.
    #[inline]
    pub const fn new(start_line: u16, start_col: u8, end_line: u16, end_col: u8) -> Self {
        // Widening integer casts are lossless and constant-stable, unlike
        // the `From<u64>` conversions used by the non-const counterparts.
        let encoded = ((start_line as u64) << Self::START_LINE_SHIFT)
            | ((start_col as u64) << Self::START_COL_SHIFT)
            | ((end_line as u64) << Self::END_LINE_SHIFT)
            | (end_col as u64);
        Self(encoded)
    }

    /// Packs a new relative position, validating the span first.
    ///
    /// Returns [`PositionError::InvalidRange`] when any line or column is
    /// zero, or when the start does not come before (or exactly at) the end.
    #[inline]
    pub fn try_new(
        start_line: u16,
        start_col: u8,
        end_line: u16,
        end_col: u8,
    ) -> Result<Self, PositionError> {
        let ordered = start_line < end_line || (start_line == end_line && start_col <= end_col);
        let positive = start_line >= 1 && start_col >= 1 && end_line >= 1 && end_col >= 1;
        if !(ordered && positive) {
            return Err(PositionError::InvalidRange {
                start_line,
                start_col,
                end_line,
                end_col,
            });
        }
        Ok(Self::new(start_line, start_col, end_line, end_col))
    }

    /// Rebuilds a relative position from its raw 64-bit representation.
    ///
    /// The high 16 bits must be zero for the round trip to be lossless; use
    /// [`as_raw`](Self::as_raw) to obtain compatible values. Like
    /// [`new`](Self::new), this is a constant function.
    #[inline]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw & 0x0000_FFFF_FFFF_FFFF)
    }

    /// Returns the raw 64-bit representation.
    #[inline]
    pub const fn as_raw(&self) -> u64 {
        self.0
    }

    /// Attaches a file id, turning the span into an
    /// [`AbsolutePosition`].
    ///
    /// The span components are re-encoded into the absolute layout of the
    /// given [`FileId`] type; nothing else changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::{AbsolutePosition, RelativePosition};
    ///
    /// let rel = RelativePosition::new(2, 5, 2, 22);
    /// let abs = rel.to_absolute(3_u8);
    /// assert_eq!(abs, AbsolutePosition::<u8>::new(3, 2, 5, 2, 22));
    /// ```
    #[inline]
    pub fn to_absolute<Id: FileId>(self, file_id: Id) -> AbsolutePosition<Id> {
        AbsolutePosition::new(
            file_id,
            self.start_line(),
            self.start_column(),
            self.end_line(),
            self.end_column(),
        )
    }
}

impl SourceFilePosition for RelativePosition {
    #[inline]
    fn source_file_id(&self) -> Option<u16> {
        None
    }

    #[inline]
    fn start_line(&self) -> u16 {
        ((self.0 >> Self::START_LINE_SHIFT) & Self::LINE_MASK) as u16
    }

    #[inline]
    fn start_column(&self) -> u8 {
        ((self.0 >> Self::START_COL_SHIFT) & Self::COL_MASK) as u8
    }

    #[inline]
    fn end_line(&self) -> u16 {
        ((self.0 >> Self::END_LINE_SHIFT) & Self::LINE_MASK) as u16
    }

    #[inline]
    fn end_column(&self) -> u8 {
        (self.0 & Self::COL_MASK) as u8
    }
}

impl fmt::Display for RelativePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start_line(),
            self.start_column(),
            self.end_line(),
            self.end_column()
        )
    }
}

/// Absolute position using `u8` file ids, supporting up to 255 files.
pub type CompactAbsolutePosition = AbsolutePosition<u8>;

/// Absolute position using `u16` file ids, supporting up to 65,535 files.
pub type StandardAbsolutePosition = AbsolutePosition<u16>;

impl<Id: FileId> From<AbsolutePosition<Id>> for RelativePosition {
    fn from(abs: AbsolutePosition<Id>) -> Self {
        abs.to_relative()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_round_trips_all_components() {
        let pos = AbsolutePosition::<u8>::new(42, 1, 2, 65_535, 255);
        assert_eq!(pos.file_id(), 42);
        assert_eq!(pos.start_line(), 1);
        assert_eq!(pos.start_column(), 2);
        assert_eq!(pos.end_line(), 65_535);
        assert_eq!(pos.end_column(), 255);
    }

    #[test]
    fn absolute_u16_round_trips_all_components() {
        let pos = AbsolutePosition::<u16>::new(65_535, 32_000, 128, 1, 1);
        assert_eq!(pos.file_id(), 65_535);
        assert_eq!(pos.start_line(), 32_000);
        assert_eq!(pos.start_column(), 128);
        assert_eq!(pos.end_line(), 1);
        assert_eq!(pos.end_column(), 1);
    }

    #[test]
    fn from_raw_validates_the_embedded_file_id() {
        // A u16 id of 300 fits the 16-bit id field and round trips.
        let pos = AbsolutePosition::<u16>::new(300, 10, 5, 12, 20);
        assert!(AbsolutePosition::<u16>::from_raw(pos.as_raw()).is_some());

        // Zeroing the id field yields the never-assigned id 0, which both
        // id types reject.
        let zeroed = pos.as_raw() & !u16::FILE_ID_MASK;
        assert!(AbsolutePosition::<u16>::from_raw(zeroed).is_none());
        assert!(AbsolutePosition::<u8>::from_raw(zeroed).is_none());
    }

    #[test]
    fn try_new_rejects_bad_spans() {
        assert!(AbsolutePosition::<u8>::try_new(1, 0, 1, 1, 1).is_err());
        assert!(AbsolutePosition::<u8>::try_new(1, 1, 0, 1, 1).is_err());
        assert!(AbsolutePosition::<u8>::try_new(1, 2, 1, 1, 1).is_err());
        assert!(AbsolutePosition::<u8>::try_new(1, 1, 5, 1, 4).is_err());
        assert!(AbsolutePosition::<u8>::try_new(1, 1, 1, 1, 1).is_ok());
        assert!(AbsolutePosition::<u8>::try_new(1, 1, 4, 1, 5).is_ok());
        assert!(RelativePosition::try_new(0, 1, 1, 1).is_err());
        assert!(RelativePosition::try_new(1, 1, 0, 1).is_err());
    }

    #[test]
    fn relative_round_trips_all_components() {
        let pos = RelativePosition::new(65_535, 255, 1, 2);
        assert_eq!(pos.start_line(), 65_535);
        assert_eq!(pos.start_column(), 255);
        assert_eq!(pos.end_line(), 1);
        assert_eq!(pos.end_column(), 2);
        assert_eq!(pos.source_file_id(), None);
    }

    #[test]
    fn from_raw_masks_high_bits() {
        let pos = RelativePosition::from_raw(0xFFFF_0000_0000_0005);
        assert_eq!(pos.end_column(), 5);
        assert_eq!(pos.start_line(), 0);
    }

    #[test]
    fn display_uses_line_col_pairs() {
        let pos = AbsolutePosition::<u8>::new(1, 10, 5, 12, 20);
        assert_eq!(pos.to_string(), "10:5-12:20");
        let rel = RelativePosition::new(3, 1, 4, 2);
        assert_eq!(rel.to_string(), "3:1-4:2");
    }

    #[test]
    fn debug_shows_decoded_components() {
        let pos = AbsolutePosition::<u8>::new(7, 3, 4, 5, 6);
        let rendered = format!("{pos:?}");
        assert!(rendered.contains("file_id: 7"), "{rendered}");
        assert!(rendered.contains("start: 3:4"), "{rendered}");
        assert!(rendered.contains("end: 5:6"), "{rendered}");

        let rel = RelativePosition::new(3, 4, 5, 6);
        let rendered = format!("{rel:?}");
        assert!(rendered.contains("start: 3:4"), "{rendered}");
        assert!(rendered.contains("end: 5:6"), "{rendered}");
    }

    #[test]
    fn ordering_follows_lexicographic_span_order() {
        let a = AbsolutePosition::<u8>::new(1, 1, 1, 1, 1);
        let b = AbsolutePosition::<u8>::new(1, 1, 1, 1, 2);
        let c = AbsolutePosition::<u8>::new(1, 2, 1, 2, 1);
        let d = AbsolutePosition::<u8>::new(2, 1, 1, 1, 1);
        assert!(a < b && b < c && c < d);
    }
}
