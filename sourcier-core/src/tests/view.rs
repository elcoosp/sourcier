//! Integration tests for the `view` feature: byte extraction by position and
//! line offset navigation.

#![cfg(feature = "view")]

use crate::{AbsolutePosition, LineOffsets, RelativePosition, SourceFilesMap};

fn sample_map() -> SourceFilesMap<u8> {
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_file(
            "code.rs",
            b"fn main() {\n    let x = 1;\n    println!(x);\n}\n",
        )
        .expect("fits");
    files
        .add_file("text.txt", b"alpha\nbeta\ngamma")
        .expect("fits");
    files.add_file("empty.txt", b"").expect("fits");
    files.finalize().expect("fits");
    files
}

#[test]
fn view_extracts_single_line_spans() {
    let files = sample_map();
    let id = files.get_id("code.rs").expect("present");

    let pos = AbsolutePosition::new(id, 1, 1, 1, 10);
    assert_eq!(files.view(id, &pos), Some(&b"fn main() "[..]));

    let pos = AbsolutePosition::new(id, 2, 5, 2, 14);
    assert_eq!(files.view(id, &pos), Some(&b"let x = 1;"[..]));

    // A column one past the last content byte includes the line terminator.
    let pos = AbsolutePosition::new(id, 2, 5, 2, 15);
    assert_eq!(files.view(id, &pos), Some(&b"let x = 1;\n"[..]));
}

#[test]
fn view_extracts_multi_line_spans() {
    let files = sample_map();
    let id = files.get_id("code.rs").expect("present");

    let pos = AbsolutePosition::new(id, 1, 1, 4, 1);
    assert_eq!(
        files.view(id, &pos),
        Some(&b"fn main() {\n    let x = 1;\n    println!(x);\n}"[..])
    );
}

#[test]
fn view_str_validates_utf8() {
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_file("utf8.txt", b"h\xc3\xa9llo\nw\xc3\xb6rld")
        .expect("fits");
    files.add_file("binary.bin", b"\xff\xfe\x00").expect("fits");
    files.finalize().expect("fits");

    let utf8_id = files.get_id("utf8.txt").expect("present");
    // Columns are byte columns: 6 bytes cover "wörld".
    let pos = AbsolutePosition::new(utf8_id, 1, 1, 2, 6);
    assert_eq!(files.view_str(utf8_id, &pos), Some("héllo\nwörld"));

    let binary_id = files.get_id("binary.bin").expect("present");
    let pos = AbsolutePosition::new(binary_id, 1, 1, 1, 3);
    assert_eq!(files.view(binary_id, &pos), Some(&b"\xff\xfe\x00"[..]));
    assert_eq!(files.view_str(binary_id, &pos), None);
}

#[test]
fn view_rejects_out_of_bounds_positions() {
    let files = sample_map();
    let id = files.get_id("text.txt").expect("present");

    // Line 0 is invalid.
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 0, 1, 1, 1)), None);
    // Past the last line.
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 4, 1, 4, 2)), None);
    // Column 0 is invalid (positions are one-based), for both the start
    // and the end column.
    assert_eq!(
        files
            .view(id, &RelativePosition::new(1, 0, 1, 1))
            .map(|_| ()),
        None
    );
    assert_eq!(
        files
            .view(id, &RelativePosition::new(1, 1, 1, 0))
            .map(|_| ()),
        None
    );
    // Start column past the end column on the same line.
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 3, 4, 3, 2)), None);
    // Start line after end line.
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 3, 1, 2, 1)), None);
    // Start column beyond the line terminator while the end stays valid:
    // the start byte must stay within its own line.
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 1, 7, 2, 4)), None);
    // Column beyond the line terminator.
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 3, 1, 3, 7)), None);
    // Unknown file id.
    let unknown = 99_u8;
    assert_eq!(
        files.view(unknown, &AbsolutePosition::new(unknown, 1, 1, 1, 1)),
        None
    );
}

#[test]
fn view_works_with_relative_positions() {
    let files = sample_map();
    let id = files.get_id("text.txt").expect("present");
    let pos = RelativePosition::new(1, 1, 2, 4);
    assert_eq!(files.view(id, &pos), Some(&b"alpha\nbeta"[..]));
}

#[test]
fn view_supports_u16_file_ids() {
    let mut files = SourceFilesMap::<u16>::new();
    files
        .add_file("wide.txt", b"alpha\nbeta\ngamma")
        .expect("fits");
    files.finalize().expect("fits");

    let id = files.get_id("wide.txt").expect("present");
    let pos = AbsolutePosition::new(id, 2, 1, 3, 5);
    assert_eq!(files.view(id, &pos), Some(&b"beta\ngamma"[..]));
    assert_eq!(files.view_str(id, &pos), Some("beta\ngamma"));

    // Unknown ids and out-of-bounds lines fail the same way as for u8.
    let unknown = 9_999_u16;
    assert_eq!(
        files.view(unknown, &AbsolutePosition::new(unknown, 1, 1, 1, 1)),
        None
    );
    assert_eq!(files.view(id, &AbsolutePosition::new(id, 9, 1, 9, 1)), None);
    assert_eq!(
        files
            .view_str(unknown, &AbsolutePosition::new(unknown, 1, 1, 1, 1))
            .map(|_| ()),
        None
    );
}

#[test]
fn view_covers_relative_positions_against_u16_maps() {
    // Views on an unfinalized map fail at the content lookup.
    let mut files = SourceFilesMap::<u16>::new();
    files.add_file("w.txt", b"alpha\nbeta").expect("fits");
    let pos = RelativePosition::new(1, 1, 1, 5);
    assert_eq!(files.view(1, &pos), None);

    files.finalize().expect("fits");
    let id = files.get_id("w.txt").expect("present");
    assert_eq!(files.view(id, &pos), Some(&b"alpha"[..]));
    assert_eq!(files.view_str(id, &pos), Some("alpha"));

    // A relative position with an invalid end line resolves to nothing.
    let bad = RelativePosition::new(1, 1, 9, 1);
    assert_eq!(files.view(id, &bad), None);
    assert_eq!(files.view_str(id, &bad), None);
}

#[test]
fn view_fails_at_the_content_lookup_for_u8_relative_positions() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("a.txt", b"alpha").expect("fits");
    let pos = RelativePosition::new(1, 1, 1, 1);

    // Unfinalized maps hold no content to look up.
    assert_eq!(files.view(1, &pos), None);

    files.finalize().expect("fits");
    // Unknown ids fail before any line lookup.
    assert_eq!(files.view(99, &pos), None);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn files_beyond_the_offset_limit_are_not_indexed() {
    // Large zeroed allocations are mmap-backed: untouched pages are never
    // committed to physical memory, so the 4 GiB limit can be exercised
    // here without committing 4 GiB of memory.
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_file("huge.bin", vec![0_u8; LineOffsets::MAX_CONTENT_LENGTH + 1])
        .expect("fits");
    files.add_file("small.txt", b"alpha\nbeta").expect("fits");
    files
        .finalize()
        .expect("oversized files finalize without an index");

    let huge = files.get_id("huge.bin").expect("present");
    let small = files.get_id("small.txt").expect("present");

    // The oversized file has no line index, so views resolve to nothing.
    assert_eq!(
        files.view(huge, &AbsolutePosition::new(huge, 1, 1, 1, 1)),
        None
    );
    assert_eq!(
        files
            .view_str(huge, &AbsolutePosition::new(huge, 1, 1, 1, 1))
            .map(|_| ()),
        None
    );

    // Small files in the same map are unaffected.
    let pos = AbsolutePosition::new(small, 2, 1, 2, 4);
    assert_eq!(files.view_str(small, &pos), Some("beta"));
}

#[test]
fn view_handles_empty_files_and_full_line_endpoints() {
    let files = sample_map();

    let empty_id = files.get_id("empty.txt").expect("present");
    // A valid zero-width position in an empty file yields an empty view.
    assert_eq!(
        files.view(empty_id, &AbsolutePosition::new(empty_id, 1, 1, 1, 1)),
        Some(&b""[..])
    );

    // A position covering exactly the last byte of the last line.
    let id = files.get_id("text.txt").expect("present");
    let pos = AbsolutePosition::new(id, 3, 5, 3, 5);
    assert_eq!(files.view(id, &pos), Some(&b"a"[..]));

    // A span that runs into the one-past-EOF column is clamped to the file
    // length ("gamma" has 5 bytes; column 6 would be one past the end).
    let pos = AbsolutePosition::new(id, 3, 1, 3, 6);
    assert_eq!(files.view(id, &pos), Some(&b"gamma"[..]));

    // A zero-width position at the end of the last line is still valid.
    let pos = AbsolutePosition::new(id, 3, 6, 3, 6);
    assert_eq!(files.view(id, &pos), Some(&b""[..]));

    // The end column pointing one past the line content includes the
    // terminator.
    let pos = AbsolutePosition::new(id, 1, 1, 1, 6);
    assert_eq!(files.view(id, &pos), Some(&b"alpha\n"[..]));
}

#[test]
fn line_offsets_answer_line_col_queries() {
    let lines = LineOffsets::compute(b"one\ntwo\nthree").expect("fits");
    assert_eq!(lines.line_count(), 3);
    assert_eq!(lines.content_length(), 13);
    assert_eq!(lines.line_range(1), Some(0..3));
    assert_eq!(lines.line_range(3), Some(8..13));
    assert_eq!(lines.line_col_from_offset(8), Some((3, 1)));
    assert_eq!(lines.line_col_from_offset(13), Some((3, 6)));
    assert_eq!(lines.line_col_from_offset(14), None);
    assert_eq!(lines.offset_from_line_col(2, 2), Some(5));
    assert_eq!(lines.offset_from_line_col(2, 0), None);
    assert_eq!(lines.offset_from_line_col(4, 1), None);
    // A column beyond the end of its line resolves to nothing (the last
    // line is the exception: it allows one past the final byte).
    assert_eq!(lines.offset_from_line_col(1, 5), None);
    assert_eq!(lines.offset_from_line_col(2, 5), None);
}

#[test]
#[cfg(feature = "serde")]
fn line_offsets_serialize_round_trip() {
    let lines = LineOffsets::compute(b"a\nbc\ndef").expect("fits");
    let json = serde_json::to_string(&lines).expect("serializable");
    let rebuilt: LineOffsets = serde_json::from_str(&json).expect("deserializable");
    assert_eq!(lines, rebuilt);
}
