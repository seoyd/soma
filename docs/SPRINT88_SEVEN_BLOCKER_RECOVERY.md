# Sprint 88 Seven Blocker Recovery

Sprint 88 follows Sprint 87 by taking the remaining seven blocker families and reporting them in an ordered recovery queue. The queue stays conservative: family-level recovery and probe output help narrow the workspace gate, but only a finished passing `cargo test --workspace --quiet` clears full acceptance.

The ordered queue is:

1. `CandleExpansionOps`
2. `ExternalPrediction`
3. `KrxEvidence`
4. `DashboardRenderer`
5. `CommitteeCliSafety`
6. `BaselineSignal`
7. `CounterfactualBackfill`

Per-family probe output is diagnostic only. No-run remains compile-only interpretation, and full workspace acceptance must remain explicit and honest.
