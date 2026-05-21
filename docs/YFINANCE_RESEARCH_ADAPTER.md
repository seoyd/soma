# YFinance Research Adapter

Sprint 27 adds a bounded **research-only** yfinance path.

- Python side: `research/yfinance_fetch.py`
- Rust side: `soma-experiment yfinance-import --config <toml>`
- Aggregate report: `soma-experiment yahoo-research --config <toml>`

## Guarantees

- yfinance data is tagged `YFinanceResearch`
- it is **not** `OfficialApiCollected`
- it is **not** readiness eligible
- it stays local-file-only on the Rust side

## Fixture-first flow

1. `python research/yfinance_fetch.py --config research/configs/yfinance_research_compact.toml`
2. `soma-experiment yfinance-import --config examples/soma_yfinance_import.toml`
3. `soma-experiment yahoo-research --config examples/soma_yfinance_research_benchmark.toml`

## Comparison

Use `official-vs-yfinance` to interpret counts conservatively. yfinance-only evidence should produce `ResearchOnlyNoOfficialClaim`.
