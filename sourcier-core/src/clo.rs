// Compact line offset representation
#[derive(Debug, Clone)]
pub struct CompactLineOffsets {
    // Store offsets as u32 to save memory
    offsets: Vec<u32>,
    content_length: usize,
}

impl CompactLineOffsets {
    // Precompute line offsets more efficiently
    pub fn compute(content: &[u8]) -> Self {
        let mut offsets = vec![0u32];
        let content_length = content.len();

        // Use memchr for faster line break detection
        let mut current_pos = 0;
        while let Some(newline_pos) = memchr::memchr(b'\n', &content[current_pos..]) {
            current_pos += newline_pos + 1;
            offsets.push(current_pos as u32);
        }

        Self {
            offsets,
            content_length,
        }
    }

    // More efficient line lookup
    pub fn get_line_range(&self, line: usize) -> Option<(usize, usize)> {
        if line == 0 || line > self.offsets.len() {
            return None;
        }

        let start = self.offsets[line - 1] as usize;
        let end = if line < self.offsets.len() {
            self.offsets[line] as usize - 1
        } else {
            self.content_length
        };

        Some((start, end))
    }
}
