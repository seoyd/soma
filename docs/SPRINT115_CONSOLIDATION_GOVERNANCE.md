# Sprint 115 Consolidation Governance

Sprint 115 is governance-only. It carries forward Sprint 114 truth, keeps the fifth patch unapplied, blocks assertion movement and target retirement, splits workspace timeout diagnostics from consolidation, and preserves full-workspace acceptance requirements.

## Key rules
- Stop or pause is a valid outcome.
- Assertion destination proof must pass before any movement.
- Evidence blur remains a central blocker.
- Focused tests, CLI smoke, cargo build, cargo progress, and timeout cleanup are supporting-only.
- Only a finished and passed `cargo test --workspace --quiet` can claim full acceptance.
- Runtime, training, live inference, live trading, broker, order, and account paths remain deferred or forbidden.
