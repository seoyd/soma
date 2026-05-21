# Core-checked benchmark

Sprint 24 adds a research-only wrapper around the existing official AI benchmark:

1. run `core-check`
2. require an allowed `CoreReadinessStatus`
3. load bounded official collection evidence
4. export dataset / run baseline
5. optionally evaluate external tabular predictions
6. write a deterministic benchmark report

## Why the core gate exists

The benchmark must not treat experiment plumbing as safe by default. `core-check` proves:

- runtime stays research-only
- contract drift is not present
- determinism guard still passes
- reason-code audit is complete enough
- live safety still blocks broker/account/live surfaces
- Risk Governor invariants still hold

If `core-check` fails, `core-benchmark` stops unless the core config is explicitly in `DiagnosticsOnly`.

## What this benchmark allows

- local file inputs only
- bounded official collection reports
- dataset export for research
- baseline evaluation
- optional external prediction evaluation
- optional Python training outside Rust

## What it blocks

- live trading
- broker/order/account flows
- runtime LLM paths
- Rust-native neural training
- Mamba3 / Mamba3Fin runtime work
- all-symbol and full-history expansion by default

## Outputs

- `core_checked_benchmark_report.json`
- `core_checked_benchmark_report.txt`
- `core_checked_benchmark_report.md`

`CoreCheckedBenchmarkStatus` remains conservative. Even a passing report is not live readiness and not a real-money recommendation.
