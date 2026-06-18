# Rust Technical Audit Report

## Executive Summary

Repository `/Users/jdrobpar/Documents/Codex/2026-06-12/eres-un-ingeniero-rust-senior-el-2/work/audit-targets/sled` received an overall technical due diligence score of **64/100**.

Cargo workspace with 1 declared member(s) and 2 package manifest(s).

### Weighted Scoring Model

| Area | Weight | Score |
| --- | ---: | ---: |
| Dependency Health | 20% | 76 |
| Code Quality | 25% | 16 |
| Architecture | 25% | 70 |
| Testing | 15% | 100 |
| Risk Posture | 15% | 82 |

## Architecture

- Style: layered single service or compact workspace
- Separation of concerns: Limited explicit layer separation detected.
- Detected layers: none detected
- Domain boundaries: sync

## Dependency Health

- Total direct dependencies: 27
- Critical dependencies: serde in Cargo.toml
- Maintenance risks: none detected
- Outdated indicators: crossbeam-queue is pinned to pre-1.0 API surface in Cargo.toml, ebr is pinned to pre-1.0 API surface in Cargo.toml, fs2 is pinned to pre-1.0 API surface in Cargo.toml, inline-array is pinned to pre-1.0 API surface in Cargo.toml, log is pinned to pre-1.0 API surface in Cargo.toml, pagetable is pinned to pre-1.0 API surface in Cargo.toml, parking_lot is pinned to pre-1.0 API surface in Cargo.toml, zstd is pinned to pre-1.0 API surface in Cargo.toml, env_logger is pinned to pre-1.0 API surface in Cargo.toml, futures is pinned to pre-1.0 API surface in Cargo.toml, libc is pinned to pre-1.0 API surface in Cargo.toml, num-format is pinned to pre-1.0 API surface in Cargo.toml, rand is pinned to pre-1.0 API surface in Cargo.toml, rand_distr is pinned to pre-1.0 API surface in Cargo.toml

## Code Quality

- Lines of Rust code: 12191
- Rust modules: 37
- Function count: 497
- Average function size: 27.5 lines
- Large modules: tests/tree/mod.rs (341 lines), tests/test_tree_failpoints.rs (1848 lines), tests/test_tree.rs (1459 lines), tests/00_regression.rs (1666 lines), examples/bench.rs (610 lines), src/flush_epoch.rs (384 lines), src/object_location_mapper.rs (308 lines), src/leaf.rs (310 lines), src/heap.rs (1119 lines), src/lib.rs (414 lines), src/metadata_store.rs (852 lines), src/db.rs (596 lines), src/tree.rs (2249 lines), src/object_cache.rs (998 lines)
- Potential God modules: tests/test_tree_failpoints.rs, tests/test_tree.rs, tests/00_regression.rs, examples/bench.rs, src/heap.rs, src/metadata_store.rs, src/db.rs, src/tree.rs, src/object_cache.rs

## Testing

- Unit and integration testing structures detected.
- Unit test files: 9
- Integration test files: 17
- Test functions: 130

## Risks

### Potential God modules (high)

Evidence: tests/test_tree_failpoints.rs, tests/test_tree.rs, tests/00_regression.rs, examples/bench.rs, src/heap.rs, src/metadata_store.rs, src/db.rs, src/tree.rs, src/object_cache.rs

Recommendation: Extract cohesive submodules and isolate orchestration from domain behavior.

## Recommendations

- Document intended architectural boundaries and enforce them through crate or module ownership.
- Create a dependency ownership and upgrade policy.
- Prioritize decomposition of large modules and high-branching functions.
