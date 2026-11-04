//! The type-state build phase: registering files before finalization.
//!
//! [`SourceFilesBuilder`] wraps a
//! [`SourceFilesMap`] during its build phase and exposes only the
//! registration API. The phase errors of the raw map
//! ([`MapError::Finalized`](crate::MapError) and
//! [`MapError::AlreadyFinalized`](crate::MapError)) are unrepresentable here,
//! and [`finalize`](SourceFilesBuilder::finalize) hands back a map that is
//! already in its immutable lookup phase.

use crate::error::MapError;
use crate::file_id::FileId;
use crate::map::SourceFilesMap;

#[cfg(feature = "rt-feedback")]
use std::sync::Arc;

#[cfg(feature = "rt-feedback")]
use crate::feedback::RuntimeFeedback;

/// Chainable build-phase companion of [`SourceFilesMap`].
///
/// The builder owns a map in its build phase and only exposes registration,
/// so the failure modes that the raw map reports at runtime — adding after
/// finalization, querying before ids exist — cannot be expressed. When the
/// build is complete, [`finalize`](Self::finalize) consumes the builder and
/// returns the finished, lookup-ready map.
///
/// # Examples
///
/// ```
/// use sourcier_core::SourceFilesBuilder;
///
/// let files = SourceFilesBuilder::<u8>::new()
///     .add_file("src/main.rs", b"fn main() {}")
///     .expect("capacity available")
///     .add_file("src/lib.rs", b"pub mod utils;")
///     .expect("capacity available")
///     .finalize()
///     .expect("no capacity issue");
///
/// let id = files.get_id("src/main.rs").expect("present");
/// assert_eq!(files.get_path(id), Some("src/main.rs"));
/// ```
///
/// Loops should prefer the mutating [`add_file_mut`](Self::add_file_mut):
///
/// ```
/// use sourcier_core::SourceFilesBuilder;
///
/// let mut builder = SourceFilesBuilder::<u16>::with_capacity(3);
/// for i in 0..3 {
///     builder
///         .add_file_mut(format!("f{i}.rs"), format!("content {i}"))
///         .expect("u16 capacity");
/// }
/// let files = builder.finalize().expect("fits");
/// assert_eq!(files.len(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct SourceFilesBuilder<Id: FileId> {
    map: SourceFilesMap<Id>,
}

impl<Id: FileId> Default for SourceFilesBuilder<Id> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: FileId> SourceFilesBuilder<Id> {
    /// Creates an empty builder without pre-reserved storage.
    #[inline]
    pub fn new() -> Self {
        Self {
            map: SourceFilesMap::new(),
        }
    }

    /// Creates an empty builder that pre-reserves storage for
    /// `expected_files` files, reducing reallocations while building.
    #[inline]
    pub fn with_capacity(expected_files: usize) -> Self {
        Self {
            map: SourceFilesMap::with_capacity(expected_files),
        }
    }

    /// Creates an empty builder that reports its finalization statistics
    /// into the shared `feedback` context.
    ///
    /// Requires the `rt-feedback` feature.
    #[cfg(feature = "rt-feedback")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-feedback")))]
    #[inline]
    pub fn with_feedback(feedback: Arc<RuntimeFeedback>) -> Self {
        Self {
            map: SourceFilesMap::with_feedback(feedback),
        }
    }

    /// Registers a file, returning the builder for chaining.
    ///
    /// The path and content semantics are exactly those of
    /// [`SourceFilesMap::add_file`](crate::SourceFilesMap::add_file):
    /// duplicates are allowed and resolved during
    /// [`finalize`](Self::finalize) with the first registration winning, and
    /// the only failure is an exhausted id type.
    ///
    /// On failure the builder is consumed; loops that want to keep working
    /// with the builder after an error should use
    /// [`add_file_mut`](Self::add_file_mut) instead.
    #[inline]
    pub fn add_file(
        mut self,
        path: impl AsRef<str>,
        content: impl Into<Vec<u8>>,
    ) -> Result<Self, MapError> {
        self.map.add_file(path, content)?;
        Ok(self)
    }

    /// Registers a file in place, for registration loops.
    ///
    /// Behaves like [`add_file`](Self::add_file) but borrows the builder, so
    /// the builder stays usable after a failed registration.
    #[inline]
    pub fn add_file_mut(
        &mut self,
        path: impl AsRef<str>,
        content: impl Into<Vec<u8>>,
    ) -> Result<(), MapError> {
        self.map.add_file(path, content)
    }

    /// Registers a batch of files, returning the builder for chaining.
    ///
    /// The batch is all-or-nothing like
    /// [`SourceFilesMap::add_files`](crate::SourceFilesMap::add_files); on
    /// failure the builder is consumed with no files of the batch
    /// registered.
    #[inline]
    pub fn add_files<I, P, C>(mut self, files: I) -> Result<Self, MapError>
    where
        I: IntoIterator<Item = (P, C)>,
        P: AsRef<str>,
        C: Into<Vec<u8>>,
    {
        self.map.add_files(files)?;
        Ok(self)
    }

    /// Registers a batch of files in place.
    ///
    /// Behaves like [`add_files`](Self::add_files) but borrows the builder,
    /// so the builder stays usable after a failed batch.
    #[inline]
    pub fn add_files_mut<I, P, C>(&mut self, files: I) -> Result<(), MapError>
    where
        I: IntoIterator<Item = (P, C)>,
        P: AsRef<str>,
        C: Into<Vec<u8>>,
    {
        self.map.add_files(files)
    }

    /// Registers a file from a [`Path`]-like value, returning the builder
    /// for chaining.
    ///
    /// Paths that are not valid UTF-8 fail with
    /// [`MapError::PathNotUtf8`] and nothing is registered; see
    /// [`SourceFilesMap::add_path`](crate::SourceFilesMap::add_path). On
    /// failure the builder is consumed; loops should prefer
    /// [`add_path_mut`](Self::add_path_mut).
    ///
    /// [`Path`]: std::path::Path
    #[inline]
    pub fn add_path(
        mut self,
        path: impl AsRef<std::path::Path>,
        content: impl Into<Vec<u8>>,
    ) -> Result<Self, MapError> {
        self.map.add_path(path, content)?;
        Ok(self)
    }

    /// Registers a file from a [`Path`]-like value in place.
    ///
    /// Behaves like [`add_path`](Self::add_path) but borrows the builder, so
    /// the builder stays usable after a failed registration.
    ///
    /// [`Path`]: std::path::Path
    #[inline]
    pub fn add_path_mut(
        &mut self,
        path: impl AsRef<std::path::Path>,
        content: impl Into<Vec<u8>>,
    ) -> Result<(), MapError> {
        self.map.add_path(path, content)
    }

    /// Returns the number of files registered so far.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` when no file is registered yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns how many more files the id type can address.
    ///
    /// Registering beyond this count fails with
    /// [`MapError::TooManyFiles`]; a count of `0` means the id type is
    /// exhausted and a wider id type (`u16`) is required.
    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        Id::MAX_FILES.saturating_sub(self.map.len())
    }

    /// Iterates over the paths registered so far, in registration order.
    #[inline]
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.map.paths()
    }

    /// Consumes the builder and returns the finalized, lookup-ready map.
    ///
    /// The result answers queries in constant time exactly like a map
    /// finalized by hand; see
    /// [`SourceFilesMap::finalize`](crate::SourceFilesMap::finalize) for the
    /// id assignment rules.
    #[inline]
    pub fn finalize(self) -> Result<SourceFilesMap<Id>, MapError> {
        let mut map = self.map;
        map.finalize()?;
        Ok(map)
    }
}
