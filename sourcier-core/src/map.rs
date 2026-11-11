//! The source file registry: paths, contents and stable numeric ids.
//!
//! [`SourceFilesMap`] collects files in a build phase and, once
//! [`finalize`](SourceFilesMap::finalize) is called, hands out compact
//! [`FileId`]s with constant-time lookups in both
//! directions. Finalizing sorts the files by path, removes duplicate paths
//! (keeping the first occurrence) and assigns one-based ids in path order.
//!
//! # Lookup state
//!
//! The lookup phase is served by two compact derived structures built once at
//! finalization:
//!
//! - A verified open-addressed index maps path hashes to entry indexes. The
//!   table stores 4-byte slot values instead of owned key strings, so
//!   finalization performs zero per-file string allocations, and every probe
//!   is verified against the stored path bytes, so hash collisions can never
//!   produce a wrong answer. The hash function is seeded per map, which keeps
//!   adversarial collision patterns (`HashDoS`) from being precomputed.
//! - Per-file [`LineOffsets`] live in a dense vector indexed by id, avoiding
//!   a hash lookup on every `view` call.
use crate::traits::SourceFilePosition;

use crate::error::MapError;
use crate::file_id::FileId;

use core::fmt;

#[cfg(feature = "rt-feedback")]
use std::sync::Arc;

#[cfg(feature = "rt-feedback")]
use crate::feedback::RuntimeFeedback;
#[cfg(feature = "view")]
use crate::line_map::LineOffsets;
#[cfg(feature = "view")]
use crate::position::{AbsolutePosition, SourceFilePosition};

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

/// Iterator over the files of a finalized map, yielded together with their
/// ids.
///
/// Created by [`iter`](SourceFilesMap::iter) and by
/// `&SourceFilesMap`'s [`IntoIterator`] implementation. Yields nothing
/// before finalization, because ids only exist afterwards.
#[derive(Debug, Clone)]
pub struct Iter<'a, Id: FileId> {
    files: core::iter::Enumerate<core::slice::Iter<'a, Entry>>,
    finalized: bool,
    marker: core::marker::PhantomData<Id>,
}

impl<'a, Id: FileId> Iterator for Iter<'a, Id> {
    type Item = FileRef<'a, Id>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.finalized {
            return None;
        }
        loop {
            let (index, entry) = self.files.next()?;
            if let Some(id) = Id::from_index(index) {
                return Some(FileRef {
                    id,
                    path: entry.path.as_str(),
                    content: entry.content.as_ref(),
                });
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.finalized {
            (0, Some(self.files.size_hint().0))
        } else {
            (0, Some(0))
        }
    }
}

/// Renders a position as `path:line:column`, the conventional diagnostic
/// anchor format.
///
/// Created by [`display_position`](SourceFilesMap::display_position). The
/// renderer borrows the file path and formats lazily, so building it
/// performs no allocation.
///
/// # Examples
///
/// ```
/// use sourcier_core::{AbsolutePosition, SourceFilesMap};
///
/// let mut files = SourceFilesMap::<u8>::new();
/// files.add_file("src/main.rs", b"fn main() {}").expect("fits");
/// files.finalize().expect("fits");
///
/// let id = files.get_id("src/main.rs").expect("present");
/// let pos = AbsolutePosition::new(id, 1, 4, 1, 7);
/// assert_eq!(
///     files.display_position(id, &pos).expect("known id").to_string(),
///     "src/main.rs:1:4"
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionDisplay<'a> {
    path: &'a str,
    line: u16,
    column: u8,
}

impl fmt::Display for PositionDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.path, self.line, self.column)
    }
}

/// Maximum byte length of a path stored inline inside its entry, without a
/// heap allocation. Most source-tree paths fit this bound, which keeps the
/// hot lookup path down to two dependent cache lines (index slot + entry).
const INLINE_PATH_CAP: usize = 30;

/// Path storage with short-string optimization.
///
/// Paths up to [`INLINE_PATH_CAP`] bytes live inline inside the 32-byte
/// payload, so registering them performs no allocation and lookups verify
/// their bytes without touching the heap. Longer paths fall back to a
/// `Box<str>`.
#[derive(Debug, Clone)]
enum Path {
    Inline { len: u8, buf: [u8; INLINE_PATH_CAP] },
    Heap(Box<str>),
}

impl Path {
    /// Stores `path`, inline when it fits the inline budget.
    fn new(path: &str) -> Self {
        let bytes = path.as_bytes();
        if bytes.len() <= INLINE_PATH_CAP {
            let mut buf = [0_u8; INLINE_PATH_CAP];
            buf[..bytes.len()].copy_from_slice(bytes);
            Self::Inline {
                len: bytes.len() as u8,
                buf,
            }
        } else {
            Self::Heap(path.into())
        }
    }

    /// Returns the raw path bytes.
    ///
    /// Byte equality is the verification primitive of the lookup index, so
    /// this accessor deliberately skips any UTF-8 handling.
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Inline { len, buf } => &buf[..*len as usize],
            Self::Heap(path) => path.as_bytes(),
        }
    }

    /// Returns the path as a string slice.
    fn as_str(&self) -> &str {
        match self {
            Self::Inline { len, buf } => {
                let bytes = &buf[..*len as usize];
                // The bytes came from a `&str` at registration time, so they
                // are valid UTF-8 by construction.
                core::str::from_utf8(bytes).expect("inline path stays valid UTF-8")
            }
            Self::Heap(path) => path,
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Path {}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Byte order matches the lexicographic `str` order.
        self.as_bytes().cmp(other.as_bytes())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Path {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        Ok(Self::new(&path))
    }
}

/// Internal storage for one file.
///
/// `hash` and `seq` are derived state: the hash depends on the per-map seed
/// and the sequence number records registration order, so both are rebuilt
/// after deserialization instead of being persisted.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Entry {
    /// Path hash mixed with the map seed; a cheap prefilter for dedup and
    /// lookup verification.
    #[cfg_attr(feature = "serde", serde(skip))]
    hash: u32,
    /// Registration sequence number; the sort tiebreak that guarantees the
    /// first registration of a duplicate path wins.
    #[cfg_attr(feature = "serde", serde(skip))]
    seq: u32,
    path: Path,
    content: Box<[u8]>,
}

/// Verified open-addressed index from path hash to entry index.
///
/// Slots hold `entry index + 1`; `0` marks an empty slot. The table is sized
/// to at least twice the entry count, so linear probing always terminates on
/// an empty slot. Lookups verify both the stored hash and the actual path
/// bytes, keeping the index exact under any collision pattern.
#[derive(Debug, Clone)]
struct PathIndex {
    mask: u32,
    slots: Vec<u32>,
}

impl PathIndex {
    /// Rebuilds the index over the first `count` entries.
    fn rebuild(&mut self, entries: &[Entry], count: usize) {
        let capacity = (count * 2).max(8).next_power_of_two();
        self.mask = (capacity - 1) as u32;
        self.slots.clear();
        self.slots.resize(capacity, 0);
        for (index, entry) in entries[..count].iter().enumerate() {
            let mut slot = entry.hash as usize & self.mask as usize;
            while self.slots[slot] != 0 {
                slot = (slot + 1) & self.mask as usize;
            }
            self.slots[slot] = index as u32 + 1;
        }
    }

    /// Number of occupied slots.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.slots.iter().filter(|&&slot| slot != 0).count()
    }
}

/// Returns a fresh random seed for the per-map path hashing.
fn random_seed() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
    hasher.write(b"sourcier");
    hasher.finish()
}

/// Hashes a path with two parallel multiply chains and one combine multiply.
///
/// The seed differs per map, so adversarial collision patterns cannot be
/// precomputed. The result is only a prefilter: probes always verify the
/// path bytes, so quality has no correctness impact.
///
/// The per-word multiplies run on two independent accumulation chains (even
/// and odd words), so their latencies overlap; a single final multiply mixes
/// the chains and the fold spreads the state over the table index range.
#[inline]
fn hash_path(seed: u64, bytes: &[u8]) -> u32 {
    const K: u64 = 0x517c_c1b7_2722_0a95;
    const K2: u64 = 0x9e37_79b9_7f4a_7c15;

    #[inline]
    fn tail_word(tail: &[u8]) -> u64 {
        let mut buf = [0_u8; 8];
        buf[..tail.len()].copy_from_slice(tail);
        u64::from_le_bytes(buf)
    }

    let (blocks, rest) = bytes.as_chunks::<16>();
    let mut even = seed;
    let mut odd = !seed;
    for block in blocks {
        let (words, _) = block.as_chunks::<8>();
        even = (even ^ u64::from_le_bytes(words[0])).wrapping_mul(K);
        odd = (odd ^ u64::from_le_bytes(words[1])).wrapping_mul(K);
    }
    let (chunks, tail) = rest.as_chunks::<8>();
    for chunk in chunks {
        even = (even ^ u64::from_le_bytes(*chunk)).wrapping_mul(K);
    }
    if !tail.is_empty() {
        odd = (odd ^ tail_word(tail)).wrapping_mul(K);
    }
    let combined = (even ^ odd.rotate_left(21)).wrapping_mul(K2);
    (combined ^ (combined >> 32)) as u32
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

    /// Verified open-addressed path index, built at finalization and skipped
    /// by serde: it is fully derived from the sorted `files`.
    #[cfg_attr(feature = "serde", serde(skip))]
    path_index: PathIndex,

    finalized: bool,

    /// Total content bytes, maintained incrementally in the build phase and
    /// recomputed once at finalization.
    #[cfg_attr(feature = "serde", serde(skip))]
    total_bytes: usize,

    /// Per-map seed for the path hashing (`HashDoS` resistance).
    #[cfg_attr(feature = "serde", serde(skip))]
    hash_seed: u64,

    /// Dense per-file line offset indexes, maintained by the `view` feature.
    #[cfg(feature = "view")]
    #[cfg_attr(feature = "serde", serde(skip))]
    line_offsets: Vec<Option<LineOffsets>>,

    /// Shared usage statistics, maintained by the `rt-feedback` feature.
    #[cfg(feature = "rt-feedback")]
    #[cfg_attr(feature = "serde", serde(skip))]
    feedback: Option<Arc<RuntimeFeedback>>,

    /// Marker tying the map to its [`FileId`] type.
    #[cfg_attr(feature = "serde", serde(skip))]
    marker: core::marker::PhantomData<Id>,
}

/// Sort comparator: entries in lexicographic path order.
///
/// The registration sequence number breaks ties between duplicate paths so
/// that the first registration always sorts first, which `dedup_by` then
/// keeps: the winner no longer depends on how the unstable sort permutes
/// equal paths.
///
/// A named function (instead of a closure) keeps a single codegen copy that
/// is shared by every `FileId` instantiation of `finalize`.
fn path_order(a: &Entry, b: &Entry) -> core::cmp::Ordering {
    a.path.cmp(&b.path).then(a.seq.cmp(&b.seq))
}

/// Dedup predicate: duplicate paths keep their first registration.
///
/// The precomputed hash acts as a cheap prefilter: differing hashes reject
/// the comparison without touching the path bytes.
fn same_path(a: &mut Entry, b: &mut Entry) -> bool {
    a.hash == b.hash && a.path == b.path
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
            path_index: PathIndex {
                mask: 0,
                slots: Vec::new(),
            },
            finalized: false,
            total_bytes: 0,
            hash_seed: random_seed(),
            #[cfg(feature = "view")]
            line_offsets: Vec::new(),
            #[cfg(feature = "rt-feedback")]
            feedback: None,
            marker: core::marker::PhantomData,
        }
    }

    /// Creates an empty map that pre-reserves storage for `expected_files`
    /// files, reducing reallocations while building.
    #[inline]
    pub fn with_capacity(expected_files: usize) -> Self {
        Self {
            files: Vec::with_capacity(expected_files),
            path_index: PathIndex {
                mask: 0,
                slots: Vec::new(),
            },
            finalized: false,
            total_bytes: 0,
            hash_seed: random_seed(),
            #[cfg(feature = "view")]
            line_offsets: Vec::new(),
            #[cfg(feature = "rt-feedback")]
            feedback: None,
            marker: core::marker::PhantomData,
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
    ///
    /// The path hash and total byte count are maintained incrementally, so
    /// registration stays O(1) in the file size.
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
        let path = path.as_ref();
        let content: Vec<u8> = content.into();
        self.total_bytes += content.len();
        self.files.push(Entry {
            hash: hash_path(self.hash_seed, path.as_bytes()),
            seq: self.files.len() as u32,
            path: Path::new(path),
            content: content.into(),
        });
        Ok(())
    }

    /// Registers a batch of files, reserving storage for the reported batch
    /// size up front.
    ///
    /// The batch is all-or-nothing: when any registration fails (running out
    /// of ids is the practical cause), every file added by the batch is
    /// removed again, the byte total is restored and the map is left exactly
    /// as it was. The error is reported unchanged, mirroring
    /// [`add_file`](Self::add_file).
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::SourceFilesMap;
    ///
    /// let mut files = SourceFilesMap::<u8>::new();
    /// files
    ///     .add_files([("a.txt", &b"alpha"[..]), ("b.txt", &b"beta"[..])])
    ///     .expect("capacity available");
    /// assert_eq!(files.len(), 2);
    /// ```
    pub fn add_files<I, P, C>(&mut self, files: I) -> Result<(), MapError>
    where
        I: IntoIterator<Item = (P, C)>,
        P: AsRef<str>,
        C: Into<Vec<u8>>,
    {
        let files = files.into_iter();
        let (lower, _) = files.size_hint();
        self.files.reserve(lower);

        let rollback = self.files.len();
        let mut failure = None;
        for (path, content) in files {
            if let Err(error) = self.add_file(path, content) {
                failure = Some(error);
                break;
            }
        }
        if let Some(error) = failure {
            self.files.truncate(rollback);
            // The incremental byte total is only maintained on the happy
            // path; recomputing it once over the retained entries restores
            // it exactly, and this runs on the failure path only.
            self.total_bytes = self.files.iter().map(|e| e.content.len()).sum();
            return Err(error);
        }
        Ok(())
    }

    /// Registers a file from a [`Path`]-like value with its raw byte
    /// content.
    ///
    /// A convenience over [`add_file`](Self::add_file) for tools that carry
    /// `Path`/`PathBuf` values: the path is converted to the `str` form the
    /// map stores. Paths that are not valid UTF-8 cannot be registered
    /// losslessly and fail with [`MapError::PathNotUtf8`], carrying the
    /// lossy rendering of the path; nothing is registered in that case.
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::SourceFilesMap;
    /// use std::path::PathBuf;
    ///
    /// let mut files = SourceFilesMap::<u8>::new();
    /// files
    ///     .add_path(PathBuf::from("src/main.rs"), b"fn main() {}")
    ///     .expect("utf-8 path");
    /// files.finalize().expect("fits");
    ///
    /// let id = files.get_id("src/main.rs").expect("present");
    /// assert_eq!(files.get_path(id), Some("src/main.rs"));
    /// ```
    ///
    /// [`Path`]: std::path::Path
    pub fn add_path(
        &mut self,
        path: impl AsRef<std::path::Path>,
        content: impl Into<Vec<u8>>,
    ) -> Result<(), MapError> {
        let path = path.as_ref();
        let Some(path) = path.to_str() else {
            return Err(MapError::PathNotUtf8 {
                path: path.to_string_lossy().into_owned(),
            });
        };
        self.add_file(path, content)
    }

    /// Sorts the files by path, deduplicates paths, assigns one-based ids in
    /// path order and switches the map to its immutable lookup phase.
    ///
    /// Finalization performs zero per-file string allocations: the lookup
    /// index is a verified hash table over the already-stored paths.
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
        // order, then drop duplicate paths keeping the first occurrence. The
        // sequence-number tiebreak makes the winner independent of the sort
        // internals.
        self.files.sort_unstable_by(path_order);
        self.files.dedup_by(same_path);
        self.files.shrink_to_fit();

        // Deduplication may have dropped bytes; recompute the total in the
        // same pass that feeds the lookup index.
        self.total_bytes = self.files.iter().map(|e| e.content.len()).sum();
        self.path_index
            .rebuild(&self.files, self.files.len().min(Id::MAX_FILES));

        #[cfg(feature = "view")]
        self.rebuild_line_offsets();

        #[cfg(feature = "rt-feedback")]
        if let Some(feedback) = &self.feedback {
            let max_file_size = self
                .files
                .iter()
                .map(|e| e.content.len())
                .max()
                .unwrap_or(0);
            feedback.record(self.files.len(), self.total_bytes as u64, max_file_size);
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
    ///
    /// The count is maintained incrementally during the build phase and
    /// cached after finalization, so this is O(1) at every point of the
    /// lifecycle.
    #[inline]
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Iterates over the registered paths.
    ///
    /// Before finalization the paths appear in insertion order; afterwards
    /// they are sorted.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|e| e.path.as_str())
    }

    /// Iterates over all files together with their ids.
    ///
    /// Returns nothing before finalization, because ids only exist afterwards.
    pub fn iter(&self) -> Iter<'_, Id> {
        Iter {
            files: self.files.iter().enumerate(),
            finalized: self.finalized,
            marker: core::marker::PhantomData,
        }
    }

    /// Returns the id assigned to `path`, or `None` for unknown paths and
    /// unfinalized maps.
    ///
    /// The lookup runs through a verified open-addressed index: one seeded
    /// hash over the queried path, then linear probing with hash-and-bytes
    /// verification.
    #[inline]
    pub fn get_id(&self, path: &str) -> Option<Id> {
        let index = self.lookup_index(path)?;
        Id::from_index(index)
    }

    /// Returns the path registered under `id`, or `None` for invalid ids and
    /// unfinalized maps.
    pub fn get_path(&self, id: Id) -> Option<&str> {
        if !self.finalized {
            return None;
        }
        let index = self.index_of(id)?;
        self.files.get(index).map(|e| e.path.as_str())
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
            path: entry.path.as_str(),
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
        let offsets = self.line_offsets_for(id)?;

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

    /// Returns the per-file line offset index for `id`, or `None` for
    /// unknown ids and unfinalized maps.
    ///
    /// Requires the `view` feature. This exposes the index behind
    /// [`view`](Self::view) so callers can run their own offset/line/column
    /// conversions against the exact index the views use, without
    /// re-scanning file contents.
    #[cfg(feature = "view")]
    #[cfg_attr(docsrs, doc(cfg(feature = "view")))]
    #[inline]
    pub fn line_offsets(&self, id: Id) -> Option<&LineOffsets> {
        self.line_offsets_for(id)
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

    /// Converts a byte offset into a one-based point position.
    ///
    /// The result is a zero-width span whose start and end are the
    /// `(line, column)` of the byte at `offset`. Offsets up to and including
    /// the content length resolve: the content length maps to the column one
    /// past the last byte, the conventional insertion anchor at the end of
    /// the file.
    ///
    /// Returns `None` for unknown ids, unfinalized maps, offsets beyond the
    /// content length, and offsets whose line or column cannot be
    /// represented by the packed position encoding (files of more than
    /// 65,535 lines or lines longer than 255 bytes).
    ///
    /// Requires the `view` feature. The conversion is O(log n) over the
    /// per-file line index; it allocates nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::{AbsolutePosition, SourceFilesMap};
    ///
    /// let mut files = SourceFilesMap::<u8>::new();
    /// files.add_file("a.txt", b"one\ntwo").expect("fits");
    /// files.finalize().expect("fits");
    ///
    /// let id = files.get_id("a.txt").expect("present");
    /// assert_eq!(
    ///     files.position_from_offset(id, 4),
    ///     Some(AbsolutePosition::new(id, 2, 1, 2, 1))
    /// );
    /// ```
    #[cfg(feature = "view")]
    #[cfg_attr(docsrs, doc(cfg(feature = "view")))]
    pub fn position_from_offset(&self, id: Id, offset: usize) -> Option<AbsolutePosition<Id>> {
        let (line, col) = self.line_offsets_for(id)?.line_col_from_offset(offset)?;
        // Lines longer than 255 bytes or files with more than 65,535 lines
        // have components the packed encoding cannot represent.
        let line = u16::try_from(line).ok()?;
        let col = u8::try_from(col).ok()?;
        Some(AbsolutePosition::new(id, line, col, line, col))
    }

    /// Converts the start of a position span into its byte offset.
    ///
    /// The end of the span is ignored: the returned offset anchors the
    /// position's `(start_line, start_column)`. Returns `None` for unknown
    /// ids, unfinalized maps, and spans whose start does not resolve to a
    /// byte inside or at the end of the file.
    ///
    /// Requires the `view` feature. The conversion is O(log n) over the
    /// per-file line index; it allocates nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::{AbsolutePosition, SourceFilesMap};
    ///
    /// let mut files = SourceFilesMap::<u8>::new();
    /// files.add_file("a.txt", b"one\ntwo").expect("fits");
    /// files.finalize().expect("fits");
    ///
    /// let id = files.get_id("a.txt").expect("present");
    /// let span = AbsolutePosition::new(id, 2, 1, 2, 3);
    /// assert_eq!(files.offset_from_position(id, &span), Some(4));
    /// ```
    #[cfg(feature = "view")]
    #[cfg_attr(docsrs, doc(cfg(feature = "view")))]
    pub fn offset_from_position(&self, id: Id, pos: &impl SourceFilePosition) -> Option<usize> {
        let offsets = self.line_offsets_for(id)?;
        offsets.offset_from_line_col(pos.start_line() as usize, pos.start_column() as usize)
    }

    /// Prepares a zero-allocation `path:line:column` rendering of the
    /// position's start, the conventional diagnostic anchor format.
    ///
    /// Returns `None` for unknown ids and unfinalized maps. The returned
    /// value borrows the file path and only formats when displayed, so no
    /// string is built until the value is rendered.
    ///
    /// # Examples
    ///
    /// ```
    /// use sourcier_core::{AbsolutePosition, SourceFilesMap};
    ///
    /// let mut files = SourceFilesMap::<u8>::new();
    /// files.add_file("src/main.rs", b"fn main() {}").expect("fits");
    /// files.finalize().expect("fits");
    ///
    /// let id = files.get_id("src/main.rs").expect("present");
    /// let pos = AbsolutePosition::new(id, 1, 4, 1, 7);
    /// assert_eq!(
    ///     files.display_position(id, &pos).expect("known id").to_string(),
    ///     "src/main.rs:1:4"
    /// );
    /// ```
    pub fn display_position(
        &self,
        id: Id,
        pos: &impl SourceFilePosition,
    ) -> Option<PositionDisplay<'_>> {
        let path = self.get_path(id)?;
        Some(PositionDisplay {
            path,
            line: pos.start_line(),
            column: pos.start_column(),
        })
    }

    /// Recomputes the per-file line offset indexes.
    ///
    /// Contents beyond the 4 GiB line offset limit of
    /// [`LineOffsets::MAX_CONTENT_LENGTH`](crate::line_map::LineOffsets) are
    /// not indexed; `view` on such files returns `None` like for any other
    /// unindexed file.
    #[cfg(feature = "view")]
    fn rebuild_line_offsets(&mut self) {
        // Ids only exist for the first `Id::MAX_FILES` entries; crafted
        // deserialized maps can carry more entries than that.
        let count = self.files.len().min(Id::MAX_FILES);
        self.line_offsets.clear();
        self.line_offsets.reserve(count);
        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            let computed: Vec<Option<LineOffsets>> = self.files[..count]
                .par_iter()
                .map(|entry| LineOffsets::compute(&entry.content).ok())
                .collect();
            self.line_offsets = computed;
        }
        #[cfg(not(feature = "parallel"))]
        self.line_offsets.extend(
            self.files[..count]
                .iter()
                .map(|entry| LineOffsets::compute(&entry.content).ok()),
        );
    }

    /// Returns the line offset index of `id`, or `None` for unknown ids and
    /// unfinalized maps.
    #[cfg(feature = "view")]
    #[inline]
    fn line_offsets_for(&self, id: Id) -> Option<&LineOffsets> {
        if !self.finalized {
            return None;
        }
        let index = self.index_of(id)?;
        self.line_offsets.get(index).and_then(|slot| slot.as_ref())
    }

    /// Resolves `path` to its zero-based entry index through the verified
    /// open-addressed index.
    #[inline]
    fn lookup_index(&self, path: &str) -> Option<usize> {
        if !self.finalized {
            return None;
        }
        let hash = hash_path(self.hash_seed, path.as_bytes());
        let mask = self.path_index.mask as usize;
        let mut slot = hash as usize & mask;
        loop {
            let probed = self.path_index.slots[slot];
            if probed == 0 {
                return None;
            }
            let index = probed as usize - 1;
            let entry = self.files.get(index)?;
            if entry.hash == hash && entry.path.as_bytes() == path.as_bytes() {
                return Some(index);
            }
            slot = (slot + 1) & mask;
        }
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

    /// Recomputes every piece of derived state (hashes, sequence numbers,
    /// byte totals, lookup index and line offsets) from the raw entries.
    #[cfg(feature = "serde")]
    fn rebuild_derived_state(&mut self) {
        let seed = self.hash_seed;
        for (index, entry) in self.files.iter_mut().enumerate() {
            entry.hash = hash_path(seed, entry.path.as_bytes());
            entry.seq = index as u32;
        }
        self.total_bytes = self.files.iter().map(|e| e.content.len()).sum();
        if self.finalized {
            self.path_index
                .rebuild(&self.files, self.files.len().min(Id::MAX_FILES));
            #[cfg(feature = "view")]
            self.rebuild_line_offsets();
        }
    }
}

impl<'a, Id: FileId> IntoIterator for &'a SourceFilesMap<Id> {
    type Item = FileRef<'a, Id>;
    type IntoIter = Iter<'a, Id>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use serde::de::Deserialize;

    use super::{Entry, FileId, PathIndex, SourceFilesMap};

    impl<'de, Id> Deserialize<'de> for SourceFilesMap<Id>
    where
        Id: FileId + serde::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            /// Wire format. Only the raw entries and the phase flag are
            /// persisted; lookup state (path index, line offsets, hashes)
            /// is derived and rebuilt, so payloads stay compact and can
            /// never carry inconsistent lookup tables. The v1 field
            /// `path_to_id` is accepted on read and ignored.
            #[derive(serde::Deserialize)]
            struct Repr {
                files: Vec<Entry>,
                finalized: bool,
            }

            let repr = Repr::deserialize(deserializer)?;
            let mut map = SourceFilesMap::<Id> {
                files: repr.files,
                path_index: PathIndex {
                    mask: 0,
                    slots: Vec::new(),
                },
                finalized: repr.finalized,
                total_bytes: 0,
                hash_seed: super::random_seed(),
                #[cfg(feature = "view")]
                line_offsets: Vec::new(),
                #[cfg(feature = "rt-feedback")]
                feedback: None,
                marker: core::marker::PhantomData,
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
    fn duplicate_paths_keep_the_first_content_at_scale() {
        // Hundreds of files with scattered duplicates exercise the seq
        // tiebreak of the sort: the first registration must win no matter
        // how the unstable sort permutes equal paths.
        let mut files = SourceFilesMap::<u16>::new();
        for index in 0..400 {
            let slot = index % 25;
            files
                .add_file(format!("dup_{slot:02}.txt"), format!("content {index}"))
                .expect("u16 capacity");
        }
        files.finalize().expect("fits");
        assert_eq!(files.len(), 25);
        for slot in 0..25 {
            let expected = format!("content {slot}");
            let id = files
                .get_id(&format!("dup_{slot:02}.txt"))
                .expect("present");
            assert_eq!(
                files.get_content(id),
                Some(expected.as_bytes()),
                "first registration wins"
            );
        }
    }

    #[test]
    fn path_index_holds_one_slot_per_unique_path() {
        let files = sample();
        assert_eq!(files.path_index.len(), 2);

        // Duplicated paths collapse to a single slot.
        let mut dups = SourceFilesMap::<u8>::new();
        dups.add_file("x.txt", b"1").expect("fits");
        dups.add_file("x.txt", b"2").expect("fits");
        dups.finalize().expect("fits");
        assert_eq!(dups.path_index.len(), 1);

        // An empty map builds the minimum table without any slot.
        let mut empty = SourceFilesMap::<u8>::new();
        empty.finalize().expect("fits");
        assert_eq!(empty.path_index.len(), 0);
        assert_eq!(empty.get_id("anything"), None);
    }

    #[test]
    fn total_bytes_survives_dedup_and_matches_contents() {
        let mut files = SourceFilesMap::<u8>::new();
        files.add_file("dup.txt", b"first").expect("fits");
        files.add_file("dup.txt", b"second-longer").expect("fits");
        files.add_file("solo.txt", b"abc").expect("fits");
        assert_eq!(files.total_bytes(), 21);
        files.finalize().expect("fits");
        // Only the surviving first registration counts.
        assert_eq!(files.total_bytes(), 8);
        let counted: usize = files.iter().map(|f| f.content.len()).sum();
        assert_eq!(files.total_bytes(), counted);
    }

    #[test]
    fn hash_path_changes_with_the_seed() {
        let a = hash_path(1, b"src/main.rs");
        let b = hash_path(2, b"src/main.rs");
        assert_ne!(a, b, "seeds must decorrelate maps");
        // Same seed is deterministic and handles short and multi-chunk paths.
        assert_eq!(hash_path(7, b""), hash_path(7, b""));
        assert_eq!(hash_path(7, b"a"), hash_path(7, b"a"));
        let long = "src/some/quite/long/module/path/that/spans/many/chunks.rs";
        assert_eq!(hash_path(7, long.as_bytes()), hash_path(7, long.as_bytes()));
    }

    #[test]
    fn long_paths_take_the_heap_variant_and_stay_exact() {
        let short = "src/lib.rs";
        let long = "a/very/deeply/nested/source/tree/that/exceeds/the/inline/budget/module.rs";
        assert!(short.len() <= 30);
        assert!(long.len() > 30);

        let mut files = SourceFilesMap::<u8>::new();
        files.add_file(long, b"heap").expect("fits");
        files.add_file(short, b"inline").expect("fits");
        files.finalize().expect("fits");

        // Both path kinds resolve exactly, in sorted path order.
        let id_long = files.get_id(long).expect("heap path indexed");
        let id_short = files.get_id(short).expect("inline path indexed");
        assert!(u64::from(id_long) < u64::from(id_short), "sorted by path");
        assert_eq!(files.get_path(id_long), Some(long));
        assert_eq!(files.get_content(id_long), Some(&b"heap"[..]));
        assert_eq!(files.get_path(id_short), Some(short));

        // Unknown paths still miss.
        assert_eq!(files.get_id("src/lib.rs/"), None);

        // The ordering impls compare by bytes, exactly like `str`.
        let a = Path::new(short);
        let b = Path::new(long);
        assert_eq!(a.partial_cmp(&b), Some(core::cmp::Ordering::Greater));
        assert_eq!(a.cmp(&b), core::cmp::Ordering::Greater);
        assert!(a > b);
        assert!(b < a);
        assert_eq!(a, Path::new(short));
    }

    #[test]
    #[cfg(feature = "view")]
    fn line_offsets_accessor_fails_before_finalization() {
        let mut files = SourceFilesMap::<u8>::new();
        files.add_file("a.txt", b"alpha\nbeta").expect("fits");
        assert_eq!(files.line_offsets(1), None);
        files.finalize().expect("fits");
        assert!(files.line_offsets(1).is_some());
    }

    #[test]
    fn probe_distribution_diagnostic() {
        // Measures the average probe length of successful lookups over 2000
        // realistic paths (kept as a diagnostic; assertions are loose).
        let paths: Vec<String> = (0..2_000)
            .map(|file| format!("src/module_{file:04}.rs"))
            .collect();
        let mut map = SourceFilesMap::<u16>::new();
        for path in &paths {
            map.add_file(path.as_str(), "x").expect("fits");
        }
        map.finalize().expect("fits");

        let seed = map.hash_seed;
        let mask = map.path_index.mask as usize;
        let slots = &map.path_index.slots;
        let files = &map.files;
        let mut total_probes = 0_usize;
        for path in &paths {
            let hash = hash_path(seed, path.as_bytes());
            let mut slot = hash as usize & mask;
            let mut probes = 1;
            loop {
                let occupied = slots[slot];
                let found = occupied != 0 && {
                    let index = occupied as usize - 1;
                    files[index].hash == hash && files[index].path.as_bytes() == path.as_bytes()
                };
                if found {
                    break;
                }
                slot = (slot + 1) & mask;
                probes += 1;
                assert!(probes <= slots.len(), "probe run must terminate");
            }
            total_probes += probes;
        }
        let average = total_probes as f64 / paths.len() as f64;
        println!(
            "average probes per successful lookup: {average:.2} (load factor {:.2})",
            files.len() as f64 / slots.len() as f64
        );
        assert!(average < 2.5, "hash distribution degraded: {average:.2}");
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
