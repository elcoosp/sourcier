use std::collections::HashMap;
use std::convert::TryInto;

use crate::fid::FileId;

#[cfg(feature = "rt-feedback")]
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct SourceFilesMap<Id: FileId> {
    files: Vec<FileEntry>,
    path_to_id: HashMap<String, Id>,
    avg_file_size: usize,
    expected_files: usize,

    // Feature-gated feedback state
    #[cfg(feature = "rt-feedback")]
    feedback: Option<Arc<Mutex<RuntimeFeedback>>>,
}

#[cfg(feature = "rt-feedback")]
#[derive(Debug, Default)]
pub struct RuntimeFeedback {
    pub total_files: usize,
    pub total_bytes: u64,
    pub max_file_size: usize,
    pub usage_count: u32,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: String,
    content: Vec<u8>,
}
impl<Id: FileId> SourceFilesMap<Id> {
    /// Create a new map with conservative defaults for small projects
    #[cfg(not(feature = "rt-feedback"))]
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
    /// Create new instance with optional feedback context
    #[cfg(feature = "rt-feedback")]
    pub fn with_feedback(feedback: Option<Arc<Mutex<RuntimeFeedback>>>) -> Self {
        let (expected, avg_size) = feedback.as_ref().map_or_else(
            || (Self::DEFAULT_FILE_COUNT, Self::DEFAULT_AVG_SIZE), // Defaults
            |f| {
                let data = f.lock().unwrap();
                let expected = (data.total_files * 120) / 100; // 20% buffer
                let avg_size = if data.total_files > 0 {
                    (data.total_bytes / data.total_files as u64) as usize
                } else {
                    Self::DEFAULT_AVG_SIZE
                };
                (expected, avg_size)
            },
        );

        Self {
            files: Vec::with_capacity(expected),
            path_to_id: HashMap::with_capacity(expected),
            avg_file_size: avg_size,
            expected_files: expected,
            feedback,
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
        #[cfg(feature = "rt-feedback")]
        if let Some(feedback) = &self.feedback {
            let total_bytes = self.files.iter().map(|e| e.content.len() as u64).sum();

            let max_size = self
                .files
                .iter()
                .map(|e| e.content.len())
                .max()
                .unwrap_or(0);

            let mut data = feedback.lock().unwrap();
            data.total_files = self.files.len();
            data.total_bytes = total_bytes;
            data.max_file_size = max_size;
            data.usage_count += 1;
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
