# Comparable Evidence Backfill

Sprint 42 backfills candle coverage into comparable evidence without changing source boundaries.

## What backfill can do
- mark `candle_coverage_available`
- attach the matched candle series id and match status
- mark official-ready candle coverage only when the comparable row and candle series are both officially eligible

## What backfill cannot do
- fabricate outcome, baseline, or counterfactual references
- promote yfinance, fixture, or controlled rows to official evidence
- silently convert summary-derived rows into row-level rows

## Command
```bash
cargo run --bin soma_experiment -- comparable-backfill --config examples/soma_comparable_backfill_official_replication.toml
```
