# Sample Audit Packs

These sample packs show the output shape of `rta audit-pack` on public Rust repositories.

They are **not** complete audits, endorsements, or judgments on the quality of these projects. The current scoring model is heuristic and intentionally conservative. The useful artifact is the structured due diligence workflow: executive report, scorecard, evidence, risk register, review questions, and methodology.

| Repository | Source commit | Overall | Evidence pack |
| --- | --- | ---: | --- |
| `tokio-rs/tokio` | [`7892f60`](https://github.com/tokio-rs/tokio/commit/7892f6020d9c914a41d0c350693fb71937d43c03) | 46/100 | [Open pack](tokio/README.md) |
| `tokio-rs/axum` | [`485c603`](https://github.com/tokio-rs/axum/commit/485c603dddcee45bb4bc40aab492b47576e2a2f8) | 46/100 | [Open pack](axum/README.md) |
| `ratatui/ratatui` | [`e306ce6`](https://github.com/ratatui/ratatui/commit/e306ce69df3113d41c00c483e36ba3ecc88f3c79) | 47/100 | [Open pack](ratatui/README.md) |

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
