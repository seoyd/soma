# Official Candle Expansion Plan

Sprint 43 adds a bounded expansion planner for explicit candle gaps.

## Job types
- local import jobs prefer existing canonical CSV, provenance, preflight, and manifest sidecars.
- provider collection jobs stay bounded by job count, symbol count, rows, requests, and total bytes.
- skipped jobs remain explicit when auth, approval, endpoint templates, or source eligibility are missing.

## Policy
- local-only config paths are required.
- `run_collection_jobs` stays off by default.
- `run_import_jobs` stays on by default.
- provider collection is bounded and research-only.
- yfinance, fixture, and synthetic sources are never promoted into official expansion.

## Operator actions
Missing prerequisites produce deterministic actions instead of silent success:
- set `ALPHAVANTAGE_API_KEY`
- set `KRX_API_KEY`
- set `KRX_ENDPOINT_TEMPLATE`
- wait for KRX approval
- provide canonical CSV, provenance, or preflight files

## Commands
```bash
cargo run --bin soma_experiment -- candle-expansion-plan --config examples/soma_candle_expansion_plan_missing_auth.toml
cargo run --bin soma_experiment -- candle-expansion-plan --config examples/soma_candle_expansion_plan_local_import.toml
cargo run --bin soma_experiment -- candle-expansion-actions --config examples/soma_candle_expansion_plan_missing_auth.toml
```
