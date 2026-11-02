//! Property-based tests with `proptest`.

use crate::{AbsolutePosition, RelativePosition, SourceFilePosition, SourceFilesMap};
use proptest::prelude::*;

/// Builds the duplicate-path used by the first-wins property test.
fn dup_path(slot: usize) -> String {
    format!("dup_file_{slot:02}.rs")
}

proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(512))]

    #[test]
    fn absolute_positions_round_trip(
        file_id in 1_u8..=255,
        start_line in 1_u16..=65_535,
        start_col in 1_u8..=255,
        end_line in 1_u16..=65_535,
        end_col in 1_u8..=255,
    ) {
        let pos = AbsolutePosition::new(file_id, start_line, start_col, end_line, end_col);
        prop_assert_eq!(pos.file_id(), file_id);
        prop_assert_eq!(pos.start_line(), start_line);
        prop_assert_eq!(pos.start_column(), start_col);
        prop_assert_eq!(pos.end_line(), end_line);
        prop_assert_eq!(pos.end_column(), end_col);

        let rebuilt = AbsolutePosition::from_raw(pos.as_raw()).expect("valid id");
        prop_assert_eq!(pos, rebuilt);
    }

    #[test]
    fn u16_positions_round_trip(
        file_id in 1_u16..=65_535,
        start_line in 1_u16..=65_535,
        start_col in 1_u8..=255,
        end_line in 1_u16..=65_535,
        end_col in 1_u8..=255,
    ) {
        let pos = AbsolutePosition::new(file_id, start_line, start_col, end_line, end_col);
        prop_assert_eq!(pos.file_id(), file_id);
        prop_assert_eq!(pos.start_line(), start_line);
        prop_assert_eq!(pos.start_column(), start_col);
        prop_assert_eq!(pos.end_line(), end_line);
        prop_assert_eq!(pos.end_column(), end_col);

        let rebuilt = AbsolutePosition::from_raw(pos.as_raw()).expect("valid id");
        prop_assert_eq!(pos, rebuilt);
    }

    #[test]
    fn relative_positions_round_trip(
        start_line in 1_u16..=65_535,
        start_col in 1_u8..=255,
        end_line in 1_u16..=65_535,
        end_col in 1_u8..=255,
    ) {
        let pos = RelativePosition::new(start_line, start_col, end_line, end_col);
        prop_assert_eq!(pos.start_line(), start_line);
        prop_assert_eq!(pos.start_column(), start_col);
        prop_assert_eq!(pos.end_line(), end_line);
        prop_assert_eq!(pos.end_column(), end_col);
        prop_assert_eq!(pos.source_file_id(), None);
    }

    #[test]
    fn map_lookups_stay_consistent(
        names in proptest::collection::vec("[a-z]{1,8}\\.rs", 1..30),
    ) {
        let mut files = SourceFilesMap::<u16>::new();
        for (index, name) in names.iter().enumerate() {
            files
                .add_file(name.as_str(), format!("content {index}"))
                .expect("u16 capacity");
        }
        files.finalize().expect("u16 capacity");

        prop_assert_eq!(
            files.len(),
            names.iter().collect::<std::collections::BTreeSet<_>>().len()
        );
        for (index, name) in names.iter().enumerate() {
            let id = files.get_id(name).expect("path survives dedup");
            let expected = if names[..index].contains(name) {
                // Duplicate path: the first registration wins.
                let first = names.iter().position(|n| n == name).expect("exists");
                format!("content {first}")
            } else {
                format!("content {index}")
            };
            prop_assert_eq!(files.get_content(id), Some(expected.as_bytes()));
            prop_assert_eq!(files.get_path(id), Some(name.as_str()));
        }
    }

    #[test]
    fn duplicate_paths_keep_the_first_registration_at_scale(
        // Hundreds of files with scattered duplicates stress the unstable
        // sort used during finalization: the first-registered content of a
        // duplicate path must survive regardless of sort internals.
        pattern in proptest::collection::vec(0_usize..24, 300..460),
    ) {
        let mut files = SourceFilesMap::<u16>::new();
        let mut first_content: Vec<Option<usize>> = vec![None; 24];
        for (index, slot) in pattern.iter().enumerate() {
            if first_content[*slot].is_none() {
                first_content[*slot] = Some(index);
            }
            files
                .add_file(dup_path(*slot), format!("content {index}"))
                .expect("u16 capacity");
        }
        files.finalize().expect("u16 capacity");

        prop_assert_eq!(files.len(), first_content.iter().flatten().count());
        for (slot, first) in first_content.iter().enumerate() {
            let first = first.expect("slot registered");
            let path = dup_path(slot);
            let expected = format!("content {first}");
            let id = files.get_id(&path).expect("path survives dedup");
            prop_assert_eq!(
                files.get_content(id),
                Some(expected.as_bytes()),
                "first registration wins for slot {}",
                slot
            );
        }
    }
}

#[cfg(feature = "view")]
mod view_properties {
    use crate::LineOffsets;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(512))]

        #[test]
        fn line_col_and_offset_are_inverse(
            content in proptest::collection::vec(any::<u8>(), 0..300),
            offset in 0_usize..301,
        ) {
            let lines = LineOffsets::compute(&content).expect("small content");
            if offset > content.len() {
                prop_assert_eq!(lines.line_col_from_offset(offset), None);
            } else {
                let (line, col) = lines
                    .line_col_from_offset(offset)
                    .expect("offset in range maps");
                prop_assert!(line >= 1 && line <= lines.line_count());
                prop_assert!(col >= 1);
                prop_assert_eq!(lines.offset_from_line_col(line, col), Some(offset));
            }
        }
    }
}
