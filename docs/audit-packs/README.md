# Sample Audit Packs

These sample packs show the output shape of `rta audit-pack` on public Rust repositories.

They are **not** complete audits, endorsements, or judgments on the quality of these projects. The current scoring model is heuristic and intentionally conservative. The useful artifact is the structured due diligence workflow: executive report, scorecard, evidence, risk register, review questions, and methodology.

| Repository | Source commit | Overall | Evidence pack |
| --- | --- | ---: | --- |
| `tokio-rs/tokio` | [`9fe3c56`](https://github.com/tokio-rs/tokio/commit/9fe3c5619dced7157fe46104641d2e1d0af44417) | 74/100 | [Open pack](tokio/README.md) |
| `tokio-rs/axum` | [`b90b8e0`](https://github.com/tokio-rs/axum/commit/b90b8e02d0f761ce36a13610acd2afa60984a5e2) | 80/100 | [Open pack](axum/README.md) |
| `ratatui/ratatui` | [`f8c7866`](https://github.com/ratatui/ratatui/commit/f8c78660742a5ce240b4a8c26d51eff42d897a4f) | 81/100 | [Open pack](ratatui/README.md) |

## Files In Each Pack

- `executive-report.md`: human-readable technical due diligence report
- `scorecard.json`: compact scorecard for dashboards and CI gates
- `evidence.json`: machine-readable evidence behind the assessment
- `risk-register.json`: structured findings with stable risk IDs
- `review-questions.md`: follow-up questions for senior review
- `methodology.md`: scope, scoring model, and interpretation notes

## Regeneration

Example:

```bash
rta audit-pack /path/to/repo --output docs/audit-packs/tokio --repo-label tokio-rs/tokio
```

Use `--repo-label` for public demos so generated reports use a clean repository name instead of a local filesystem path.
