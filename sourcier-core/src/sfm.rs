use std::collections::HashMap;
use std::convert::TryInto;

use crate::fid::FileId;

/// Manages mapping between file paths and their deterministic IDs
#[derive(Debug, Clone)]
pub struct SourceFilesMap<Id: FileId> {
    files: Vec<FileEntry>,
    path_to_id: HashMap<String, Id>,
    avg_file_size: usize,
    expected_files: usize,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: String,
    content: Vec<u8>,
}
impl<Id: FileId> SourceFilesMap<Id> {
    /// Create a new map with conservative defaults for small projects
    pub fn new() -> Self {
        // Default heuristics: 100 files @ 2KB average
        const DEFAULT_FILE_COUNT: usize = 100;
        const DEFAULT_AVG_SIZE: usize = 2048;

        Self {
            files: Vec::with_capacity(DEFAULT_FILE_COUNT),
            path_to_id: HashMap::with_capacity(DEFAULT_FILE_COUNT),
            avg_file_size: DEFAULT_AVG_SIZE,
            expected_files: DEFAULT_FILE_COUNT,
        }
    }
    /// Create with custom preallocation heuristics
    pub fn with_heuristics(expected_files: usize, avg_file_size: usize) -> Self {
        Self {
            files: Vec::with_capacity(expected_files),
            path_to_id: HashMap::with_capacity(expected_files),
            avg_file_size,
            expected_files,
        }
    }

    /// Add a file with content (bytes preferred over String)
    pub fn add_file(&mut self, path: String, content: Vec<u8>) {
        if self.files.len() < Id::MAX_FILES {
            self.files.push(FileEntry { path, content });
        }
    }

    /// Finalize with path-based sorting and deduplication
    pub fn finalize(&mut self) -> Result<(), String> {
        // Sort by path first, then by content size for potential grouping
        self.files.sort_unstable_by(|a, b| {
            a.path
                .cmp(&b.path)
                .then_with(|| a.content.len().cmp(&b.content.len()))
        });

        // Deduplicate paths while keeping first occurrence
        self.files.dedup_by(|a, b| a.path == b.path);

        // Check capacity constraints
        if self.files.len() > Id::MAX_FILES {
            return Err(format!(
                "Exceeded maximum of {} files for ID type",
                Id::MAX_FILES
            ));
        }

        // Preallocate content storage in bulk (heuristic-based)
        let total_bytes = self.avg_file_size * self.expected_files;
        let mut consolidated = Vec::with_capacity(total_bytes);

        // Build ID mapping and consolidate memory
        self.path_to_id.clear();
        for (idx, entry) in self.files.iter_mut().enumerate() {
            // Move content to consolidated storage
            consolidated.extend_from_slice(&entry.content);

            // Store ID mapping
            let id = (idx + 1) as u64;
            let id = id.try_into().map_err(|_| "ID conversion failed")?;
            self.path_to_id.insert(entry.path.clone(), id);
        }

        // Replace individual content vectors with slices into consolidated storage
        let mut offset = 0;
        for entry in &mut self.files {
            let len = entry.content.len();
            entry.content = consolidated[offset..offset + len].to_vec();
            offset += len;
        }

        Ok(())
    }

    /// Get immutable view of file content
    pub fn get_content(&self, id: Id) -> Option<&[u8]> {
        let raw_id: u64 = id.into();
        let index = (raw_id - 1) as usize;
        self.files.get(index).map(|e| e.content.as_slice())
    }

    /// Get file ID for a path (returns None for unknown files)
    pub fn get_id(&self, path: &str) -> Option<Id> {
        self.path_to_id.get(path).copied()
    }

    /// Get file path for an ID (returns None for invalid IDs)
    pub fn get_path(&self, id: Id) -> Option<&str> {
        let raw_id: u64 = id.into();
        let index = (raw_id - 1) as usize;
        self.files.get(index).map(|s| s.path.as_str())
    }

    /// Get total number of registered files
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}
