use std::collections::HashMap;
use std::convert::TryInto;

use crate::fid::FileId;

/// Manages mapping between file paths and their deterministic IDs
#[derive(Debug, Clone)]
pub struct SourceFilesMap<Id: FileId> {
    paths: Vec<String>,
    path_to_id: HashMap<String, Id>,
}

impl<Id: FileId> SourceFilesMap<Id> {
    /// Create a new empty file map
    pub fn new() -> Self {
        Self {
            paths: Vec::with_capacity(Id::MAX_FILES),
            path_to_id: HashMap::with_capacity(Id::MAX_FILES),
        }
    }

    /// Add a file path to the map (before finalization)
    pub fn add_file(&mut self, path: String) {
        if self.paths.len() < Id::MAX_FILES {
            self.paths.push(path);
        }
    }

    /// Finalize the map with deterministic ID assignment
    pub fn finalize(&mut self) -> Result<(), String> {
        // Deduplicate and sort paths for deterministic IDs
        self.paths.sort_unstable();
        self.paths.dedup();

        if self.paths.len() > Id::MAX_FILES {
            return Err(format!(
                "Exceeded maximum of {} files for ID type",
                Id::MAX_FILES
            ));
        }

        self.path_to_id.clear();
        for (idx, path) in self.paths.iter().enumerate() {
            let id = (idx + 1) as u64; // ID 0 is reserved for relative positions
            let id = id.try_into().map_err(|_| "ID conversion failed")?;
            self.path_to_id.insert(path.clone(), id);
        }

        Ok(())
    }

    /// Get file ID for a path (returns None for unknown files)
    pub fn get_id(&self, path: &str) -> Option<Id> {
        self.path_to_id.get(path).copied()
    }

    /// Get file path for an ID (returns None for invalid IDs)
    pub fn get_path(&self, id: Id) -> Option<&str> {
        let raw_id: u64 = id.into();
        let index = (raw_id - 1) as usize;
        self.paths.get(index).map(|s| s.as_str())
    }

    /// Get total number of registered files
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}
