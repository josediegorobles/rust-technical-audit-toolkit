# Changelog

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
