# Benchmark Matrix

Sprint 11 adds a batch research layer on top of the existing single-run experiment harness.

## Core types

### `DatasetBundleConfig`

Defines the ordered set of datasets and shared defaults:

- data validation
- feature config
- walk-forward config
- barrier/cost/risk defaults
- output root

Each `DatasetEntry` includes dataset identity, symbol, local path, timeframe, venue, asset class, and enable/disable state.

### `ExperimentVariant`

Defines an ordered run variant:

- `BaselineOnly`
- `ExternalPredictionOnly`
- `TrainAndCompare`
- other existing single-run modes when useful

Variant overrides can change timeframe/resample and optionally wire prediction or training paths for local comparison experiments.

### `BatchExperimentReport`

Collects:

- `run_summaries`
- `aggregate_benchmark`
- `data_quality_summary`
- `regime_summary`
- `risk_governor_summary`
- `model_comparison_summary`
- `persona_readiness_summary`
- `expansion_readiness`

## Aggregate interpretation

The aggregate benchmark is descriptive, not promotional.

- counts include passed/failed/skipped runs
- net return is cost-aware
- drawdown matters as much as return
- model comparison only credits `external_better` conservatively
- skipped or failed runs remain visible in the matrix instead of being hidden

## Conservative defaults

- disabled datasets and variants are explicitly skipped
- remote-style paths are rejected
- `require_all_pass` surfaces matrix-level failure
- ambiguous evidence does not recommend persona expansion
