//! Benchmarks for the core source files map operations.
//!
//! Run with `cargo bench -p sourcier-core`.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
#[cfg(feature = "view")]
use sourcier_core::AbsolutePosition;
use sourcier_core::SourceFilesMap;

/// Builds `n` synthetic source files with realistic line structure.
fn build_files(n: usize, lines_per_file: usize) -> Vec<(String, Vec<u8>)> {
    (0..n)
        .map(|file| {
            let content: Vec<u8> = (0..lines_per_file)
                .flat_map(|line| {
                    format!("fn item_{file}_{line}() {{ let x = {line}; }}\n").into_bytes()
                })
                .collect();
            (format!("src/module_{file:04}.rs"), content)
        })
        .collect()
}

/// Builds a finalized map over `entries`.
fn build_map(entries: &[(String, Vec<u8>)]) -> SourceFilesMap<u16> {
    let mut map = SourceFilesMap::<u16>::with_capacity(entries.len());
    for (path, content) in entries {
        map.add_file(path.as_str(), content.as_slice())
            .expect("fits");
    }
    map.finalize().expect("fits");
    map
}

fn bench_finalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("finalize");
    for (files, lines) in [(100_usize, 50_usize), (500, 100), (2_000, 200)] {
        let entries = build_files(files, lines);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{files}_files_{lines}_lines")),
            &entries,
            |b, entries| {
                b.iter(|| {
                    let mut map = SourceFilesMap::<u16>::new();
                    for (path, content) in entries {
                        map.add_file(path.as_str(), content.as_slice())
                            .expect("fits");
                    }
                    map.finalize().expect("fits");
                    black_box(map);
                });
            },
        );
    }
    group.finish();
}

fn bench_get_id(c: &mut Criterion) {
    let entries = build_files(2_000, 100);
    let map = build_map(&entries);
    let paths: Vec<&str> = entries.iter().map(|(p, _)| p.as_str()).collect();

    c.bench_function("get_id/2000_files", |b| {
        b.iter(|| {
            for path in black_box(&paths) {
                black_box(map.get_id(path).expect("present"));
            }
        });
    });

    // Every lookup misses: probes to the first empty slot of each bucket.
    let misses: Vec<String> = paths.iter().map(|path| format!("{path}.miss")).collect();
    let misses: Vec<&str> = misses.iter().map(String::as_str).collect();
    c.bench_function("get_id/2000_misses", |b| {
        b.iter(|| {
            for path in black_box(&misses) {
                black_box(map.get_id(path));
            }
        });
    });
}

fn bench_view(c: &mut Criterion) {
    #[cfg(feature = "view")]
    {
        let entries = build_files(1_000, 200);
        let map = build_map(&entries);
        let ids: Vec<_> = (0..1_000)
            .map(|i| {
                map.get_id(format!("src/module_{i:04}.rs").as_str())
                    .expect("present")
            })
            .collect();

        c.bench_function("view/1000_files_multi_line", |b| {
            b.iter(|| {
                for id in black_box(&ids) {
                    let pos = AbsolutePosition::new(*id, 10, 1, 190, 30);
                    black_box(map.view(*id, &pos).expect("valid position"));
                }
            });
        });
    }

    #[cfg(not(feature = "view"))]
    {
        let _ = c;
    }
}

#[cfg(feature = "view")]
fn bench_line_offsets(c: &mut Criterion) {
    let entries = build_files(100, 1_000);
    c.bench_function("line_offsets/100x1000_lines", |b| {
        b.iter(|| {
            for (_, content) in black_box(&entries) {
                black_box(sourcier_core::LineOffsets::compute(content).expect("fits"));
            }
        });
    });

    // LSP-style offset to (line, column) navigation.
    let (_, content) = &entries[0];
    let offsets = sourcier_core::LineOffsets::compute(content).expect("fits");
    let probes: Vec<usize> = (0..1_000).map(|i| i * (content.len() / 1_000)).collect();
    c.bench_function("line_col/1000_probes", |b| {
        b.iter(|| {
            for offset in black_box(&probes) {
                black_box(offsets.line_col_from_offset(*offset).expect("in range"));
            }
        });
    });

    let map = build_map(&entries);
    let id = map.get_id("src/module_0000.rs").expect("present");
    let pos = AbsolutePosition::new(id, 500, 1, 900, 30);
    c.bench_function("view_str/single_line_span", |b| {
        b.iter(|| {
            black_box(map.view_str(id, &pos).expect("valid position"));
        });
    });
}

#[cfg(not(feature = "view"))]
fn bench_line_offsets(c: &mut Criterion) {
    let _ = c;
}

criterion_group!(
    benches,
    bench_finalize,
    bench_get_id,
    bench_view,
    bench_line_offsets
);
criterion_main!(benches);
