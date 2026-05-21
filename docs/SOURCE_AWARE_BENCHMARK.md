# Source-Aware Benchmark

Sprint 28 adds a research-only layer that compares **official** and **yfinance** datasets without merging their meanings.

## Inputs

- official benchmark reports
- official collection reports
- yahoo research reports
- optional yfinance import configs

## Guarantees

- `OfficialApiCollected` and `YFinanceResearch` are inventoried separately
- yfinance readiness count stays `0`
- yfinance-only runs stay research-only
- high source mismatch blocks stability claims

## CLI

```bash
cargo run --bin soma_experiment -- source-benchmark --config examples/soma_source_benchmark_yfinance_only.toml
```
