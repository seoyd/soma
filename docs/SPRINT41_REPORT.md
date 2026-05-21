# Sprint 41 Report

Implemented:
- comparable committee evidence config, rows, builder, and quality report
- counterfactual depth plan and closure bundle
- scenario materialization weak closure report
- core scorecard rerun summary
- CLI commands, examples, docs, and tests

Validation targets:
- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- Sprint 41 CLI smoke runs

Primary interpretation:
- comparable evidence is normalized but conservative
- counterfactual depth improves only when references and official-complete rows actually improve
- scorecard reruns stay research-only
- risk governor remains an absolute veto
