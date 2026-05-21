# Local Research Training Bridge

Sprint 08 adds a local research-only bridge around the Rust evaluation stack.

## Flow

1. Rust or a synthetic generator provides dataset CSV
2. your local Python validator checks the dataset contract
3. your local Python trainer produces predictions or a deterministic fallback
4. Python writes prediction CSV compatible with Sprint 07
5. Rust imports predictions and evaluates them through the existing external prediction path

## What Rust does

- exports dataset contract
- validates imported predictions
- applies Chair / Risk Governor / walk-forward evaluation
- compares baseline vs external results

## What Python does

- validates dataset files
- trains research models locally
- writes prediction CSV
- writes model card

## Runtime boundary

The Rust live/runtime path does **not** depend on Python. Python is research-only in this sprint.

## Repository note

The repository no longer bundles the old `research/` helper scripts. If you use this bridge now, point `training_script_path` at your own local script and optionally place a sibling `validate_dataset.py` next to it.
