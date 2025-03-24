#[cfg(not(feature = "rt-feedback"))]
// Example usage to show the simple integration
#[cfg(test)]
mod basic {
    use crate::*;

    // Macro to simplify file addition with optional content
    macro_rules! add_file {
        ($files:expr, $path:expr) => {
            $files.add_file($path.to_string(), Vec::new())
        };
        ($files:expr, $path:expr, $content:expr) => {
            $files.add_file($path.to_string(), $content.to_vec())
        };
    }

    #[test]
    fn integrated_usage() -> Result<(), String> {
        // Create a file map
        let mut files = SourceFilesMap::<u8>::new();

        // Add files using the new macro
        add_file!(files, "src/sfp.rs", include_bytes!("sfp.rs"));
        let abs_file_id = "src/fid.rs";
        add_file!(files, abs_file_id, include_bytes!("fid.rs"));

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

#[cfg(feature = "rt-feedback")]
#[cfg(test)]
mod rt_feedback {
    use crate::*;
    use std::sync::{Arc, Mutex};

    // Macro to simplify file addition with optional content
    macro_rules! add_file {
        ($files:expr, $path:expr) => {
            $files.add_file($path.to_string(), Vec::new())
        };
        ($files:expr, $path:expr, $content:expr) => {
            $files.add_file($path.to_string(), $content.to_vec())
        };
    }

    // Helper function to create a runtime feedback context
    fn create_feedback_context() -> Arc<Mutex<RuntimeFeedback>> {
        Arc::new(Mutex::new(RuntimeFeedback::default()))
    }

    #[test]
    fn feedback_file_tracking() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create SourceFilesMap with feedback context
        let mut files_map = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));

        // Add some files using the new macro
        add_file!(files_map, "src/main.rs");
        add_file!(files_map, "src/lib.rs");
        add_file!(files_map, "tests/integration.rs");

        // Finalize the map to trigger feedback tracking
        files_map.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 3);
        assert_eq!(feedback_data.usage_count, 1);

        Ok(())
    }

    #[test]
    fn feedback_multiple_finalizations() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create multiple file maps with the same feedback context
        let mut files_map1 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map1, "project1/src/main.rs");
        files_map1.finalize()?;

        let mut files_map2 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map2, "project2/src/lib.rs");
        add_file!(files_map2, "project2/src/utils.rs");
        files_map2.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 2); // Second finalization overwrites first
        assert_eq!(feedback_data.usage_count, 2);

        Ok(())
    }

    #[test]
    fn feedback_file_size_tracking() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create SourceFilesMap with feedback context
        let mut files_map = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));

        // Simulate files with different sizes
        add_file!(files_map, "small.rs");
        add_file!(files_map, "medium.rs");
        add_file!(files_map, "large.rs");

        // Finalize the map
        files_map.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 3);

        Ok(())
    }

    #[test]
    fn feedback_context_sharing() -> Result<(), String> {
        let feedback = create_feedback_context();

        // Create multiple file maps sharing the same feedback context
        let mut files_map1 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map1, "project1/main.rs");
        files_map1.finalize()?;

        let mut files_map2 = SourceFilesMap::<u8>::with_feedback(Some(feedback.clone()));
        add_file!(files_map2, "project2/lib.rs");
        files_map2.finalize()?;

        // Check feedback state
        let feedback_data = feedback.lock().unwrap();
        assert_eq!(feedback_data.total_files, 1); // Only most recent finalization counts
        assert_eq!(feedback_data.usage_count, 2);

        Ok(())
    }
}
