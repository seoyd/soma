# Multi-Row Official Evidence Set

Sprint 47 adds a bounded, deterministic multi-row official evidence set builder.

## Inputs
- official-ready inventory JSON or config
- comparable evidence bundle JSON (optional but useful for baseline refs)
- local official candle CSV/pack paths
- local provenance and preflight sidecars
- outcome linkage v3 report/config
- counterfactual completion v2 report/config

## Guarantees
- local-only path validation
- no hidden lookahead
- deterministic ordering
- item status is emitted per row (`OfficialComplete`, partial missing-reference states, `DiagnosticControlled`, `CryptoOnly`, `ResearchOnly`, `FixtureOnly`, `SourceIneligible`, `NoLookaheadRejected`)
- official/non-crypto rows counted separately from crypto, controlled, yfinance, and fixture rows
- set summaries include official partial rows, outcome/baseline/counterfactual coverage, no-lookahead-safe count, and bounded storage bytes
- one complete row remains insufficient for usefulness claims

## CLI
`cargo run --bin soma_experiment -- multi-row-official-set --config examples/soma_multi_row_official_set_multi_row.toml`
