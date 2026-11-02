//! Integration tests for the `serde` feature.

#![cfg(feature = "serde")]

use core::fmt::Write as _;

use crate::{
    AbsolutePosition, MapError, RelativePosition, SourceFilesMap, StandardAbsolutePosition,
};

/// Crafts JSON for a map with 300 files, more than a `u8` id type supports.
fn crafted_oversized_map_json(finalized: bool) -> String {
    let mut json = String::from("{\"files\":[");
    for index in 0..300 {
        if index > 0 {
            json.push(',');
        }
        let _ = write!(json, "{{\"path\":\"f{index:03}\",\"content\":[120]}}");
    }
    let _ = write!(json, "],\"finalized\":{finalized}}}");
    json
}

#[test]
fn positions_serialize_losslessly() {
    let abs = AbsolutePosition::<u8>::new(200, 1_234, 56, 4_321, 78);
    let json = serde_json::to_string(&abs).expect("serializable");
    let rebuilt: AbsolutePosition<u8> = serde_json::from_str(&json).expect("deserializable");
    assert_eq!(abs, rebuilt);
    assert_eq!(abs.as_raw(), rebuilt.as_raw());

    let std = StandardAbsolutePosition::new(65_000, 10, 1, 20, 2);
    let json = serde_json::to_string(&std).expect("serializable");
    let rebuilt: StandardAbsolutePosition = serde_json::from_str(&json).expect("deserializable");
    assert_eq!(std, rebuilt);

    let rel = RelativePosition::new(12, 34, 56, 78);
    let json = serde_json::to_string(&rel).expect("serializable");
    let rebuilt: RelativePosition = serde_json::from_str(&json).expect("deserializable");
    assert_eq!(rel, rebuilt);
}

#[test]
fn finalized_map_round_trips_with_lookup_state() {
    let mut files = SourceFilesMap::<u16>::new();
    files.add_file("src/lib.rs", b"pub mod a;").expect("fits");
    files
        .add_file("src/main.rs", b"fn main() {}\n")
        .expect("fits");
    files.finalize().expect("fits");

    let json = serde_json::to_string(&files).expect("serializable");
    let rebuilt: SourceFilesMap<u16> = serde_json::from_str(&json).expect("deserializable");

    assert!(rebuilt.is_finalized());
    assert_eq!(rebuilt.len(), 2);
    let id = rebuilt.get_id("src/main.rs").expect("id restored");
    assert_eq!(rebuilt.get_path(id), Some("src/main.rs"));
    assert_eq!(rebuilt.get_content(id), Some(&b"fn main() {}\n"[..]));
    assert_eq!(rebuilt.get_content(id), files.get_content(id));
    assert_eq!(rebuilt.iter().count(), 2);
}

#[test]
fn serialized_format_omits_redundant_lookup_state() {
    // The path-to-id table is derived from the sorted files, so v2 payloads
    // no longer carry it: payloads shrink and deserialized maps can never
    // hold inconsistent lookup state.
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("a.txt", b"alpha").expect("fits");
    files.finalize().expect("fits");

    let json = serde_json::to_string(&files).expect("serializable");
    assert!(
        !json.contains("path_to_id"),
        "derived lookup state must not be serialized: {json}"
    );
    assert!(json.contains("finalized"), "{json}");

    let rebuilt: SourceFilesMap<u8> = serde_json::from_str(&json).expect("deserializable");
    assert_eq!(rebuilt.get_id("a.txt"), Some(1));
}

#[test]
fn legacy_payloads_with_lookup_state_still_load() {
    // v1 payloads carried a `path_to_id` table; deserialization stays a
    // tolerant reader and ignores the redundant field.
    let legacy = r#"{
        "files": [{"path": "legacy.txt", "content": [108, 105, 110, 101]}],
        "path_to_id": {"legacy.txt": 1},
        "finalized": true
    }"#;
    let rebuilt: SourceFilesMap<u8> = serde_json::from_str(legacy).expect("tolerant reader");
    assert!(rebuilt.is_finalized());
    let id = rebuilt.get_id("legacy.txt").expect("path indexed");
    assert_eq!(id, 1);
    assert_eq!(rebuilt.get_path(1), Some("legacy.txt"));
    assert_eq!(rebuilt.get_content(1), Some(&b"line"[..]));
}

#[test]
fn deserialized_unfinalized_maps_keep_first_registration_wins() {
    // A deserialized build-phase map can be finalized later; duplicate
    // resolution must keep the first registration of the serialized order.
    let mut files = SourceFilesMap::<u8>::new();
    files.add_file("dup.txt", b"first").expect("fits");
    files.add_file("z.txt", b"zeta").expect("fits");
    files.add_file("dup.txt", b"second").expect("fits");
    files.add_file("a.txt", b"alpha").expect("fits");

    let json = serde_json::to_string(&files).expect("serializable");
    let mut rebuilt: SourceFilesMap<u8> = serde_json::from_str(&json).expect("deserializable");
    rebuilt.finalize().expect("fits");

    let id = rebuilt.get_id("dup.txt").expect("present");
    assert_eq!(rebuilt.get_content(id), Some(&b"first"[..]));
    assert_eq!(rebuilt.len(), 3);
}

#[test]
fn deserialized_map_supports_views_again() {
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_file("text.txt", b"alpha\nbeta\ngamma")
        .expect("fits");
    files.finalize().expect("fits");

    let json = serde_json::to_string(&files).expect("serializable");
    let rebuilt: SourceFilesMap<u8> = serde_json::from_str(&json).expect("deserializable");

    let id = rebuilt.get_id("text.txt").expect("present");
    let pos = AbsolutePosition::new(id, 2, 1, 2, 4);
    assert_eq!(rebuilt.view_str(id, &pos), Some("beta"));
}

#[test]
fn build_phase_map_round_trips_too() {
    let mut files = SourceFilesMap::<u8>::new();
    files
        .add_file("pending.txt", b"not yet finalized")
        .expect("fits");

    let json = serde_json::to_string(&files).expect("serializable");
    let mut rebuilt: SourceFilesMap<u8> = serde_json::from_str(&json).expect("deserializable");

    assert!(!rebuilt.is_finalized());
    assert_eq!(rebuilt.len(), 1);
    // The restored map can still be finalized.
    rebuilt.finalize().expect("fits");
    assert!(rebuilt.get_id("pending.txt").is_some());
}

#[test]
fn malformed_json_is_rejected() {
    let broken: Result<SourceFilesMap<u8>, _> = serde_json::from_str("{not json");
    assert!(broken.is_err());

    let missing_fields: Result<SourceFilesMap<u8>, _> = serde_json::from_str("{}");
    assert!(missing_fields.is_err());

    let wrong_type: Result<SourceFilesMap<u8>, _> = serde_json::from_str("{\"files\": 3}");
    assert!(wrong_type.is_err());
}

#[test]
fn deserialized_maps_beyond_capacity_fail_to_finalize() {
    // Hand-crafted data can carry more files than a `u8` id type supports;
    // finalization must reject it instead of producing broken lookups.
    let json = crafted_oversized_map_json(false);

    let mut rebuilt: SourceFilesMap<u8> = serde_json::from_str(&json).expect("structurally valid");
    assert_eq!(rebuilt.len(), 300);
    assert_eq!(rebuilt.finalize(), Err(MapError::TooManyFiles { max: 255 }));
    // The failed finalization leaves the map consistent: nothing is looked
    // up and no ids were assigned.
    assert!(!rebuilt.is_finalized());
    assert_eq!(rebuilt.iter().count(), 0);
}

#[test]
fn iteration_skips_files_beyond_the_id_capacity() {
    // Finalized maps from hand-crafted data can reference more files than
    // the id type supports; iteration simply skips entries without a valid
    // id instead of panicking.
    let json = crafted_oversized_map_json(true);

    let rebuilt: SourceFilesMap<u8> = serde_json::from_str(&json).expect("structurally valid");
    assert_eq!(rebuilt.len(), 300);
    assert_eq!(
        rebuilt.iter().count(),
        255,
        "files without a valid id are skipped"
    );
    assert_eq!(rebuilt.get_path(255), Some("f254"));
    assert_eq!(rebuilt.get_id("f299"), None);
}
