# Experiment Harness

Sprint 10 adds a deterministic **offline experiment harness** for local research runs.

## Supported modes

- `ValidateDataOnly`
- `DatasetExportOnly`
- `BaselineOnly`
- `ExternalPredictionOnly`
- `TrainAndCompare`

## Stage flow

The harness records stage status for:

- data load / validation
- optional resampling
- feature build
- dataset export
- baseline evaluation
- optional Python validation/training
- prediction import
- external evaluation
- model comparison
- report bundle writing

## Local-only policy

- local CSV only
- no live API
- no broker integration
- no runtime LLM
- Python is optional and research-only

## Output bundle

Each run writes a deterministic bundle under:

`target/soma_experiments/<experiment_id>/`

with manifest, data quality report, optional dataset, baseline/external/comparison reports, and experiment summary.

## Failure handling

Stages fail or skip explicitly. Invalid data, invalid predictions, missing Python, and report write problems are surfaced as stage statuses and reason codes.

## No live trading

This harness is for local research and backtest orchestration only. It does not enable live trading or real-money execution.
