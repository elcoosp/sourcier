//! Numeric identifiers assigned to source files.
//!
//! A [`FileId`] is a small unsigned integer (`u8` or `u16`) that stands in for
//! a file path inside packed positions. Ids are **one-based**: the first file
//! registered in a [`SourceFilesMap`](crate::map::SourceFilesMap) receives id
//! `1`, and `0` is never assigned, which leaves the all-zero encoding free for
//! future use as a sentinel.
//!
//! The trait is sealed: the only implementations are [`u8`] and [`u16`], so
//! new methods with default implementations can be added without breaking
//! downstream code.

use core::fmt::{Debug, Display};
use core::hash::Hash;

mod sealed {
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
}

/// Numeric id assigned to a source file inside packed positions.
///
/// Implementations are provided for [`u8`] (up to 255 files) and [`u16`] (up
/// to 65,535 files). Each implementation also defines where the file id sits
/// inside the 64-bit position encoding used by
/// [`AbsolutePosition`](crate::position::AbsolutePosition).
///
/// # Encoding layout
///
/// Positions pack five fields into a single `u64`: the file id, the start
/// line (16 bits), the start column (8 bits), the end line (16 bits) and the
/// end column (8 bits). The file id always occupies the most significant
/// bits, and the remaining fields are laid out so that no bits are wasted:
///
/// | Id type | File id | Start line | Start col | End line | End col |
/// |---------|---------|------------|-----------|----------|---------|
/// | `u8`    | 56..64  | 40..56     | 32..40    | 16..32   | 0..8    |
/// | `u16`   | 48..64  | 32..48     | 24..32    | 8..24    | 0..8    |
pub trait FileId:
    sealed::Sealed + Copy + Eq + Ord + Hash + Debug + Display + Into<u64> + TryFrom<u64> + 'static
{
    /// Maximum number of files addressable by this id type.
    ///
    /// Ids are one-based, so this equals the largest valid id value.
    const MAX_FILES: usize;

    /// Number of bits used by the file id field in packed positions.
    const FILE_ID_BITS: u32;

    /// Bit offset of the file id field in packed positions.
    const FILE_ID_SHIFT: u32;

    /// Bit offset of the start line field in packed positions.
    const START_LINE_SHIFT: u32;

    /// Bit offset of the start column field in packed positions.
    const START_COL_SHIFT: u32;

    /// Bit offset of the end line field in packed positions.
    const END_LINE_SHIFT: u32;

    /// Bit offset of the end column field in packed positions.
    const END_COL_SHIFT: u32;

    /// Mask selecting the file id field of a packed position.
    const FILE_ID_MASK: u64;

    /// Mask selecting a 16-bit line field of a packed position.
    const LINE_MASK: u64;

    /// Mask selecting an 8-bit column field of a packed position.
    const COL_MASK: u64;

    /// Returns the id assigned to the file at `index` (zero-based), or `None`
    /// if the index is out of range for this id type.
    ///
    /// The mapping is `index 0 -> id 1`, `index 1 -> id 2`, and so on.
    #[inline]
    fn from_index(index: usize) -> Option<Self> {
        let value = index as u64 + 1;
        Self::try_from(value).ok()
    }

    /// Returns the raw numeric value of this id.
    #[inline]
    fn to_raw(self) -> u64 {
        self.into()
    }
}

impl FileId for u8 {
    const MAX_FILES: usize = u8::MAX as usize;
    const FILE_ID_BITS: u32 = 8;
    const FILE_ID_SHIFT: u32 = 56;
    const START_LINE_SHIFT: u32 = 40;
    const START_COL_SHIFT: u32 = 32;
    const END_LINE_SHIFT: u32 = 16;
    const END_COL_SHIFT: u32 = 0;
    const FILE_ID_MASK: u64 = 0xFF00_0000_0000_0000;
    const LINE_MASK: u64 = 0xFFFF;
    const COL_MASK: u64 = 0xFF;
}

impl FileId for u16 {
    const MAX_FILES: usize = u16::MAX as usize;
    const FILE_ID_BITS: u32 = 16;
    const FILE_ID_SHIFT: u32 = 48;
    const START_LINE_SHIFT: u32 = 32;
    const START_COL_SHIFT: u32 = 24;
    const END_LINE_SHIFT: u32 = 8;
    const END_COL_SHIFT: u32 = 0;
    const FILE_ID_MASK: u64 = 0xFFFF_0000_0000_0000;
    const LINE_MASK: u64 = 0xFFFF;
    const COL_MASK: u64 = 0xFF;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_index_is_one_based() {
        assert_eq!(u8::from_index(0), Some(1));
        assert_eq!(u8::from_index(254), Some(255));
        assert_eq!(u8::from_index(255), None);
        assert_eq!(u16::from_index(0), Some(1));
        assert_eq!(u16::from_index(65_534), Some(65_535));
        assert_eq!(u16::from_index(65_535), None);
    }

    #[test]
    fn limits_match_primitive_widths() {
        assert_eq!(u8::MAX_FILES, 255);
        assert_eq!(u16::MAX_FILES, 65_535);
        assert_eq!(u8::FILE_ID_BITS, 8);
        assert_eq!(u16::FILE_ID_BITS, 16);
    }

    #[test]
    fn to_raw_round_trips() {
        assert_eq!(u8::to_raw(42), 42_u64);
        assert_eq!(u16::to_raw(1_234), 1_234_u64);
    }

    #[test]
    fn masks_select_their_fields() {
        // For u8 the file id occupies the top 8 bits.
        assert_eq!(
            ((42_u64 << u8::FILE_ID_SHIFT) & u8::FILE_ID_MASK) >> u8::FILE_ID_SHIFT,
            42
        );
        // For u16 it occupies the top 16 bits.
        assert_eq!(
            ((1_234_u64 << u16::FILE_ID_SHIFT) & u16::FILE_ID_MASK) >> u16::FILE_ID_SHIFT,
            1_234
        );
        // Line and column masks cover exactly 16 and 8 bits.
        assert_eq!(u8::LINE_MASK, 0xFFFF);
        assert_eq!(u8::COL_MASK, 0xFF);
    }
}
