//! Integration tests for the offset ⇄ position bridge and diagnostic
//! rendering: converting byte offsets to positions and back, and rendering
//! `path:line:col` strings without allocations.

use crate::{AbsolutePosition, RelativePosition, SourceFilePosition, SourceFilesMap};

fn sample_map() -> SourceFilesMap<u16> {
    let mut files = SourceFilesMap::<u16>::new();
    files
        .add_file(
            "code.rs",
            b"fn main() {\n    let x = 1;\n    println!(x);\n}\n",
        )
        .expect("fits");
    files
        .add_file("text.txt", b"alpha\nbeta\ngamma")
        .expect("fits");
    files.finalize().expect("fits");
    files
}

#[test]
fn offset_to_position_maps_to_one_based_line_col() {
    let files = sample_map();
    let id = files.get_id("text.txt").expect("present");

    // "alpha\nbeta\ngamma": offsets 0-4 line 1, 6-9 line 2, 11-15 line 3.
    assert_eq!(
        files.position_from_offset(id, 0),
        Some(AbsolutePosition::new(id, 1, 1, 1, 1))
    );
    assert_eq!(
        files.position_from_offset(id, 3),
        Some(AbsolutePosition::new(id, 1, 4, 1, 4))
    );
    assert_eq!(
        files.position_from_offset(id, 6),
        Some(AbsolutePosition::new(id, 2, 1, 2, 1))
    );
    assert_eq!(
        files.position_from_offset(id, 15),
        Some(AbsolutePosition::new(id, 3, 5, 3, 5))
    );

    // The produced positions render the exact bytes at the offset.
    let pos = files.position_from_offset(id, 6).expect("in range");
    assert_eq!(files.view_str(id, &pos), Some("b"));
}

#[test]
fn offset_to_position_covers_the_newline_and_eof_anchors() {
    let files = sample_map();
    let id = files.get_id("text.txt").expect("present");

    // The newline byte itself maps to the column one past the line end.
    let (line, col) = files
        .line_offsets(id)
        .expect("indexed")
        .line_col_from_offset(5)
        .expect("in range");
    let pos = files.position_from_offset(id, 5).expect("in range");
    assert_eq!(pos.start_line() as usize, line);
    assert_eq!(pos.start_column() as usize, col);

    // One past the last byte is a valid insertion anchor at EOF.
    let eof = files.position_from_offset(id, 16).expect("in range");
    assert_eq!(eof.start_line(), 3);
    assert_eq!(eof.start_column(), 6);

    // Anything beyond the content length is out of range.
    assert_eq!(files.position_from_offset(id, 17), None);
    assert_eq!(files.position_from_offset(id, usize::MAX), None);
}

#[test]
fn position_to_offset_uses_the_span_anchor() {
    let files = sample_map();
    let id = files.get_id("text.txt").expect("present");

    // The start of the span is the anchor; the end is ignored.
    let span = AbsolutePosition::new(id, 2, 1, 3, 4);
    assert_eq!(files.offset_from_position(id, &span), Some(6));

    // Relative positions work through the same uniform accessor.
    let rel = RelativePosition::new(3, 2, 3, 2);
    assert_eq!(files.offset_from_position(id, &rel), Some(12));

    // Unknown lines and columns resolve to nothing.
    assert_eq!(
        files.offset_from_position(id, &AbsolutePosition::new(id, 9, 1, 9, 1)),
        None
    );
    assert_eq!(
        files.offset_from_position(id, &RelativePosition::new(1, 99, 1, 99)),
        None
    );
}

#[test]
fn offset_and_position_conversions_are_inverse() {
    let files = sample_map();
    let id = files.get_id("text.txt").expect("present");
    let content_len = files.get_content(id).expect("present").len();

    for offset in 0..=content_len {
        let pos = files
            .position_from_offset(id, offset)
            .unwrap_or_else(|| panic!("offset {offset} in range"));
        assert_eq!(
            files.offset_from_position(id, &pos),
            Some(offset),
            "round trip failed at offset {offset}"
        );
    }
}

#[test]
fn offset_bridge_rejects_unknown_ids_and_unfinalized_maps() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("a.txt", b"one\ntwo").expect("fits");
    assert_eq!(files.position_from_offset(1, 0), None);
    assert_eq!(
        files.offset_from_position(1, &RelativePosition::new(1, 1, 1, 1)),
        None
    );
    files.finalize().expect("fits");
    assert_eq!(files.position_from_offset(9, 0), None);
    assert_eq!(
        files.offset_from_position(9, &RelativePosition::new(1, 1, 1, 1)),
        None
    );
}

#[test]
fn positions_render_as_path_line_col() {
    let files = sample_map();
    let id = files.get_id("code.rs").expect("present");
    let pos = AbsolutePosition::new(id, 2, 5, 2, 14);

    let rendered = files
        .display_position(id, &pos)
        .expect("known id")
        .to_string();
    assert_eq!(rendered, "code.rs:2:5");

    // Relative positions render through the same accessor.
    let rel = RelativePosition::new(1, 1, 1, 2);
    assert_eq!(
        files
            .display_position(id, &rel)
            .expect("known id")
            .to_string(),
        "code.rs:1:1"
    );
}

#[test]
fn display_position_requires_a_known_file() {
    let files = sample_map();
    let rel = RelativePosition::new(1, 1, 1, 1);
    assert_eq!(files.display_position(99, &rel), None);

    let mut unfinalized = SourceFilesMap::<u8>::new();
    unfinalized.add_file("a.txt", b"x").expect("fits");
    assert_eq!(
        unfinalized.display_position(1, &RelativePosition::new(1, 1, 1, 1)),
        None
    );
}

#[test]
fn display_position_borrows_without_copying() {
    // The returned value only borrows the map: rendering happens lazily in
    // `Display`, so building the formatter performs no allocation.
    use core::fmt::Write as _;

    let files = sample_map();
    let id = files.get_id("code.rs").expect("present");
    let display = files
        .display_position(id, &AbsolutePosition::new(id, 1, 1, 1, 2))
        .expect("known id");
    let mut buffer = String::new();
    write!(buffer, "{display}").expect("infallible write");
    assert_eq!(buffer, "code.rs:1:1");
}

#[test]
fn maps_iterate_by_reference() {
    let files = sample_map();

    let mut seen = Vec::new();
    for entry in &files {
        seen.push((entry.path.to_owned(), entry.id));
    }
    assert_eq!(seen.len(), 2);
    // Sorted path order: code.rs before text.txt.
    assert_eq!(seen[0].0, "code.rs");
    assert_eq!(seen[1].0, "text.txt");

    // The iterator yields the same items as the `iter` method.
    assert_eq!(files.iter().count(), (&files).into_iter().count());
}
