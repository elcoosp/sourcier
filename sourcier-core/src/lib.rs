mod tests;
// Public modules
pub mod fid;
pub mod sfm;
pub mod sfp;
// Re-export commonly used types for convenience
pub use fid::{
    AbsolutePosition, CompactAbsolutePosition, FileId, RelativePosition, SourceFilePosition,
    StandardAbsolutePosition,
};
#[cfg(feature = "rt-feedback")]
pub use sfm::RuntimeFeedback;
pub use sfm::SourceFilesMap;
pub use sfp::{create_absolute_position, create_relative_position, print_position_info};
