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

## Interpretation

Scores are triage indicators for senior engineering review, not absolute judgments. Evidence files preserve the signals used to produce the report so reviewers can validate, challenge, and deepen the assessment.
