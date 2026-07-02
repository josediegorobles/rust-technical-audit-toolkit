# Methodology

Rust Technical Audit Toolkit produces a heuristic technical due diligence snapshot from local repository evidence.

## Scope

- Repository structure, Cargo manifests, Rust source files, dependency declarations, test structure, and generated risk findings.
- No AI analysis, network calls, package registry lookups, or external scanners are used by the evidence pack command.

## Scoring Model

| Area | Weight |
| --- | ---: |
| Dependency Health | 20% |
| Code Quality | 25% |
| Architecture | 25% |
| Testing | 15% |
| Risk Posture | 15% |

Code Quality normalizes large modules against the repository's own p90 module size and penalizes hotspot density instead of absolute hotspot count.
Architecture uses workspace crate count, average module depth, top-level `use crate::` fan-out, module centralization, and real top-level cycle detection.
Dependency Health uses direct dependency surface, critical dependency signals, broad declarations, non-registry dependencies, and pre-1.0 API indicators.
Testing rewards visible unit and integration structures. Risk Posture discounts material findings surfaced from the other analyzers.

Bands: 90-100 excellent, 80-89 strong, 65-79 workable with focused review, 50-64 material diligence concerns, below 50 deep review or remediation required.

## Interpretation

Scores are triage indicators for senior engineering review, not absolute judgments. Evidence files preserve the signals used to produce the report so reviewers can validate, challenge, and deepen the assessment.
