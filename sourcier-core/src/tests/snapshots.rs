//! Snapshot tests backed by [`insta`].
//!
//! Snapshots are part of the repository (`src/snapshots/`) and act as
//! reviewable contracts for the serialized forms, packed encodings, rendered
//! error messages and `view` output of the public API. After an intentional
//! change, regenerate them with `cargo insta review` or
//! `INSTA_UPDATE=always cargo test`.

#![cfg(feature = "serde")]

use crate::{AbsolutePosition, MapError, PositionError, RelativePosition, SourceFilesMap};
use insta::assert_yaml_snapshot;

/// Builds a finalized map with three files registered out of path order.
fn sample_map() -> SourceFilesMap<u8> {
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_file("src/main.rs", b"fn main() {}")
        .expect("fits");
    files
        .add_file("src/lib.rs", b"pub mod utils;")
        .expect("fits");
    files.add_file("README.md", b"# sourcier").expect("fits");
    files.finalize().expect("fits");
    files
}

#[test]
fn finalized_map_serializes_as_reviewable_yaml() {
    let files = sample_map();
    // `path_to_id` is a `HashMap` whose iteration order is not stable, so it
    // is redacted to the sorted list that insta renders deterministically.
    assert_yaml_snapshot!(files, { ".path_to_id" => insta::sorted_redaction() });
}

#[test]
fn build_phase_map_serializes_without_lookup_state() {
    let mut files = SourceFilesMap::<u16>::new();
    files
        .add_file("pending.rs", b"not finalized yet")
        .expect("fits");
    assert_yaml_snapshot!(files, { ".path_to_id" => insta::sorted_redaction() });
}

#[test]
fn packed_positions_render_their_raw_encoding() {
    let compact = AbsolutePosition::<u8>::new(7, 3, 4, 5, 6);
    let standard = AbsolutePosition::<u16>::new(65_535, 65_535, 255, 1, 1);
    let relative = RelativePosition::new(3, 4, 5, 6);
    assert_yaml_snapshot!([
        ("compact_absolute", compact.as_raw()),
        ("standard_absolute", standard.as_raw()),
        ("relative", relative.as_raw()),
    ]);
}

#[test]
fn error_messages_are_part_of_the_api_surface() {
    insta::assert_snapshot!(MapError::TooManyFiles { max: 255 }.to_string());
    insta::assert_snapshot!(
        MapError::FileTooLarge {
            size: 4_294_967_296
        }
        .to_string()
    );
    insta::assert_snapshot!(
        PositionError::InvalidRange {
            start_line: 2,
            start_col: 1,
            end_line: 1,
            end_col: 9
        }
        .to_string()
    );
}

#[cfg(feature = "view")]
#[test]
fn view_extraction_renders_the_expected_bytes() {
    let files = sample_map();
    let id = files.get_id("src/main.rs").expect("present");
    let pos = AbsolutePosition::new(id, 1, 1, 1, 2);
    let view = files.view(id, &pos).expect("span in bounds");
    insta::assert_snapshot!(core::str::from_utf8(view).expect("utf-8 content"));
}
