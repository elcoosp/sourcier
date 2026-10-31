//! The source file registry: paths, contents and stable numeric ids.
//!
//! [`SourceFilesMap`] collects files in a build phase and, once
//! [`finalize`](SourceFilesMap::finalize) is called, hands out compact
//! [`FileId`]s with constant-time lookups in both
//! directions. Finalizing sorts the files by path, removes duplicate paths
//! (keeping the first occurrence) and assigns one-based ids in path order.

use std::collections::HashMap;

#[cfg(feature = "rt-feedback")]
use std::sync::Arc;

use crate::error::MapError;
use crate::file_id::FileId;

#[cfg(feature = "rt-feedback")]
use crate::feedback::RuntimeFeedback;
#[cfg(feature = "view")]
use crate::line_map::LineOffsets;
#[cfg(feature = "view")]
use crate::position::SourceFilePosition;

/// A borrowed view of one registered file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileRef<'a, Id: FileId> {
    /// Numeric id assigned to the file.
    pub id: Id,
    /// Path the file was registered under.
    pub path: &'a str,
    /// Raw content bytes of the file.
    pub content: &'a [u8],
}

/// Internal storage for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Entry {
    path: Box<str>,
    content: Box<[u8]>,
}

/// A finalized registry of source files with compact numeric ids.
///
/// # Build phase
///
/// Call [`add_file`](SourceFilesMap::add_file) any number of times, then
/// [`finalize`](SourceFilesMap::finalize). The map can be tuned with
/// [`with_capacity`](SourceFilesMap::with_capacity) to pre-reserve storage.
///
/// # Lookup phase
///
/// After finalization the map is immutable and answers queries in constant
/// time: [`get_id`](SourceFilesMap::get_id), [`get_path`](SourceFilesMap::get_path),
/// [`get_content`](SourceFilesMap::get_content) and [`get`](SourceFilesMap::get).
/// With the `view` feature, [`view`](SourceFilesMap::view) and
/// [`view_str`](SourceFilesMap::view_str) extract the bytes covered by a
/// position.
///
/// # Examples
///
/// ```
/// use sourcier_core::{AbsolutePosition, SourceFilesMap};
///
/// let mut files = SourceFilesMap::<u8>::new();
/// files.add_file("src/main.rs", b"fn main() {\n    println!(\"Hello\");\n}").expect("capacity available");
/// files.add_file("src/lib.rs", b"pub mod utils;").expect("capacity available");
/// files.finalize().expect("no capacity issue");
///
/// let id = files.get_id("src/main.rs").expect("file present");
/// assert_eq!(files.get_path(id), Some("src/main.rs"));
/// assert_eq!(files.get_content(id).unwrap().len(), 36);
/// assert_eq!(files.len(), 2);
///
/// // Ids are assigned in sorted path order, starting at 1.
/// assert_eq!(u8::from(files.get_id("src/lib.rs").expect("file present")), 1);
///
/// let pos = AbsolutePosition::new(id, 2, 5, 2, 22);
/// assert_eq!(files.view(id, &pos), Some(&b"println!(\"Hello\");"[..]));
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SourceFilesMap<Id: FileId> {
    files: Vec<Entry>,
    path_to_id: HashMap<Box<str>, Id>,
    finalized: bool,

    /// Per-file line offset indexes, maintained by the `view` feature.
    #[cfg(feature = "view")]
    #[cfg_attr(feature = "serde", serde(skip))]
    line_offsets: HashMap<Id, LineOffsets>,

    /// Shared usage statistics, maintained by the `rt-feedback` feature.
    #[cfg(feature = "rt-feedback")]
    #[cfg_attr(feature = "serde", serde(skip))]
    feedback: Option<Arc<RuntimeFeedback>>,
}

/// Sort comparator: entries in lexicographic path order.
///
/// A named function (instead of a closure) keeps a single codegen copy that
/// is shared by every `FileId` instantiation of `finalize`.
fn path_order(a: &Entry, b: &Entry) -> core::cmp::Ordering {
    a.path.cmp(&b.path)
}

/// Dedup predicate: duplicate paths keep their first registration.
fn same_path(a: &mut Entry, b: &mut Entry) -> bool {
    a.path == b.path
}

impl<Id: FileId> Default for SourceFilesMap<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: FileId> SourceFilesMap<Id> {
    /// Creates an empty map without pre-reserved storage.
    #[inline]
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            path_to_id: HashMap::new(),
            finalized: false,
            #[cfg(feature = "view")]
            line_offsets: HashMap::new(),
            #[cfg(feature = "rt-feedback")]
            feedback: None,
        }
    }

    /// Creates an empty map that pre-reserves storage for `expected_files`
    /// files, reducing reallocations while building.
    #[inline]
    pub fn with_capacity(expected_files: usize) -> Self {
        Self {
            files: Vec::with_capacity(expected_files),
            path_to_id: HashMap::with_capacity(expected_files),
            finalized: false,
            #[cfg(feature = "view")]
            line_offsets: HashMap::with_capacity(expected_files),
            #[cfg(feature = "rt-feedback")]
            feedback: None,
        }
    }

    /// Creates an empty map that reports its finalization statistics into
    /// the shared `feedback` context.
    ///
    /// Requires the `rt-feedback` feature.
    #[cfg(feature = "rt-feedback")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-feedback")))]
    #[inline]
    pub fn with_feedback(feedback: Arc<RuntimeFeedback>) -> Self {
        Self {
            feedback: Some(feedback),
            ..Self::new()
        }
    }

    /// Registers a file with its raw byte content.
    ///
    /// Paths are not required to be unique or sorted; duplicates are resolved
    /// during [`finalize`](Self::finalize) (the first occurrence of a path
    /// wins). Fails with [`MapError::TooManyFiles`] once the id type is
    /// exhausted, or [`MapError::Finalized`] after finalization.
    pub fn add_file(
        &mut self,
        path: impl AsRef<str>,
        content: impl Into<Vec<u8>>,
    ) -> Result<(), MapError> {
        if self.finalized {
            return Err(MapError::Finalized);
        }
        if self.files.len() >= Id::MAX_FILES {
            return Err(MapError::TooManyFiles { max: Id::MAX_FILES });
        }
        self.files.push(Entry {
            path: path.as_ref().into(),
            content: content.into().into(),
        });
        Ok(())
    }

    /// Sorts the files by path, deduplicates paths, assigns one-based ids in
    /// path order and switches the map to its immutable lookup phase.
    ///
    /// Calling this method twice fails with [`MapError::AlreadyFinalized`].
    pub fn finalize(&mut self) -> Result<(), MapError> {
        if self.finalized {
            return Err(MapError::AlreadyFinalized);
        }
        if self.files.len() > Id::MAX_FILES {
            return Err(MapError::TooManyFiles { max: Id::MAX_FILES });
        }

        // Sort by path so ids follow a deterministic, path-lexicographic
        // order, then drop duplicate paths keeping the first occurrence.
        self.files.sort_unstable_by(path_order);
        self.files.dedup_by(same_path);

        self.path_to_id.clear();
        self.path_to_id.reserve(self.files.len());
        // The capacity check above keeps the registry within `Id::MAX_FILES`,
        // so every index has a one-based id; `filter_map` absorbs the range
        // check of `FileId::from_index` without introducing a dead branch.
        let ids = (0..self.files.len()).filter_map(Id::from_index);
        for (entry, id) in self.files.iter().zip(ids) {
            self.path_to_id.insert(entry.path.clone(), id);
        }

        #[cfg(feature = "view")]
        self.rebuild_line_offsets();

        #[cfg(feature = "rt-feedback")]
        if let Some(feedback) = &self.feedback {
            let total_bytes: u64 = self.files.iter().map(|e| e.content.len() as u64).sum();
            let max_file_size = self
                .files
                .iter()
                .map(|e| e.content.len())
                .max()
                .unwrap_or(0);
            feedback.record(self.files.len(), total_bytes, max_file_size);
        }

        self.finalized = true;
        Ok(())
    }

    /// Returns `true` once the map has been finalized.
    #[inline]
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Returns the number of registered files.
    #[inline]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns `true` when no file is registered.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the total number of content bytes currently held.
    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.files.iter().map(|e| e.content.len()).sum()
    }

    /// Iterates over the registered paths.
    ///
    /// Before finalization the paths appear in insertion order; afterwards
    /// they are sorted.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|e| e.path.as_ref())
    }

    /// Iterates over all files together with their ids.
    ///
    /// Returns nothing before finalization, because ids only exist afterwards.
    pub fn iter(&self) -> impl Iterator<Item = FileRef<'_, Id>> {
        let finalized = self.finalized;
        self.files
            .iter()
            .enumerate()
            .filter_map(move |(index, entry)| {
                if !finalized {
                    return None;
                }
                let id = Id::from_index(index)?;
                Some(FileRef {
                    id,
                    path: entry.path.as_ref(),
                    content: entry.content.as_ref(),
                })
            })
    }

    /// Returns the id assigned to `path`, or `None` for unknown paths and
    /// unfinalized maps.
    #[inline]
    pub fn get_id(&self, path: &str) -> Option<Id> {
        if !self.finalized {
            return None;
        }
        self.path_to_id.get(path).copied()
    }

    /// Returns the path registered under `id`, or `None` for invalid ids and
    /// unfinalized maps.
    pub fn get_path(&self, id: Id) -> Option<&str> {
        if !self.finalized {
            return None;
        }
        let index = self.index_of(id)?;
        self.files.get(index).map(|e| e.path.as_ref())
    }

    /// Returns the raw content registered under `id`, or `None` for invalid
    /// ids and unfinalized maps.
    pub fn get_content(&self, id: Id) -> Option<&[u8]> {
        if !self.finalized {
            return None;
        }
        let index = self.index_of(id)?;
        self.files.get(index).map(|e| e.content.as_ref())
    }

    /// Returns both path and content registered under `id`.
    pub fn get(&self, id: Id) -> Option<FileRef<'_, Id>> {
        if !self.finalized {
            return None;
        }
        let index = self.index_of(id)?;
        self.files.get(index).map(|entry| FileRef {
            id,
            path: entry.path.as_ref(),
            content: entry.content.as_ref(),
        })
    }

    /// Returns the bytes covered by a one-based position span, or `None`
    /// when the position does not address valid bytes of the file.
    ///
    /// Requires the `view` feature. Columns are byte columns. Both the start
    /// and the end column are inclusive; a column pointing one past the last
    /// byte of a line resolves to the line terminator (or to one past the
    /// last byte of the file for the final line), so a span ending there
    /// includes the terminator. Valid zero-width spans yield an empty slice.
    /// Files whose content exceeds the 4 GiB line offset limit
    /// ([`LineOffsets::MAX_CONTENT_LENGTH`](crate::line_map::LineOffsets))
    /// are not indexed; `view` returns `None` for them.
    #[cfg(feature = "view")]
    #[cfg_attr(docsrs, doc(cfg(feature = "view")))]
    pub fn view(&self, id: Id, pos: &impl SourceFilePosition) -> Option<&[u8]> {
        let content = self.get_content(id)?;
        let offsets = self.line_offsets.get(&id)?;

        let start_line = pos.start_line() as usize;
        let start_col = pos.start_column() as usize;
        let end_line = pos.end_line() as usize;
        let end_col = pos.end_column() as usize;

        // Positions are one-based.
        if start_col == 0 || end_col == 0 {
            return None;
        }

        let start_range = offsets.line_range(start_line)?;
        let end_range = offsets.line_range(end_line)?;

        let start_byte = start_range.start + start_col - 1;
        // The end column is inclusive; it may point at the line terminator
        // (or one past the last byte of the file on the final line).
        let end_limit = end_range.end + 1;
        let end_byte = end_range.start + end_col;
        if end_byte > end_limit {
            return None;
        }
        // On a final line without terminator the EOF column is exclusive.
        let end_byte = end_byte.min(content.len());

        // Columns must stay inside their line, and the end byte must not
        // come before the start byte (this also rejects start > end lines).
        if start_byte > start_range.end || start_byte > end_byte {
            return None;
        }

        Some(&content[start_byte..end_byte])
    }

    /// Like [`view`](Self::view) but only returns the slice when it is valid
    /// UTF-8.
    ///
    /// Requires the `view` feature.
    #[cfg(feature = "view")]
    #[cfg_attr(docsrs, doc(cfg(feature = "view")))]
    pub fn view_str(&self, id: Id, pos: &impl SourceFilePosition) -> Option<&str> {
        core::str::from_utf8(self.view(id, pos)?).ok()
    }

    /// Recomputes the per-file line offset indexes.
    ///
    /// Contents beyond the 4 GiB line offset limit of
    /// [`LineOffsets::MAX_CONTENT_LENGTH`](crate::line_map::LineOffsets) are
    /// not indexed; `view` on such files returns `None` like for any other
    /// unindexed file.
    #[cfg(feature = "view")]
    fn rebuild_line_offsets(&mut self) {
        self.line_offsets.clear();
        // Ids always exist here for the same reason as in `finalize`.
        let ids = (0..self.files.len()).filter_map(Id::from_index);
        let indexes = self.files.iter().zip(ids).filter_map(|(entry, id)| {
            LineOffsets::compute(&entry.content)
                .ok()
                .map(|offsets| (id, offsets))
        });
        self.line_offsets.extend(indexes);
    }

    /// Maps a one-based id to its zero-based index into `files`.
    #[inline]
    fn index_of(&self, id: Id) -> Option<usize> {
        let raw = id.to_raw();
        let index = raw.checked_sub(1)?;
        usize::try_from(index)
            .ok()
            .filter(|&i| i < self.files.len())
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use std::collections::HashMap;

    use serde::de::Deserialize;

    use super::{Entry, FileId, SourceFilesMap};

    impl<Id: FileId> SourceFilesMap<Id> {
        /// Recomputes feature-specific derived state after deserialization.
        pub(crate) fn rebuild_derived_state(&mut self) {
            #[cfg(feature = "view")]
            if self.finalized {
                self.rebuild_line_offsets();
            }
        }
    }

    impl<'de, Id> Deserialize<'de> for SourceFilesMap<Id>
    where
        Id: FileId + serde::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(serde::Deserialize)]
            #[serde(bound = "Id: serde::Deserialize<'de>")]
            struct Repr<Id> {
                files: Vec<Entry>,
                path_to_id: HashMap<Box<str>, Id>,
                finalized: bool,
            }

            let repr = Repr::<Id>::deserialize(deserializer)?;
            let mut map = SourceFilesMap::<Id> {
                files: repr.files,
                path_to_id: repr.path_to_id,
                finalized: repr.finalized,
                #[cfg(feature = "view")]
                line_offsets: HashMap::new(),
                #[cfg(feature = "rt-feedback")]
                feedback: None,
            };
            map.rebuild_derived_state();
            Ok(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SourceFilesMap<u8> {
        let mut files = SourceFilesMap::<u8>::new();
        files.add_file("b.txt", b"beta").expect("fits");
        files.add_file("a.txt", b"alpha").expect("fits");
        files.finalize().expect("fits");
        files
    }

    #[test]
    fn ids_follow_sorted_path_order() {
        let files = sample();
        let id_a = files.get_id("a.txt").expect("present");
        let id_b = files.get_id("b.txt").expect("present");
        assert_eq!(id_a, 1);
        assert_eq!(id_b, 2);
        assert_eq!(files.get_path(id_a), Some("a.txt"));
        assert_eq!(files.get_content(id_a), Some(&b"alpha"[..]));
        assert_eq!(files.get_path(id_b), Some("b.txt"));
        assert_eq!(files.get_content(id_b), Some(&b"beta"[..]));
    }

    #[test]
    fn lookups_fail_before_finalization() {
        let mut files = SourceFilesMap::<u8>::new();
        files.add_file("a.txt", b"alpha").expect("fits");
        assert!(!files.is_finalized());
        assert_eq!(files.get_id("a.txt"), None);
        assert_eq!(files.get_path(1_u8), None);
        assert_eq!(files.get_content(1_u8), None);
        assert_eq!(files.get(1_u8), None);
        assert_eq!(files.iter().count(), 0);
    }

    #[test]
    fn adding_after_finalization_fails() {
        let mut files = sample();
        assert!(files.is_finalized());
        assert_eq!(files.add_file("c.txt", b"gamma"), Err(MapError::Finalized));
        assert_eq!(files.finalize(), Err(MapError::AlreadyFinalized));
    }

    #[test]
    fn capacity_is_enforced_at_insertion() {
        let mut files = SourceFilesMap::<u8>::new();
        for index in 0..255 {
            files
                .add_file(format!("f{index}.txt"), b"x")
                .expect("within capacity");
        }
        assert_eq!(
            files.add_file("overflow.txt", b"x"),
            Err(MapError::TooManyFiles { max: 255 })
        );
        files.finalize().expect("exactly at capacity");
        assert_eq!(files.len(), 255);
        assert!(files.get_id("f254.txt").is_some());
        assert!(files.get_id("overflow.txt").is_none());
    }

    #[test]
    fn duplicate_paths_keep_the_first_content() {
        let mut files = SourceFilesMap::<u8>::new();
        files.add_file("dup.txt", b"first").expect("fits");
        files.add_file("dup.txt", b"second").expect("fits");
        files.finalize().expect("fits");
        assert_eq!(files.len(), 1);
        let id = files.get_id("dup.txt").expect("present");
        assert_eq!(files.get_content(id), Some(&b"first"[..]));
    }

    #[test]
    fn iteration_covers_all_files_with_ids() {
        let files = sample();
        let collected: Vec<_> = files.iter().map(|f| (u64::from(f.id), f.path)).collect();
        assert_eq!(collected, vec![(1, "a.txt"), (2, "b.txt")]);
        let owned = files
            .get(files.get_id("a.txt").expect("present"))
            .expect("present");
        assert_eq!(owned.path, "a.txt");
        assert_eq!(owned.content, b"alpha");
    }

    #[test]
    fn metrics_and_paths() {
        let mut files = SourceFilesMap::<u8>::new();
        assert!(files.is_empty());
        assert_eq!(files.total_bytes(), 0);
        files.add_file("one.txt", b"1234").expect("fits");
        files.add_file("two.txt", b"12").expect("fits");
        assert_eq!(files.len(), 2);
        assert_eq!(files.total_bytes(), 6);
        assert_eq!(
            files.paths().collect::<Vec<_>>(),
            vec!["one.txt", "two.txt"]
        );
        files.finalize().expect("fits");
        assert_eq!(
            files.paths().collect::<Vec<_>>(),
            vec!["one.txt", "two.txt"]
        );
    }

    #[test]
    fn id_zero_never_resolves() {
        let files = sample();
        // Raw id 0 is not assigned to any file and must not underflow.
        assert_eq!(files.get_content(0_u8), None);
        assert_eq!(files.get_path(0_u8), None);
        assert_eq!(files.get(0_u8), None);
    }
}
