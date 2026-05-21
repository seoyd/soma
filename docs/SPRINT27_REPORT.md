# Sprint 27 Report

## Delivered

- added `YFinanceResearch` source-kind classification
- extended `DataProvenance` so unofficial/official and readiness/benchmark eligibility are explicit
- added `research/yfinance_fetch.py` with fixture-first bounded canonicalization
- added `src/data/yfinance_bridge.rs` for local-only Rust preflight import
- added `src/experiment/yahoo_research.rs` aggregate reporting
- added `src/experiment/official_vs_yfinance.rs` conservative interpretation
- added CLI commands:
  - `yfinance-import`
  - `yahoo-research`
  - `official-vs-yfinance`
- added fixtures, configs, examples, docs, and tests

## Guarantees

- yfinance stays `YFinanceResearch`
- yfinance is not `OfficialApiCollected`
- yfinance does not count as readiness evidence
- yfinance can only participate as bounded research-only local data
- Rust runtime does not depend on Python or `yfinance`

## Validation

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- fixture-mode Python smoke through `research/yfinance_fetch.py`
