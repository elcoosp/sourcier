use crate::fid::{AbsolutePosition, FileId, RelativePosition, SourceFilePosition};

// Re-export the position types for convenience
pub use crate::fid::{CompactAbsolutePosition, StandardAbsolutePosition};

// Example usage functions
pub fn create_absolute_position<Id: FileId>(
    file_id: Id,
    start_line: u16,
    start_col: u8,
    end_line: u16,
    end_col: u8,
) -> AbsolutePosition<Id> {
    AbsolutePosition::new(file_id, start_line, start_col, end_line, end_col)
}

pub fn create_relative_position(
    start_line: u16,
    start_col: u8,
    end_line: u16,
    end_col: u8,
) -> RelativePosition {
    RelativePosition::new(start_line, start_col, end_line, end_col)
}

// Helper function to print position information
pub fn print_position_info<P: SourceFilePosition>(pos: &P) {
    println!("Source file ID: {:?}", pos.source_file_id());
    println!(
        "Start position: {}:{}",
        pos.start_line(),
        pos.start_column()
    );
    println!("End position: {}:{}", pos.end_line(), pos.end_column());
}
