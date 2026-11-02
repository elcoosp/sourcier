<div align="center">
  <img src="docs/logo.png" alt="sourcier Logo" width="300"/>
  <p>
    <strong>Compact and efficient source file mapping and position tracking, written in Rust.</strong><br/>
    Register source files once, hand out stable numeric ids and pack every span — file id, line and column — into a single <code>u64</code>. Look paths up in O(1), navigate files byte-accurately, and keep diagnostics positions cheap enough to store by the million. Built for compilers, linters, formatters, language servers and code editors.
  </p>
  <p>
    <img src="https://img.shields.io/badge/Rust-1.99%2B%20%7C%20Edition%202024-000000?style=flat-square&logo=rust" alt="Rust"/>
    <img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License MIT"/>
    <img src="https://img.shields.io/crates/v/sourcier?style=flat-square" alt="Crates.io"/>
    <img src="https://img.shields.io/docsrs/sourcier?style=flat-square" alt="docs.rs"/>
    <img src="https://img.shields.io/badge/Crates-2-6F4E37?style=flat-square" alt="Crates"/>
    <img src="https://img.shields.io/badge/Spans-one%20u64-FF4500?style=flat-square" alt="Spans"/>
    <img src="https://img.shields.io/badge/File%20Ids-u8%20%7C%20u16-4B32C3?style=flat-square" alt="File Ids"/>
    <img src="https://img.shields.io/badge/Features-serde%20%7C%20view%20%7C%20rt--feedback-00BFFF?style=flat-square" alt="Features"/>
    <img src="https://img.shields.io/badge/Unsafe-forbidden-8B0000?style=flat-square" alt="Unsafe"/>
    <img src="https://img.shields.io/badge/Tests-cargo%20nextest-007ACC?style=flat-square" alt="Tests"/>
    <img src="https://img.shields.io/badge/Coverage-100%25%20functions-brightgreen?style=flat-square" alt="Coverage"/>
    <img src="https://img.shields.io/badge/Build-Passing-brightgreen?style=flat-square&logo=githubactions" alt="Build"/>
  </p>
</div>

---

`sourcier` solves a small problem that diagnostics-heavy tools hit every day: positions must be cheap to build, compare, store and serialize, yet still resolve back to the exact bytes of the file they point at. Instead of dragging `String` paths and four `usize` fields through every diagnostic, `sourcier` packs the whole span into one `u64` and keeps the heavy data in a single immutable registry.

## Highlights

- **One `u64` per span.** File id, start line, start column, end line and end column are bit-packed together. Positions are `Copy`, cache-friendly, order naturally as integers and survive serialization round trips bit-exactly.
- **Constant-time lookups.** After finalization, path to id, id to path and id to content are all O(1); line and column navigation is O(log n).
- **Deterministic ids.** Finalizing sorts files by path, deduplicates registrations and assigns one-based ids, so the same set of files always produces the same ids.
- **Byte-accurate views.** With the `view` feature, a position resolves to the exact bytes it covers — including multi-line spans, line terminators and zero-width spans.
- **Typed, sealed API.** The `FileId` trait is sealed (`u8` and `u16` only), error enums are `#[non_exhaustive]`, and the packed layouts are documented so raw values can be persisted safely.
- **No unsafe code.** The whole workspace is `#![forbid(unsafe_code)]`.

## Installation

```toml
[dependencies]
sourcier = "1.0"
```

Or track the repository directly:

```toml
[dependencies]
sourcier = { git = "https://github.com/elcoosp/sourcier", branch = "main" }
```

> [!TIP]
> You only ever depend on `sourcier`. It re-exports the full API of the
> `sourcier-core` engine crate, which exists so the implementation can stay
> small and the facade stable.

## Quick start

```rust
use sourcier::{AbsolutePosition, SourceFilePosition, SourceFilesMap};

// 1. Register files (u8 ids support up to 255 files, u16 up to 65,535).
let mut files = SourceFilesMap::<u8>::new();
files
    .add_file("src/main.rs", b"fn main() {\n    println!(\"Hello\");\n}")
    .expect("capacity available");
files.add_file("src/lib.rs", b"pub mod utils;").expect("capacity available");

// 2. Finalize: sorts by path, deduplicates, assigns one-based ids.
files.finalize().expect("no capacity issue");

// 3. Look up files and pack positions into a single u64.
let id = files.get_id("src/main.rs").expect("file present");
assert_eq!(files.get_path(id), Some("src/main.rs"));

let pos = AbsolutePosition::new(id, 2, 5, 2, 22);
assert_eq!(pos.start_line(), 2);

// 4. With the default `view` feature, extract the exact bytes of a span.
assert_eq!(files.view(id, &pos), Some(&b"println!(\"Hello\");"[..]));
```

### Checked constructors

`AbsolutePosition::new` encodes values as-is — infallible and zero-cost. Use `try_new` when a span should be validated as a well-formed, one-based region:

```rust
use sourcier::AbsolutePosition;

assert!(AbsolutePosition::<u8>::try_new(1, 2, 1, 1, 1).is_err());
assert!(AbsolutePosition::<u8>::try_new(1, 1, 1, 1, 1).is_ok());
```

### Bit layout

Positions are stable `u64` encodings. The table shows the bit range of each field, most significant first:

| Id type | File id  | Start line | Start col | End line | End col |
| ------- | -------- | ---------- | --------- | -------- | ------- |
| `u8`    | `56..64` | `40..56`   | `32..40`  | `16..32` | `0..8`  |
| `u16`   | `48..64` | `32..48`   | `24..32`  | `8..24`  | `0..8`  |

`RelativePosition` stores `start line (32..48) | start col (24..32) | end line (8..24) | end col (0..8)` without a file id. Ids are one-based: `0` is never assigned, which keeps the all-zero encoding reserved.

> [!WARNING]
> Lines are `u16` (up to 65,535) and columns are `u8` (1 to 255, byte
> columns). The `view` feature indexes files of up to 4 GiB via compact
> `u32` offsets; larger files simply resolve no views.

## Feature flags

All features are enabled by default and forwarded to `sourcier-core`.

| Feature       | Default | Description                                                     |
| ------------- | ------- | --------------------------------------------------------------- |
| `serde`       | yes     | `Serialize`/`Deserialize` for maps, positions and line indexes. |
| `view`        | yes     | Line offset indexes plus `view`/`view_str` byte extraction.     |
| `rt-feedback` | yes     | Lock-free shared usage statistics (`RuntimeFeedback`).          |

Trim them for a leaner build:

```toml
[dependencies]
sourcier = { version = "1.0", default-features = false, features = ["view"] }
```

## API overview

| Type / trait                      | Purpose                                                         |
| --------------------------------- | --------------------------------------------------------------- |
| `SourceFilesMap<Id>`              | Build phase: register files. Lookup phase: query by path or id. |
| `FileId`                          | Sealed trait implemented by `u8` and `u16`.                     |
| `AbsolutePosition<Id>`            | Span bound to a file, packed into one `u64`.                    |
| `RelativePosition`                | File-agnostic span packed into the low 48 bits of a `u64`.      |
| `SourceFilePosition`              | Uniform accessor trait over both position kinds.                |
| `LineOffsets` (`view`)            | Compact per-line byte offset index with line/col mapping.       |
| `RuntimeFeedback` (`rt-feedback`) | Atomic counters shared across maps via `Arc`.                   |
| `MapError`, `PositionError`       | Typed, `non_exhaustive` error enums.                            |

## Performance

Measured with the bundled criterion suite (`cargo bench -p sourcier-core`); treat the numbers as indicative:

| Scenario                                | Time    |
| --------------------------------------- | ------- |
| Finalize 2,000 files x 200 lines        | ~5.6 ms |
| 2,000 path-to-id lookups (one per file) | ~34 us  |
| 1,000 multi-line `view` extractions     | ~13 us  |
| Line offsets for 100 files x 1,000 lines| ~0.8 ms |

Design notes:

- Finalization is a single `sort_unstable` plus one O(n) pass; file contents are never copied during finalization.
- Line offsets use `memchr`-accelerated newline scanning and 4 bytes per line (`u32` offsets, files up to 4 GiB).
- Path and content storage use `Box<str>`/`Box<[u8]>` to keep per-file overhead minimal.
- `rt-feedback` uses atomics only; no locks are taken on the reporting path.

## Testing and verification

The workspace ships a layered test suite — unit, integration, property-based (`proptest`), snapshot (`insta`) and doc tests. Coverage is measured with `cargo-llvm-cov`: 100% of functions are covered, and `llvm-cov show` confirms every line and every reachable region executes. The summary table reports 99.9% for lines and regions; the residual counts are uncallable compiler artifacts (definition-site copies of generic code), not missing tests:

```sh
cargo test --workspace --all-features      # unit, integration, property, doc tests
cargo nextest run --workspace --all-features
cargo llvm-cov --workspace --all-features --summary-only
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo deny check                           # licenses, advisories, bans, sources
cargo bench -p sourcier-core               # criterion benchmarks
```

Snapshot tests live in `sourcier-core/src/tests/snapshots/` and are committed to the repository; regenerate them after an intentional change with `INSTA_UPDATE=always cargo test`.

> [!NOTE]
> Property tests pin the round trips (position encoding, line/col to offset
> inverse mapping, map lookup consistency) against hundreds of generated
> cases, including adversarial deserialized maps that exceed the id capacity.
> The 4 GiB line-offset limit is exercised end to end with lazily committed
> allocations, so the oversized-file paths are tested for real.

## Minimum supported Rust version

Rust 1.99. The MSRV is verified in CI with `cargo check` across feature configurations.

## Semver policy

This release is 1.0.0: the public API documented on docs.rs is stable and follows semantic versioning. Error enums are `#[non_exhaustive]`, the `FileId` trait is sealed, and the packed `u64` layouts are documented so raw values can be persisted.
