# Changelog

## v0.2.1 - 2026-06-18

### Added

- Added `--repo-label` / `--repository-label` to render public demos without local filesystem paths.
- Added public sample audit packs for Tokio, Axum, and Ratatui under `docs/audit-packs/`.

## v0.2.0 - 2026-06-18

### Added

- Added `rta audit-pack [PATH] --output DIR` to generate a due diligence evidence pack.
- Added evidence pack files: `executive-report.md`, `scorecard.json`, `evidence.json`, `risk-register.json`, `review-questions.md`, and `methodology.md`.
- Added stable schema versions for evidence and risk register JSON output.
- Added score-driven and finding-driven review questions for manual diligence follow-up.
- Added regression tests for audit pack file generation, JSON validity, Markdown sections, scorecard shape, scoring weights, and sample repository signals.

### Changed

- Expanded scorecard JSON with core repository metrics.
- Improved testing detection around integration tests and async test attributes.
- Improved workspace member collection for multiline `Cargo.toml` declarations.

## v0.1.0 - 2026-06-12

Initial marketing-ready release candidate for the Rust Technical Audit Toolkit.

### Added

- `rta` binary via the `rust-technical-audit-toolkit` CLI package for Markdown, full JSON, summary, and CI scorecard output.
- `rta-core` library with repository collection, analyzers, scoring, Markdown rendering, and JSON rendering.
- Stable `rta scorecard --json` output using schema version `rta.scorecard.v1`.
- Real generated sample reports for Tokio, Axum, Ratatui, and Sled under `docs/sample-reports/`.
- Reusable GitHub Actions workflow for PR audit comments.
- Release workflow that publishes `rta-core` and `rust-technical-audit-toolkit` on `v*` tags.

### Notes

- The desired crates.io package name `rta` is already occupied by another crate, so the published package uses `rust-technical-audit-toolkit` while keeping the binary named `rta`.
