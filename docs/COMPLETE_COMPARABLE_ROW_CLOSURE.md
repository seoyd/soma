# Complete Comparable Row Closure

Sprint 45 closes the gap between official-ready rows and complete comparable rows.

## Complete row requirements
A row is only complete when it has:
- scenario materialization
- outcome reference
- baseline reference
- NoTrade counterfactual
- RiskDenied counterfactual
- `no_lookahead_safe = true`

## Closure flow
1. inventory official-ready rows
2. materialize safe scenarios
3. build outcome backfill plan
4. build baseline backfill plan
5. build counterfactual backfill plan
6. build complete/partial/diagnostic comparable rows
7. report conservative closure status and bottleneck movement

## Interpretation
- `CompleteRowsImproved` does not prove profitability
- `OfficialCompleteRowsImproved` does not imply live readiness
- bottleneck movement only says the primary blocker changed
- controlled evidence remains diagnostic-only, crypto-only remains crypto-only, yfinance remains research-only, and fixture evidence remains architecture-test-only

## Command
```bash
cargo run --bin soma_experiment -- complete-row-close --config examples/soma_complete_row_close_official_replication.toml
```
