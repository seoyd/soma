# Sprint 96 BaselineSignal Recovery

Sprint 96 adds a conservative, research-only BaselineSignal recovery bundle.

- grouped BaselineSignal suite reuse only
- preserve NoTrade default
- preserve Risk Governor hard veto
- preserve poor-data-quality denial
- preserve source-boundary and no-lookahead guarantees
- preserve research-only / paper-only interpretation
- advance CounterfactualBackfill entry and readiness precheck only

CLI entrypoint:

- `sprint96-baseline-signal-recover --config examples/soma_sprint96_baseline_signal_recover.toml`

Outputs are written under `target/soma_sprint96_baseline_signal_recovery/<reduction_id>/`.
