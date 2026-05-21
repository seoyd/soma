# Candle Coverage Closure

Sprint 42 adds a conservative closure runner for candle coverage.

## Flow
1. validate local-only config
2. build or load the candle pack
3. run timeframe and timestamp alignment
4. compute candle coverage matches
5. optionally backfill comparable evidence
6. optionally record reference-generation, counterfactual, and scorecard-rerun summaries
7. compare the bottleneck before and after
8. emit a conservative recommendation

## Interpretation
- official candle coverage improvement does not prove profitability
- diagnostic-only coverage remains diagnostic-only
- missing timeframe or timestamp alignment keeps the bottleneck open
- Risk Governor remains the final veto and is unchanged

## Command
```bash
cargo run --bin soma_experiment -- candle-coverage-close --config examples/soma_candle_coverage_close_official_replication.toml
```
