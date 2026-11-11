//! Integration tests for the type-state [`SourceFilesBuilder`]: the
//! build-phase companion that makes premature queries unrepresentable, plus
//! the atomic batch registration shared with the map.

use crate::feedback::RuntimeFeedback;
use crate::{AbsolutePosition, MapError, SourceFilesBuilder, SourceFilesMap};
use std::sync::Arc;

#[test]
fn builder_chains_registration_into_a_finalized_map() {
    // The consuming `add_file` returns the builder, so a whole setup reads
    // as one expression ending in `finalize`.
    let files = SourceFilesBuilder::<u8>::new()
        .add_file("src/main.rs", b"fn main() {}")
        .expect("capacity available")
        .add_file("src/lib.rs", b"pub mod utils;")
        .expect("capacity available")
        .finalize()
        .expect("no capacity issue");

    assert!(files.is_finalized());
    assert_eq!(files.len(), 2);
    // Ids follow the sorted path order, like any finalized map.
    let id = files.get_id("src/lib.rs").expect("present");
    assert_eq!(u64::from(id), 1);
    assert_eq!(files.get_path(id), Some("src/lib.rs"));
}

#[test]
fn builder_supports_loop_registration_and_remaining_capacity() {
    let mut builder = SourceFilesBuilder::<u8>::with_capacity(10);
    for i in 0..10 {
        builder
            .add_file_mut(format!("f{i}.rs"), format!("content {i}"))
            .expect("fits");
    }
    assert_eq!(builder.len(), 10);
    assert!(!builder.is_empty());
    // `u8` ids support 255 files; 10 are registered.
    assert_eq!(builder.remaining_capacity(), 245);
    // Paths read back in registration order during the build phase.
    assert_eq!(builder.paths().next(), Some("f0.rs"));

    let files = builder.finalize().expect("fits");
    assert_eq!(files.get_content(10_u8), Some(&b"content 9"[..]));
}

#[test]
fn builder_finalize_consumes_the_builder() {
    // `finalize` takes `self` by value: a finalized builder cannot be
    // registered into again, which the compiler enforces. The returned map
    // is already in its lookup phase.
    let builder = SourceFilesBuilder::<u16>::default();
    assert_eq!(builder.len(), 0);
    let files = builder.finalize().expect("empty map finalizes");
    assert!(files.is_finalized());
    assert!(files.is_empty());
}

#[test]
fn builder_keeps_the_map_semantics() {
    // Duplicate paths resolve first-registration-wins, ids are one-based
    // and sorted, exactly like building a map by hand.
    let files = SourceFilesBuilder::<u8>::new()
        .add_file("dup.txt", b"first")
        .expect("fits")
        .add_file("dup.txt", b"second")
        .expect("fits")
        .add_file("a.txt", b"alpha")
        .expect("fits")
        .finalize()
        .expect("fits");

    assert_eq!(files.len(), 2);
    let id = files.get_id("dup.txt").expect("present");
    assert_eq!(files.get_content(id), Some(&b"first"[..]));
}

#[test]
fn builder_reports_id_capacity_errors() {
    let mut builder = SourceFilesBuilder::<u8>::new();
    for i in 0..255 {
        builder
            .add_file_mut(format!("f{i:03}.rs"), b"")
            .expect("fits");
    }
    assert_eq!(builder.remaining_capacity(), 0);
    assert_eq!(
        builder.add_file_mut("overflow.rs", b""),
        Err(MapError::TooManyFiles { max: 255 })
    );

    // The batch form stops at the same limit.
    let exhausted = builder.add_files([("overflow.rs", b"")]);
    assert!(matches!(
        exhausted,
        Err(MapError::TooManyFiles { max: 255 })
    ));
}

#[test]
fn builder_batches_register_atomically() {
    // A batch that runs out of ids leaves no partial registration behind;
    // the mutating form keeps the builder usable after the failure.
    let mut builder = SourceFilesBuilder::<u8>::new();
    let batch: Vec<(String, &[u8])> = (0..300)
        .map(|i| (format!("f{i:03}.rs"), b"x".as_slice()))
        .collect();

    let error = builder
        .add_files_mut(
            batch
                .iter()
                .map(|(path, content)| (path.as_str(), *content)),
        )
        .expect_err("over capacity");
    assert_eq!(error, MapError::TooManyFiles { max: 255 });

    // All-or-nothing: the builder behaves as if nothing had been added.
    assert_eq!(builder.len(), 0);
    assert_eq!(builder.remaining_capacity(), 255);

    // The builder stays usable after the failed batch.
    let files = builder
        .add_file("kept.rs", b"kept")
        .expect("fits")
        .finalize()
        .expect("fits");
    assert_eq!(files.len(), 1);
    assert!(files.get_id("kept.rs").is_some());
}

#[test]
fn map_batches_register_atomically() {
    let mut files = SourceFilesMap::<u8>::new();
    for i in 0..254 {
        files.add_file(format!("f{i:03}.rs"), b"x").expect("fits");
    }
    let bytes_before = files.total_bytes();

    let batch: Vec<(String, Vec<u8>)> = (0..3)
        .map(|i| (format!("late{i}.rs"), vec![b'y'; i + 1]))
        .collect();
    assert_eq!(
        files.add_files(batch),
        Err(MapError::TooManyFiles { max: 255 })
    );

    // Nothing from the failed batch is registered, and the totals are
    // exactly as before.
    assert_eq!(files.len(), 254);
    assert_eq!(files.total_bytes(), bytes_before);
    files.finalize().expect("fits");
    assert!(files.get_id("late0.rs").is_none());
    assert!(files.get_id("f253.rs").is_some());
}

#[test]
fn map_batches_accept_mixed_path_and_content_types() {
    let mut files = SourceFilesMap::<u8>::new();
    // Both the path and the content sides accept anything that converts:
    // `&str`/`String` paths, owned or borrowed byte contents.
    files
        .add_files([("a.txt", "alpha".to_owned().into_bytes())])
        .expect("fits");
    files
        .add_files([(String::from("b.txt"), &b"beta"[..])])
        .expect("fits");
    files.finalize().expect("fits");

    assert_eq!(
        files.get_content(files.get_id("a.txt").expect("present")),
        Some(&b"alpha"[..])
    );
    assert_eq!(
        files.get_content(files.get_id("b.txt").expect("present")),
        Some(&b"beta"[..])
    );
    assert_eq!(files.total_bytes(), 9);
}

#[cfg(feature = "view")]
#[test]
fn map_batch_finalizes_into_working_views() {
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_files([("text.txt", b"alpha\nbeta".to_vec())])
        .expect("fits");
    files.finalize().expect("fits");

    let id = files.get_id("text.txt").expect("present");
    let pos = AbsolutePosition::new(id, 2, 1, 2, 4);
    assert_eq!(files.view_str(id, &pos), Some("beta"));
}

#[cfg(feature = "rt-feedback")]
#[test]
fn builder_feedback_flows_into_the_shared_context() {
    let feedback = Arc::new(RuntimeFeedback::new());
    let files = SourceFilesBuilder::<u8>::with_feedback(Arc::clone(&feedback))
        .add_file("a.rs", b"fn a() {}")
        .expect("fits")
        .finalize()
        .expect("fits");

    let snapshot = feedback.snapshot();
    assert_eq!(snapshot.total_files, 1);
    assert_eq!(snapshot.total_bytes, 9);
    assert_eq!(snapshot.usage_count, 1);
    assert!(files.is_finalized());
}

#[test]
fn builder_registers_path_types() {
    use std::path::Path;
    use std::path::PathBuf;

    let files = SourceFilesBuilder::<u8>::new()
        .add_path(Path::new("src/lib.rs"), b"pub mod a;")
        .expect("utf-8 path")
        .add_path(PathBuf::from("src/main.rs"), b"fn main() {}")
        .expect("utf-8 path")
        .finalize()
        .expect("fits");

    assert_eq!(files.len(), 2);
    let id = files.get_id("src/main.rs").expect("present");
    assert_eq!(files.get_content(id), Some(&b"fn main() {}"[..]));
}

#[test]
#[cfg(unix)]
fn builder_reports_invalid_utf8_paths_without_registering() {
    use std::ffi::OsStr;
    #[cfg(unix)]
    use std::os::unix::ffi::OsStrExt;

    let mut builder = SourceFilesBuilder::<u8>::new();
    builder.add_file_mut("valid.txt", b"v").expect("fits");

    let invalid = OsStr::from_bytes(b"caf\xe9.txt");
    assert!(matches!(
        builder.add_path_mut(invalid, b"x"),
        Err(MapError::PathNotUtf8 { .. })
    ));

    // The failed registration leaves the builder untouched and usable.
    assert_eq!(builder.len(), 1);
    let files = builder.finalize().expect("fits");
    assert_eq!(files.len(), 1);
    assert!(files.get_id("valid.txt").is_some());
}
