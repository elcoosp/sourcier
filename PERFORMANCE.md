# Performance report: sourcier 1.0 → 2.0

All numbers were measured with the bundled criterion suite on the reference
machine (shared 2-core CI sandbox, x86_64). 1.0 and 2.0 were measured
back-to-back in the same machine window with identical criterion settings
(`--warm-up-time 2 --measurement-time 4`), and criterion computed the change
intervals between the paired runs (`--save-baseline` / `--baseline`).
Treat the numbers as indicative for this class of hardware.

## Results

| Scenario                                          | 1.0       | 2.0       | Change        |
| ------------------------------------------------- | --------- | --------- | ------------- |
| `finalize/100_files_50_lines` (add + finalize)    | 63.9 µs   | 41.3 µs   | **−35.3%**    |
| `finalize/500_files_100_lines` (add + finalize)   | 708.6 µs  | 513.0 µs  | **−27.5%**    |
| Finalization phase only, 2,000 × 200 lines        | 3.72 ms   | 2.77 ms   | **−25.5%**    |
| `get_id/2000_files` (2,000 hits)                  | 33.8 µs   | 41.0 µs   | +21.5% ¹      |
| `get_id/2000_misses` (2,000 misses)               | —         | 32.9 µs   | new           |
| `view/1000_files_multi_line` (1,000 spans)        | 14.4 µs   | 4.4 µs    | **−69.9%**    |
| `line_offsets/100x1000_lines` (100 scans)         | 784 µs    | 687 µs    | **−12.4%**    |
| `line_col/1000_probes` (offset → line, col)       | —         | 8.15 µs   | new           |
| `view_str/single_line_span`                       | —         | 257 ns    | new           |

¹ The verified path index trades ~20% hit latency for byte-exact
verification, HashDoS-resistant per-map seeding, zero allocations at
finalization and about half the lookup-state memory. Misses terminate on the
first empty slot and are as cheap as hits.

## Where the wins come from

1. **Single-pass line-offset scan.** 1.0 scanned every file twice (a counting
   pre-pass followed by the build pass). 2.0 scans once into a heuristically
   reserved vector. The line-offset scan dominates finalization for realistic
   files (profiling the 1.0 build showed ~78% of `finalize` time inside the
   scan), so halving its memory traffic is the single largest win.
2. **Zero-allocation finalization.** 1.0 cloned every path into a
   `HashMap<Box<str>, Id>` (one heap allocation per file). 2.0 stores paths
   inline up to 30 bytes (short-string optimization) and builds a verified
   open-addressed index of 4-byte slots — no per-path allocation at all.
3. **Dense per-id line-offset storage.** 1.0 kept `HashMap<Id, LineOffsets>`;
   every `view` call hashed the id. 2.0 uses a `Vec<Option<LineOffsets>>`
   indexed by id — `view` is a bounds check plus an offset lookup.
4. **Parallel index construction (opt-in `parallel` feature).** Spreads the
   line-offset scan across threads with rayon. On the 2-core reference VM the
   thread-pool overhead outweighed the scan savings (which is why the feature
   is opt-in); on many-core build machines with thousands of files it scales
   the dominant finalization phase.
5. **Registration-order tiebreak in the sort.** Fixes duplicate-resolution
   determinism (see the changelog) without changing sort complexity.

## Lookup index design notes

- The index is an open-addressed table sized to at least twice the entry
  count (load factor ≤ 0.5, linear probing, measured average 1.48 probes per
  successful lookup at 2,000 files).
- The hash is an Fx-style multiply-xor mix running on two independent
  accumulation chains, seeded per map from `RandomState` so adversarial
  collision patterns cannot be precomputed.
- Every probe verifies the stored hash **and** the path bytes, so hash
  collisions can never produce a wrong id — correctness does not depend on
  hash quality.

## Reproducing

```sh
cargo bench -p sourcier-core --bench map -- --save-baseline myrun
# after any change:
cargo bench -p sourcier-core --bench map -- --baseline myrun
```
