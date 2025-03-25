#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;
use std::marker::PhantomData;

/// Trait defining core file ID behavior for numeric ID types
pub trait FileId:
    Copy + Eq + Hash + Into<u64> + TryFrom<u64> + Ord + std::fmt::Debug + 'static
{
    /// Maximum number of files supported by this ID type
    const MAX_FILES: usize;

    /// Maximum valid ID value (exclusive)
    const MAX_ID: u64;

    /// Number of bits needed to represent this file ID
    const FILE_ID_BITS: u32;

    /// Position of file ID bits in encoded positions
    const FILE_ID_SHIFT: u32;

    /// Bit positions for encoded source positions
    const START_LINE_SHIFT: u32;
    const START_COL_SHIFT: u32;
    const END_LINE_SHIFT: u32;
    const END_COL_SHIFT: u32;

    /// Bit masks for decoding components
    const FILE_ID_MASK: u64;
    const LINE_MASK: u64;
    const COL_MASK: u64;
}

macro_rules! impl_file_id {
    ($t:ty, $bits:expr, $file_shift:expr) => {
        impl FileId for $t {
            const MAX_FILES: usize = <$t>::MAX as usize;
            const MAX_ID: u64 = <$t>::MAX as u64 + 1;

            const FILE_ID_BITS: u32 = $bits;
            const FILE_ID_SHIFT: u32 = $file_shift;
            const START_LINE_SHIFT: u32 = $file_shift - 16;
            const START_COL_SHIFT: u32 = $file_shift - 24;
            const END_LINE_SHIFT: u32 = $file_shift - 40;
            const END_COL_SHIFT: u32 = $file_shift - 48;
            const FILE_ID_MASK: u64 = ((1 << $bits) - 1) << $file_shift;
            const LINE_MASK: u64 = 0xFFFF;
            const COL_MASK: u64 = 0xFF;
        }
    };
}

// Implement for common ID types
impl_file_id!(u8, 8, 56);
impl_file_id!(u16, 16, 48);

/// Trait for extracting source position information
pub trait SourceFilePosition {
    /// Get the source file ID or None for relative positions
    fn source_file_id(&self) -> Option<u16>;

    /// Get the start line number
    fn start_line(&self) -> u16;

    /// Get the start column number
    fn start_column(&self) -> u8;

    /// Get the end line number
    fn end_line(&self) -> u16;

    /// Get the end column number
    fn end_column(&self) -> u8;
}

/// Position with absolute file reference
///
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound = "Id: Serialize + serde::de::DeserializeOwned")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbsolutePosition<Id: FileId>(u64, PhantomData<Id>);

impl<Id: FileId> AbsolutePosition<Id> {
    /// Create a new absolute position
    pub fn new(file_id: Id, start_line: u16, start_col: u8, end_line: u16, end_col: u8) -> Self {
        let file_id_u64: u64 = file_id.into();

        let encoded = (file_id_u64 << Id::FILE_ID_SHIFT)
            | ((start_line as u64) << Id::START_LINE_SHIFT)
            | ((start_col as u64) << Id::START_COL_SHIFT)
            | ((end_line as u64) << Id::END_LINE_SHIFT)
            | ((end_col as u64) << Id::END_COL_SHIFT);

        Self(encoded, PhantomData)
    }

    /// Get the raw encoded value
    pub fn as_raw(&self) -> u64 {
        self.0
    }

    /// Extract the file ID component
    pub fn file_id(&self) -> Id {
        let id_value = (self.0 & Id::FILE_ID_MASK) >> Id::FILE_ID_SHIFT;
        // This should be safe since we encoded a valid Id originally
        id_value
            .try_into()
            .unwrap_or_else(|_| panic!("Invalid file ID encoding"))
    }
}

impl<Id: FileId> SourceFilePosition for AbsolutePosition<Id> {
    fn source_file_id(&self) -> Option<u16> {
        let id_value = (self.0 & Id::FILE_ID_MASK) >> Id::FILE_ID_SHIFT;
        Some(id_value as u16) // This might truncate for larger IDs, consider returning u64 instead
    }

    fn start_line(&self) -> u16 {
        ((self.0 >> Id::START_LINE_SHIFT) & Id::LINE_MASK) as u16
    }

    fn start_column(&self) -> u8 {
        ((self.0 >> Id::START_COL_SHIFT) & Id::COL_MASK) as u8
    }

    fn end_line(&self) -> u16 {
        ((self.0 >> Id::END_LINE_SHIFT) & Id::LINE_MASK) as u16
    }

    fn end_column(&self) -> u8 {
        ((self.0 >> Id::END_COL_SHIFT) & Id::COL_MASK) as u8
    }
}

/// Position relative to a file (file ID not included)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelativePosition(u64);

impl RelativePosition {
    const START_LINE_SHIFT: u32 = 32;
    const START_COL_SHIFT: u32 = 24;
    const END_LINE_SHIFT: u32 = 8;

    /// Create a new relative position
    pub fn new(start_line: u16, start_col: u8, end_line: u16, end_col: u8) -> Self {
        let encoded = ((start_line as u64) << Self::START_LINE_SHIFT)
            | ((start_col as u64) << Self::START_COL_SHIFT)
            | ((end_line as u64) << Self::END_LINE_SHIFT)
            | end_col as u64;

        Self(encoded)
    }

    /// Get the raw encoded value
    pub fn as_raw(&self) -> u64 {
        self.0
    }
}

impl SourceFilePosition for RelativePosition {
    fn source_file_id(&self) -> Option<u16> {
        None // Relative positions have no file ID
    }

    fn start_line(&self) -> u16 {
        ((self.0 >> Self::START_LINE_SHIFT) & 0xFFFF) as u16
    }

    fn start_column(&self) -> u8 {
        ((self.0 >> Self::START_COL_SHIFT) & 0xFF) as u8
    }

    fn end_line(&self) -> u16 {
        ((self.0 >> Self::END_LINE_SHIFT) & 0xFFFF) as u16
    }

    fn end_column(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }
}

/// Compact absolute position using u8 file IDs (supports up to 255 files)
pub type CompactAbsolutePosition = AbsolutePosition<u8>;

/// Standard absolute position using u16 file IDs (supports up to 65535 files)
pub type StandardAbsolutePosition = AbsolutePosition<u16>;
