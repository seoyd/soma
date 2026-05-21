# Experiment Config

`ExperimentConfig` defines a deterministic local research run.

## Core fields

- experiment ID
- symbol
- local data path
- CSV format
- timeframe / optional resample target
- data validation config
- feature / regime / walk-forward config
- triple barrier / cost / risk config
- output directory
- mode

## Research bridge fields

- `run_python_training`
- `python_executable`
- `training_script_path`
- `prediction_csv_path`
- `model_card_path`
- `strict_schema_validation`

## Safety rules

- local path only
- no broker fields
- no API fields
- no credential fields
- no LLM fields

## Examples

- `examples/soma_experiment_baseline.toml`
- `examples/soma_experiment_dataset.toml`

Python is optional. Baseline-only runs must work without Python.
