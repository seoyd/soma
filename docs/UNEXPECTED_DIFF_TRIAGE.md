# Unexpected Diff Triage

Sprint 70 adds an offline unexpected-diff triage layer on top of the Sprint 69 baseline snapshot coverage bundle.

- input stays local-only and deterministic
- output stays JSON/TXT plus optional static HTML fragments
- classifications stay conservative and never imply profitability, deployment readiness, or live trading readiness

Primary command:

```bash
cargo run --quiet --bin soma_experiment -- unexpected-diff-triage --config examples/soma_unexpected_diff_triage.toml
```

Related commands:

- `snapshot-diff-classify`
- `contract-alignment-audit-v2`
- `owner-review-close-v2`
- `trace-warning-reduce`
- `downgrade-evidence-closure-plan`
- `diff-root-cause`
- `model-version-review-disposition`
- `control-tower-diff-triage`
