# Source File Mapping (SFM) Library

⚠️ **WARNING**: This library is a Work in Progress (WIP) and is not production-ready. Expect breaking changes and API instability.

## Overview

A Rust library for compact and efficient source file mapping and position tracking. Designed to support source code analysis, debugging tools, and development environments.

## Features

- Compact file ID representation
- Absolute and relative source position tracking
- Flexible file ID types (supports `u8` and `u16`)
- Optional runtime feedback
- Source code view capabilities

## Current Capabilities

### Position Tracking
- Create absolute positions with file references
- Create relative positions without file context
- Support for line and column tracking

### File Mapping
- Add files with byte content
- Map file paths to numeric IDs
- Retrieve file contents by ID

## Usage Example

```rust
use sfm::{SourceFilesMap, create_absolute_position};

// Create a file map with u8 file IDs
let mut files = SourceFilesMap::<u8>::new();

// Add files
files.add_file("src/main.rs".to_string(), Vec::new());
files.finalize().unwrap();

// Get file ID
let file_id = files.get_id("src/main.rs").unwrap();

// Create an absolute position
let position = create_absolute_position(file_id, 10, 5, 12, 20);
```

## Supported File ID Types

- `u8`: Supports up to 255 files
- `u16`: Supports up to 65,535 files

## Optional Features

- `rt-feedback`: Runtime usage tracking
- `view`: Source code viewing capabilities

## Performance Notes

- Compact memory representation
- Efficient line and column tracking
- Minimal overhead for source position management

## Roadmap

- [ ] Stabilize API
- [ ] Comprehensive documentation
- [ ] More extensive testing
- [ ] Performance benchmarks
- [ ] Additional file ID type support

## Installation

![Crates.io](https://img.shields.io/badge/Crates.io-Not%20Published-red)

🚨 **Important Notice**

This library is currently **not available on crates.io**.

To use this library, you'll need to reference it directly from the GitHub repository:

```toml
[dependencies]
sfm = { git = "https://github.com/your-username/sfm", branch = "main" }
```

⚠️ Warning: This is a work-in-progress library and is not recommended for production use. Expect frequent breaking changes.

## Contributing

Contributions are welcome! Please be aware that the library is in early stages and the API is likely to change significantly.

## License

[To be determined - specify your license]

## Disclaimer

This library is experimental. Do not use in production environments until further notice.
