# Direct Watch Readiness

Sprint 72 hardens the direct-watch estimate for local paper/research monitoring after extra evidence is attached.

## Score meaning

- the score is a local monitoring readiness range
- it is not a live-trading permission
- it improves only when evidence, owner checklist state, and next actions become clearer

## Current example result

- current range: `82~86`
- target range: `86~90`
- status: `NeedsEvidence`

## Main commands

```bash
cargo run --quiet --bin soma_experiment -- direct-watch-score --config examples/soma_direct_watch_score.toml
cargo run --quiet --bin soma_experiment -- briefing-readiness-gate --config examples/soma_briefing_readiness_gate.toml
```
