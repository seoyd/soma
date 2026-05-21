# Research Runbook

`soma-experiment batch --config <matrix-config>` runs a deterministic local-only research matrix over multiple datasets.

The runbook is intentionally narrow:

- local CSV files only
- paper/backtest/research only
- no broker, live API, WebSocket, or runtime LLM path
- Python remains optional; `BaselineOnly` batch runs work without it

## Flow

1. Define a `DatasetBundleConfig` with local datasets.
2. Define one or more `ExperimentVariant` entries.
3. Run the batch CLI.
4. Inspect:
   - per-run experiment bundles
   - `batch_summary.txt`
   - aggregate benchmark table
   - data quality summary
   - model comparison summary
   - persona / expansion readiness output

## Baseline-only batch

Use `examples/soma_batch_baseline_only.toml` when you want a Rust-only stability sweep with no Python dependency.

## Optional external comparison

Batch matrix variants can override:

- `prediction_csv_path`
- `training_script_path`
- `python_executable`
- `run_python_training`

That keeps comparison research local and explicit. If predictions are missing or invalid, the run fails conservatively instead of silently passing.

## Report interpretation

- A single good dataset is not enough to recommend expansion.
- Bad or unusable data pushes the readiness decision toward `ImproveDataFirst`.
- Very high denial rates push the readiness decision toward `ImproveRiskGovernorFirst`.
- Mixed evidence stays conservative with `NeedMoreExperiments` or `HoldCurrentScope`.
