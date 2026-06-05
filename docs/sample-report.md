# Rust Technical Audit Report

## Executive Summary

Repository `examples/sample-rust-service` received an overall technical due diligence score of **83/100**.

Cargo workspace with 2 declared members and 2 package manifest(s).

### Weighted Scoring Model

| Area | Weight | Score |
| --- | ---: | ---: |
| Dependency Health | 20% | 88 |
| Code Quality | 25% | 92 |
| Architecture | 25% | 84 |
| Testing | 15% | 82 |
| Risk Posture | 15% | 70 |

## Architecture

- Style: modular Cargo workspace
- Separation of concerns: Clear layer vocabulary detected across the repository.
- Detected layers: api, domain, repository, service
- Domain boundaries: billing, users

## Dependency Health

- Total direct dependencies: 6
- Critical dependencies: tokio, serde, axum
- Maintenance risks: none detected
- Outdated indicators: one pre-1.0 crate requires ownership review

## Code Quality

- Lines of Rust code: 1,420
- Rust modules: 18
- Function count: 76
- Average function size: 18.4 lines
- Large modules: none detected
- Potential God modules: none detected

## Testing

- Unit and integration testing structures detected.
- Unit test files: 8
- Integration test files: 3
- Test functions: 34

## Risks

### Dependency maintenance policy (medium)

Evidence: One pre-1.0 dependency is in a critical execution path.

Recommendation: Assign explicit ownership and track upgrade cadence.

## Recommendations

- Document architectural boundaries and expected module direction.
- Track dependency ownership for critical runtime crates.
- Preserve integration tests as public API behavior expands.
