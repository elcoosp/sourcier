#!/bin/bash

# Script de commit automatique et conventionnel - Un commit par fichier
# Compatible Bash 3.2 (macOS) - Version 2.0.0 nettoyée sans cassure Markdown

files=$(git status --porcelain | awk '{print $2}')

if [ -z "$files" ]; then
    echo "Aucun fichier modifie detecte."
    exit 0
fi

for file in $files; do
    if [ ! -f "$file" ]; then
        continue
    fi

    echo "Analyse et commit pour : $file"

    case "$file" in
        "CHANGELOG.md")
            subject="docs(changelog): draft release notes for version 2.0.0"
            body="Document the line offset lookup overhauls path storage optimizations and the wire format break."
            ;;
        "Cargo.lock")
            subject="chore: update lockfile dependencies for parallel build benchmarks"
            body="Synchronize workspace packages adding bit-set borsh rayon chacha20 and plotters crates."
            ;;
        "Cargo.toml")
            subject="chore: bump workspace version to 2.0.0"
            body="Elevate repository layout version declarations and inject the opt-in parallel feature track."
            ;;
        "PERFORMANCE.md")
            subject="docs(maintainer): add performance comparison report from 1.0 to 2.0"
            body="Record zero allocation finalization statistics and dense vector line offset optimization gains."
            ;;
        "README.md")
            subject="docs(readme): rewrite specifications targeting zero allocation finalization"
            body="Update command tables documenting verified path indexes hash safety and the parallel feature."
            ;;
        "sourcier-core/Cargo.toml")
            subject="chore(core): configure optional rayon dependency requirement"
            body="Provide optional multi-threaded line offset index construction via the parallel feature switch."
            ;;
        "sourcier-core/benches/map.rs")
            subject="perf(core): add misses testing tracks and line-col navigation benchmarks"
            body="Establish lookup probes profiles checking successful hits against empty open-addressed slots."
            ;;
        "sourcier-core/src/lib.rs")
            subject="refactor(core): enrich top level documentation mapping memory footprint metrics"
            body="Detail verified table probing rules and clarify the payload wire format transformation boundaries."
            ;;
        "sourcier-core/src/line_map.rs")
            subject="feat(core): expose line starts accessor and optimize vector initialization"
            body="Prune the redundant SIMD pre-counting scan and let callers fetch line byte offsets slices directly."
            ;;
        "sourcier-core/src/map.rs")
            subject="feat(core): build verified open-addressed index and inline path storage"
            body="Remove per-file string allocations during finalization by dropping the path map string clone step."
            ;;
        "sourcier-core/src/tests/mod.rs")
            subject="test(core): map sub-component entry configurations for parallel suites"
            body="Register the multi-threaded compilation file hooks within the test suite architecture."
            ;;
        "sourcier-core/src/tests/parallel.rs")
            subject="test(core): verify parallel line-offset index construction correctness"
            body="Assert that multi-threaded rayon builds reproduce serial byte-accurate view extraction semantics."
            ;;
        "sourcier-core/src/tests/property.rs")
            subject="test(core): property-test first-registration duplicate tiebreaks"
            body="Confirm that duplicate paths scattered at hundreds-of-files scale keep their first registration."
            ;;
        "sourcier-core/src/tests/serde.rs")
            subject="test(core): test wire payload backward compatibility and tolerant reads"
            body="Verify that v1 payloads containing path-to-id tables drop the field safely and rebuild state."
            ;;
        "sourcier-core/src/tests/snapshots/sourcier_core__tests__snapshots__build_phase_map_serializes_without_lookup_state.snap" | "sourcier-core/src/tests/snapshots/sourcier_core__tests__snapshots__finalized_map_serializes_as_reviewable_yaml.snap")
            subject="test(core): update snapshot file assertions tracking lookup tables removal"
            body="Sync checked serialization string matrices with the streamlined v2 format definitions."
            ;;
        "sourcier-core/src/tests/view.rs")
            subject="test(core): implement test cases checking raw offset ranges accessors"
            body="Confirm that empty files map to a single line and check out of bounds unknown identifier requests."
            ;;
        "src/lib.rs")
            subject="refactor: forward parallel compilation flag features to the core library"
            body="Align root crate declarations layout mapping with the version 2.0 interface parameters."
            ;;
        *)
            subject="chore: update $(basename "$file")"
            body="Refactor or clean up internal logic structures within $file."
            ;;
    esac

    # Indexation progressive, validation et enregistrement de la file d'attente
    git add "$file"
    git commit -m "$subject" -m "$body"

    echo "✅ Commite : $file"
    echo "--------------------------------------------"
done

echo "Traitement complet de la version 2.0.0 acheve avec succes !"
