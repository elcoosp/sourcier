//! Integration tests for the opt-in `parallel` feature.
//!
//! The parallel line-offset build must produce byte-identical indexes to the
//! documented serial semantics, with a deterministic layout.

#![cfg(all(feature = "view", feature = "parallel"))]

use crate::{AbsolutePosition, LineOffsets, SourceFilesMap};

#[test]
fn parallel_finalize_matches_the_serial_semantics() {
    let contents: Vec<Vec<u8>> = (0..64)
        .map(|file| {
            (0..200)
                .map(|line| format!("fn item_{file}_{line}() {{ /* body */ }}\n").into_bytes())
                .fold(Vec::new(), |mut acc, mut line| {
                    acc.append(&mut line);
                    acc
                })
        })
        .collect();

    let mut files = SourceFilesMap::<u16>::new();
    for (index, content) in contents.iter().enumerate() {
        files
            .add_file(format!("src/m{index:03}.rs"), content.as_slice())
            .expect("fits");
    }
    files.finalize().expect("fits");

    for (index, content) in contents.iter().enumerate() {
        let id = files
            .get_id(&format!("src/m{index:03}.rs"))
            .expect("indexed");
        let expected = LineOffsets::compute(content).expect("fits");
        assert_eq!(files.line_offsets(id), Some(&expected));
        assert_eq!(files.get_content(id), Some(content.as_slice()));
    }

    // Views keep working through the parallel build: extract lines 10-12.
    let id = files.get_id("src/m000.rs").expect("indexed");
    let content = &contents[0];
    let offsets = files.line_offsets(id).expect("indexed");
    let line12_len = {
        let range = offsets.line_range(12).expect("line exists");
        range.end - range.start
    };
    let start = offsets.offset_from_line_col(10, 1).expect("in range");
    let start12 = offsets.offset_from_line_col(12, 1).expect("in range");
    let end = start12 + line12_len; // exclusive: covers line 12 content
    let pos = AbsolutePosition::new(id, 10, 1, 12, line12_len as u8);
    let view = files.view(id, &pos).expect("valid span");
    assert_eq!(view, &content[start..end]);
    assert!(view.starts_with(b"fn item_0_9()"));
    assert!(view.ends_with(b"}"));
}
