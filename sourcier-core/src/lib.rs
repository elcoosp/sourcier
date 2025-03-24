// Public modules
pub mod fid;
pub mod sfm;
pub mod sfp;
// Re-export commonly used types for convenience
pub use fid::{
    AbsolutePosition, CompactAbsolutePosition, FileId, RelativePosition, SourceFilePosition,
    StandardAbsolutePosition,
};
pub use sfm::SourceFilesMap;
pub use sfp::{create_absolute_position, create_relative_position, print_position_info};

// Example usage to show the integration
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrated_usage() -> Result<(), String> {
        // Create a file map
        let mut files = SourceFilesMap::<u8>::new();

        // Add files
        files.add_file("src/sfp.rs".into(), include_bytes!("sfp.rs").to_vec());
        let abs_file_id = "src/fid.rs";
        files.add_file(abs_file_id.into(), include_bytes!("fid.rs").to_vec());
        // Finalize to assign IDs
        files.finalize()?;

        // Get a file ID
        let file_id = files.get_id(abs_file_id).unwrap();

        // Create an absolute position
        let abs_pos = create_absolute_position(file_id, 10, 5, 12, 20);

        // Create a relative position
        let rel_pos = create_relative_position(10, 5, 12, 20);

        // Verify both positions have the same line/column values
        assert_eq!(abs_pos.start_line(), rel_pos.start_line());
        assert_eq!(abs_pos.start_column(), rel_pos.start_column());
        assert_eq!(abs_pos.end_line(), rel_pos.end_line());
        assert_eq!(abs_pos.end_column(), rel_pos.end_column());

        // But different file IDs
        assert!(abs_pos.source_file_id().is_some());
        assert!(rel_pos.source_file_id().is_none());

        Ok(())
    }
}
