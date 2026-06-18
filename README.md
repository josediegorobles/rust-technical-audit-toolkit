# Rust Technical Audit Toolkit

**72h Technical Due Diligence Flash for Rust codebases.**

![Sample audit packs](https://img.shields.io/badge/sample-audit%20packs-2ea44f)
![CLI scorecard](https://img.shields.io/badge/CI-scorecard%20JSON-blue)
![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)
![crates.io](https://img.shields.io/crates/v/rust-technical-audit-toolkit)

In 30 seconds, a founder, VC, or CTO can open a complete sample evidence pack and see the kind of diligence output the toolkit generates:

| Public repository | Overall | Evidence pack | Executive report |
| --- | ---: | --- | --- |
| Tokio | 46/100 | [docs/audit-packs/tokio](docs/audit-packs/tokio/README.md) | [executive-report.md](docs/audit-packs/tokio/executive-report.md) |
| Axum | 46/100 | [docs/audit-packs/axum](docs/audit-packs/axum/README.md) | [executive-report.md](docs/audit-packs/axum/executive-report.md) |
| Ratatui | 47/100 | [docs/audit-packs/ratatui](docs/audit-packs/ratatui/README.md) | [executive-report.md](docs/audit-packs/ratatui/executive-report.md) |

Sample packs are illustrative outputs from public repositories, not complete audits or judgments on those projects. The value is the repeatable evidence workflow: report, scorecard, evidence, risk register, review questions, and methodology.

```text
Rust Technical Audit Toolkit
Repository: ./service
Overall score: 93/100
Crates: 1
Dependencies: 4 direct
Maintainability: 100/100
Architecture: 90/100
Testing: 73/100
Risks: 0 finding(s)
```

`rta` is a CLI-first technical due diligence tool for Rust codebases. It helps consulting engineers form a fast, structured view of architecture, maintainability, dependency posture, testing maturity, and delivery risk before deeper manual review.

This is not a security scanner. It is not a linter. It is an engineering assessment platform for commercial technical due diligence.

## What It Analyzes

| Area | Signals |
| --- | --- |
| Repository overview | Crates, packages, workspace members, project size, language mix, Cargo manifests |
| Dependency analysis | Direct dependencies, critical dependencies, broad or non-registry declarations, maintenance indicators |
| Code quality | Lines of Rust code, module count, function count, average function size, large modules, God module candidates |
| Architecture review | Layer vocabulary, domain boundaries, modularity, circular dependency risk indicators |
| Engineering risk | Bus factor concerns, single points of failure, complex modules, lack of tests, dependency concentration |
| Testing maturity | Unit test presence, integration tests, test function count, testing structure |

## Install

Install the published CLI:

```bash
cargo install rust-technical-audit-toolkit
```

This installs the `rta` executable:

```bash
rta . --summary
```

The crates.io package is named `rust-technical-audit-toolkit` because `rta` is already published by another project. The binary name remains `rta`.

For local development:

```bash
cargo run -p rust-technical-audit-toolkit -- examples/sample-rust-service --summary
```

## CLI Usage

```bash
rta [PATH] [--markdown|--json|--summary] [--output FILE] [--repo-label LABEL]
rta scorecard [PATH] --json [--output FILE] [--repo-label LABEL]
rta audit-pack [PATH] --output DIR [--repo-label LABEL]
```

Examples:

```bash
rta . --summary
rta ./service --json
rta ./service --markdown --output audit-report.md
rta scorecard ./service --json --output scorecard.json
rta audit-pack ./service --output audit-pack --repo-label owner/repo
```

## CI Scorecard

`rta scorecard --json` emits a small, stable JSON contract suitable for CI gates, dashboards, and PR comments:

```json
{
  "schema_version": "rta.scorecard.v1",
  "repository_path": "examples/sample-rust-service",
  "overall_score": 93,
  "scores": {
    "dependency_health": 96,
    "code_quality": 100,
    "architecture": 90,
    "testing": 73,
    "risk_posture": 100
  },
  "metrics": {
    "crate_count": 1,
    "package_count": 1,
    "direct_dependencies": 4,
    "lines_of_rust_code": 71,
    "rust_modules": 6,
    "function_count": 9,
    "average_function_size": 4.1,
    "unit_test_files": 1,
    "integration_test_files": 1,
    "test_function_count": 2
  },
  "risk_findings": {
    "total": 0,
    "high": 0,
    "medium": 0,
    "low": 0
  }
}
```

See [.github/workflows/audit-pr.yml](.github/workflows/audit-pr.yml) and [docs/github-actions-pr-comment.md](docs/github-actions-pr-comment.md) for a reusable GitHub Actions PR comment workflow.

## Output Formats

`--summary` prints a compact executive snapshot for triage.

`--json` emits the full machine-readable audit report for dashboards, pipelines, or consulting portals.

`--markdown` generates a professional due diligence report with:

- Executive Summary
- Architecture
- Dependency Health
- Code Quality
- Testing
- Risks
- Recommendations
- Overall Score

`audit-pack` generates a complete diligence folder with:

- `executive-report.md`: human-readable technical due diligence report
- `scorecard.json`: compact CI/dashboard scorecard
- `evidence.json`: machine-readable signals behind the assessment
- `risk-register.json`: structured findings with stable risk IDs
- `review-questions.md`: score-driven and finding-driven follow-up questions
- `methodology.md`: scope, scoring model, and interpretation notes

Sample reports:

- [Sample audit packs](docs/audit-packs/README.md)
- [Tokio audit sample](docs/sample-reports/tokio.md)
- [Axum audit sample](docs/sample-reports/axum.md)
- [Sled audit sample](docs/sample-reports/sled.md)
- [Ratatui audit sample](docs/sample-reports/ratatui.md)

## Scoring Model

The first scoring model is intentionally transparent:

| Area | Weight |
| --- | ---: |
| Dependency Health | 20% |
| Code Quality | 25% |
| Architecture | 25% |
| Testing | 15% |
| Risk Posture | 15% |

Scores are heuristic indicators, not absolute judgments. The tool is designed to make senior review faster by surfacing where manual diligence should focus. Unsupported metrics are omitted rather than fabricated.

## Architecture

The workspace is split into:

- `crates/audit-core`: collection, analyzers, scoring, JSON rendering, Markdown rendering
- `crates/audit-cli`: CLI argument handling and command execution
- `examples/sample-rust-service`: small fixture repository for demos and regression checks

Analyzer modules implement a shared trait and consume a `RepositorySnapshot`. Report and evidence-pack renderers stay in the core crate so the CLI can remain a thin delivery layer.

## Roadmap

- Parser-backed Rust syntax analysis
- Cargo metadata integration
- Optional `cargo outdated` integration
- Trend comparison between audit runs
- HTML report output
- Rule severity configuration
- Repository ownership and contributor analysis

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT license
