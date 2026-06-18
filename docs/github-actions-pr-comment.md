# GitHub Actions PR Comment Workflow

Use the reusable workflow in `.github/workflows/audit-pr.yml` from a pull request workflow:

```yaml
name: Audit Pull Request

on:
  pull_request:

permissions:
  contents: read
  issues: write
  pull-requests: write

jobs:
  rust-technical-audit:
    uses: ./.github/workflows/audit-pr.yml
    with:
      audit-path: .
      markdown-report: rta-report.md
      scorecard-json: rta-scorecard.json
      comment: true
```

The workflow installs the `rta` binary, generates both a Markdown report and `rta.scorecard.v1` JSON, uploads both artifacts, and maintains a single sticky PR comment.

Before the first crates.io release, call the workflow with a Git install override:

```yaml
    with:
      install-command: cargo install --git https://github.com/josediegorobles/rust-technical-audit-toolkit --package rust-technical-audit-toolkit --locked
```
