# Scenario Materialization V3

Sprint 45 adds a conservative scenario materialization layer for official-ready rows.

## Materialization order
1. reuse an existing row-level scenario first
2. project from official-ready candle coverage when local evidence is sufficient
3. optionally project from canonical CSV only when provenance and preflight requirements are satisfied
4. fall back to limited-feature projection when allowed
5. keep summary-derived diagnostics weaker than row-level materialization

## Safety rules
- `no_lookahead_safe = false` rejects the row
- source class is preserved; no controlled, yfinance, fixture, or crypto row is promoted into official non-crypto evidence
- limited-feature and summary-derived outputs remain explicitly weaker than row-level scenarios

## Command
```bash
cargo run --bin soma_experiment -- scenario-materialize-v3 --config examples/soma_scenario_materialize_v3_official_replication.toml
```
