# Official AI benchmark

Sprint 21 adds a bounded benchmark layer that starts from official collection coverage and asks a conservative question:

> On bounded official-market data, does the current signal pipeline produce research-only evidence that is better than doing nothing and not worse than the baseline once costs, calibration, and risk behavior are included?

## What this benchmark does

1. load or run an `OfficialCollectionPlan`
2. select ready official datasets
3. export datasets for research
4. run baseline walk-forward evaluation
5. optionally run external prediction evaluation
6. compute calibration and model comparison summaries
7. summarize Risk Governor interaction
8. apply usefulness gates
9. write a deterministic benchmark report

## What it does not mean

- not live trading readiness
- not broker readiness
- not real-money readiness
- not proof that one symbol or one venue generalizes to all markets

## Key outputs

- `official_ai_benchmark_report.json`
- `official_ai_benchmark_report.txt`
- `ai_signal_usefulness_report.md`

## CLI

```bash
cargo run --bin soma_experiment -- ai-benchmark --config examples/soma_ai_benchmark_upbit_only.toml
```

Shortcut alias:

```bash
cargo run --bin soma_experiment -- collect-train-evaluate --config examples/soma_ai_benchmark_official_compact.toml
```
