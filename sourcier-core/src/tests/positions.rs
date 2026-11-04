//! Integration tests for packed positions through the public API.

#![cfg(feature = "view")]

use crate::{
    AbsolutePosition, CompactAbsolutePosition, FileId, RelativePosition, SourceFilePosition,
    StandardAbsolutePosition,
};

#[test]
fn absolute_position_round_trips_through_raw() {
    let pos = AbsolutePosition::<u8>::new(199, 65_535, 255, 1, 1);
    let raw = pos.as_raw();
    let rebuilt = AbsolutePosition::<u8>::from_raw(raw).expect("valid id");
    assert_eq!(pos, rebuilt);
    assert_eq!(rebuilt.file_id(), 199);
    assert_eq!(rebuilt.start_line(), 65_535);
    assert_eq!(rebuilt.start_column(), 255);
    assert_eq!(rebuilt.end_line(), 1);
    assert_eq!(rebuilt.end_column(), 1);
}

#[test]
fn from_raw_rejects_the_unassigned_zero_id() {
    let pos = StandardAbsolutePosition::new(300, 10, 5, 12, 20);
    assert!(StandardAbsolutePosition::from_raw(pos.as_raw()).is_some());

    // Zeroing the id field encodes the never-assigned id 0.
    let zeroed = pos.as_raw() & !u16::FILE_ID_MASK;
    assert!(StandardAbsolutePosition::from_raw(zeroed).is_none());
    assert!(CompactAbsolutePosition::from_raw(zeroed).is_none());
}

#[test]
fn from_raw_round_trips_across_the_full_id_range() {
    // Every non-zero id within the id field width decodes, including the
    // largest ids of both id types.
    let compact = AbsolutePosition::<u8>::new(255, 65_535, 255, 1, 1);
    assert_eq!(
        AbsolutePosition::<u8>::from_raw(compact.as_raw()),
        Some(compact)
    );
    let standard = StandardAbsolutePosition::new(65_535, 32_768, 200, 1, 3);
    assert_eq!(
        StandardAbsolutePosition::from_raw(standard.as_raw()),
        Some(standard)
    );
}

#[test]
fn u16_positions_keep_the_high_file_ids() {
    let pos = StandardAbsolutePosition::new(65_535, 32_768, 200, 1, 3);
    let rebuilt = StandardAbsolutePosition::from_raw(pos.as_raw()).expect("valid id");
    assert_eq!(rebuilt, pos);
    assert_eq!(rebuilt.file_id(), 65_535);
    assert_eq!(rebuilt.start_line(), 32_768);
}

#[test]
fn type_aliases_use_the_documented_id_widths() {
    let compact = CompactAbsolutePosition::new(255, 1, 1, 1, 1);
    assert_eq!(compact.file_id(), 255);
    assert_eq!(u8::MAX_FILES, 255);

    let standard = StandardAbsolutePosition::new(65_535, 1, 1, 1, 1);
    assert_eq!(standard.file_id(), 65_535);
    assert_eq!(u16::MAX_FILES, 65_535);
}

#[test]
fn positions_implement_display_and_debug() {
    let pos = AbsolutePosition::<u16>::new(12, 3, 4, 5, 6);
    assert_eq!(pos.to_string(), "3:4-5:6");
    let rendered = format!("{pos:?}");
    assert!(rendered.contains("file_id: 12"), "{rendered}");

    let rel = RelativePosition::new(1, 2, 3, 4);
    assert_eq!(rel.to_string(), "1:2-3:4");
}

#[test]
fn positions_are_hashable_and_ordered() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(AbsolutePosition::<u8>::new(1, 1, 1, 1, 1));
    set.insert(AbsolutePosition::<u8>::new(1, 1, 1, 1, 1));
    assert_eq!(set.len(), 1);

    let smaller = RelativePosition::new(1, 1, 1, 1);
    let larger = RelativePosition::new(1, 1, 1, 2);
    assert!(smaller < larger);
}

#[test]
fn source_file_position_is_object_safe_for_helpers() {
    fn summarize(pos: &impl SourceFilePosition) -> String {
        format!(
            "file={:?} {}:{}-{}:{}",
            pos.source_file_id(),
            pos.start_line(),
            pos.start_column(),
            pos.end_line(),
            pos.end_column()
        )
    }

    assert_eq!(
        summarize(&RelativePosition::new(2, 3, 4, 5)),
        "file=None 2:3-4:5"
    );
    assert_eq!(
        summarize(&AbsolutePosition::<u8>::new(9, 2, 3, 4, 5)),
        "file=Some(9) 2:3-4:5"
    );
}

#[test]
fn try_new_accepts_boundary_spans() {
    assert!(AbsolutePosition::<u8>::try_new(1, 1, 1, 65_535, 255).is_ok());
    assert!(RelativePosition::try_new(65_535, 255, 65_535, 255).is_ok());
    assert!(RelativePosition::try_new(1, 1, 0, 255).is_err());
}

#[test]
fn absolute_positions_strip_to_relative_positions() {
    // Stripping the file id keeps every span component untouched, whatever
    // the id width, so diagnostics can be stored file-agnostically.
    let compact = AbsolutePosition::<u8>::new(9, 12, 34, 56, 78);
    assert_eq!(compact.to_relative(), RelativePosition::new(12, 34, 56, 78));

    let standard = StandardAbsolutePosition::new(65_000, 12, 34, 56, 78);
    assert_eq!(
        standard.to_relative(),
        RelativePosition::new(12, 34, 56, 78)
    );

    // The `From` conversion is the same operation in trait form.
    let via_from: RelativePosition = compact.into();
    assert_eq!(via_from, RelativePosition::new(12, 34, 56, 78));
}

#[test]
fn relative_positions_attach_file_ids() {
    let rel = RelativePosition::new(3, 4, 5, 6);
    let abs = rel.to_absolute(42_u8);
    assert_eq!(abs.file_id(), 42);
    assert_eq!(abs.start_line(), 3);
    assert_eq!(abs.start_column(), 4);
    assert_eq!(abs.end_line(), 5);
    assert_eq!(abs.end_column(), 6);

    // Attaching and stripping round trip for both id widths.
    let round_trip = rel.to_absolute(65_000_u16).to_relative();
    assert_eq!(round_trip, rel);
}

#[test]
fn span_components_survive_the_relative_round_trip() {
    let abs = StandardAbsolutePosition::new(12, 65_535, 255, 1, 2);
    let rel = abs.to_relative();
    assert_eq!(rel.start_line(), 65_535);
    assert_eq!(rel.start_column(), 255);
    assert_eq!(rel.end_line(), 1);
    assert_eq!(rel.end_column(), 2);
    assert_eq!(rel.to_absolute(12_u16), abs);
}

#[test]
fn relative_positions_can_be_built_in_const_contexts() {
    // Packed positions are documented stable encodings, so building one in a
    // `const` item and reading its raw value must work.
    const SPAN: RelativePosition = RelativePosition::new(2, 5, 2, 22);
    const RAW: u64 = SPAN.as_raw();
    const REBUILT: RelativePosition = RelativePosition::from_raw(RAW);

    assert_eq!(RAW, RelativePosition::new(2, 5, 2, 22).as_raw());
    assert_eq!(SPAN.start_line(), 2);
    assert_eq!(REBUILT, SPAN);
}
