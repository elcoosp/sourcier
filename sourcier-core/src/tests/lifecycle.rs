//! Integration tests for the full build, finalize and lookup lifecycle.

use crate::{MapError, SourceFilesMap};

#[test]
fn build_finalize_lookup_round_trip() {
    let mut files = SourceFilesMap::<u16>::new();
    files
        .add_file("src/lib.rs", b"pub mod a;\npub mod b;")
        .expect("fits");
    files
        .add_file("src/main.rs", b"fn main() {}\n")
        .expect("fits");
    files.add_file("README.md", "").expect("fits");
    files.finalize().expect("fits");

    assert_eq!(files.len(), 3);
    assert!(!files.is_empty());
    assert!(files.is_finalized());
    // 21 bytes (lib) + 13 bytes (main) + 0 bytes (README).
    assert_eq!(files.total_bytes(), 34);

    let id_lib = files.get_id("src/lib.rs").expect("present");
    let id_main = files.get_id("src/main.rs").expect("present");
    let id_readme = files.get_id("README.md").expect("present");

    // Sorted path order: README.md < src/lib.rs < src/main.rs.
    assert_eq!(u64::from(id_readme), 1);
    assert_eq!(u64::from(id_lib), 2);
    assert_eq!(u64::from(id_main), 3);

    assert_eq!(files.get_path(id_lib), Some("src/lib.rs"));
    assert_eq!(
        files.get_content(id_lib),
        Some(&b"pub mod a;\npub mod b;"[..])
    );
    assert_eq!(files.get_content(id_readme), Some(&b""[..]));

    let entry = files.get(id_main).expect("present");
    assert_eq!(entry.path, "src/main.rs");
    assert_eq!(entry.content, b"fn main() {}\n");
    assert_eq!(u64::from(entry.id), 3);
}

#[test]
fn empty_map_finalizes_to_an_empty_lookup_phase() {
    let mut files = SourceFilesMap::<u8>::new();
    assert!(files.is_empty());
    files.finalize().expect("no files");
    assert!(files.is_finalized());
    assert_eq!(files.len(), 0);
    assert_eq!(files.get_id("anything"), None);
    assert_eq!(files.iter().count(), 0);
    assert_eq!(files.paths().next(), None);
}

#[test]
fn unknown_paths_and_ids_return_none() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("known.txt", b"x").expect("fits");
    files.finalize().expect("fits");
    assert_eq!(files.get_id("unknown.txt"), None);
    assert_eq!(files.get_path(99), None);
    assert_eq!(files.get_content(99), None);
    assert_eq!(files.get(99), None);
    // Id 0 is never assigned.
    assert_eq!(files.get_path(0), None);
}

#[test]
fn capacity_limits_are_reported_as_errors() {
    let mut files = SourceFilesMap::<u8>::new();
    for i in 0..255 {
        files
            .add_file(format!("file_{i:03}.rs"), b"")
            .expect("fits");
    }
    assert_eq!(
        files.add_file("too_many.rs", b""),
        Err(MapError::TooManyFiles { max: 255 })
    );
    files.finalize().expect("exactly at capacity");
    assert_eq!(files.len(), 255);
    assert!(files.get_id("file_254.rs").is_some());

    let mut files = SourceFilesMap::<u16>::new();
    for i in 0..1_000 {
        files
            .add_file(format!("file_{i:04}.rs"), b"")
            .expect("fits");
    }
    files.finalize().expect("fits");
    assert_eq!(files.len(), 1_000);
    assert!(files.get_id("file_0999.rs").is_some());
}

#[test]
fn duplicate_paths_keep_first_content_and_single_id() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("dup.txt", b"first").expect("fits");
    files.add_file("dup.txt", b"second").expect("fits");
    files.add_file("other.txt", b"third").expect("fits");
    files.finalize().expect("fits");

    assert_eq!(files.len(), 2);
    let id = files.get_id("dup.txt").expect("present");
    assert_eq!(files.get_content(id), Some(&b"first"[..]));
    let paths: Vec<_> = files.paths().collect();
    assert_eq!(paths, vec!["dup.txt", "other.txt"]);
}

#[test]
fn mutation_is_rejected_after_finalization() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("a.txt", b"a").expect("fits");
    files.finalize().expect("fits");

    assert_eq!(files.add_file("b.txt", b"b"), Err(MapError::Finalized));
    assert_eq!(files.finalize(), Err(MapError::AlreadyFinalized));
    assert_eq!(files.len(), 1, "failed mutations must not change the map");
}

#[test]
fn with_capacity_preallocates_and_behaves_identically() {
    let mut files = SourceFilesMap::<u16>::with_capacity(10);
    for i in 0..10 {
        files
            .add_file(format!("f{i}.rs"), format!("content {i}"))
            .expect("fits");
    }
    files.finalize().expect("fits");
    assert_eq!(files.len(), 10);
    for i in 0..10_u64 {
        let id = u16::try_from(i + 1).expect("small");
        let expected = format!("f{i}.rs");
        assert_eq!(files.get_path(id), Some(expected.as_str()));
        assert_eq!(
            files.get_content(id),
            Some(format!("content {i}").as_bytes())
        );
    }
}

#[test]
fn iteration_matches_direct_lookups() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("one.txt", b"111").expect("fits");
    files.add_file("two.txt", b"2222").expect("fits");
    files.finalize().expect("fits");

    // The map is iterable by reference, like any standard collection.
    for entry in &files {
        assert_eq!(files.get_path(entry.id), Some(entry.path));
        assert_eq!(files.get_content(entry.id), Some(entry.content));
        assert_eq!(files.get(entry.id).expect("present"), entry);
    }
    assert_eq!(files.iter().count(), 2);
}

#[test]
fn paths_iterate_in_insertion_order_before_finalization() {
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("z.txt", b"").expect("fits");
    files.add_file("a.txt", b"").expect("fits");
    assert_eq!(files.paths().collect::<Vec<_>>(), vec!["z.txt", "a.txt"]);
    files.finalize().expect("fits");
    assert_eq!(files.paths().collect::<Vec<_>>(), vec!["a.txt", "z.txt"]);
}

#[test]
fn maps_support_default_and_clone_like_values() {
    let mut files = SourceFilesMap::<u16>::default();
    assert!(files.is_empty());
    files.add_file("a.rs", b"alpha").expect("fits");
    files.finalize().expect("fits");

    let clone = files.clone();
    let id = clone.get_id("a.rs").expect("cloned lookup state");
    assert_eq!(clone.get_path(id), Some("a.rs"));
    assert_eq!(clone.get_content(id), Some(&b"alpha"[..]));
    assert_eq!(clone.total_bytes(), files.total_bytes());
}

#[test]
fn add_path_accepts_path_types() {
    use std::path::{Path, PathBuf};

    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_path(Path::new("src/lib.rs"), b"pub mod a;")
        .expect("utf-8 path");
    files
        .add_path(PathBuf::from("src/main.rs"), b"fn main() {}")
        .expect("utf-8 path");
    files.finalize().expect("fits");

    // Paths are registered under their UTF-8 form and looked up as &str.
    let id = files.get_id("src/main.rs").expect("present");
    assert_eq!(files.get_path(id), Some("src/main.rs"));
    assert_eq!(files.get_content(id), Some(&b"fn main() {}"[..]));
    assert_eq!(files.len(), 2);
}

#[test]
fn add_path_reports_invalid_utf8_without_registering() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("valid.txt", b"v").expect("fits");
    let bytes_before = files.total_bytes();

    // A path with invalid UTF-8 bytes cannot be stored losslessly.
    let invalid = OsStr::from_bytes(b"caf\xe9.txt");
    let error = files.add_path(invalid, b"x").expect_err("invalid utf-8");
    match error {
        MapError::PathNotUtf8 { path } => {
            assert_eq!(path, "caf\u{FFFD}.txt", "the lossy path is reported");
        }
        other => panic!("unexpected error: {other:?}"),
    }

    // The failed registration leaves the map untouched.
    assert_eq!(files.len(), 1);
    assert_eq!(files.total_bytes(), bytes_before);
    files.finalize().expect("fits");
    assert!(files.get_id("valid.txt").is_some());
}
