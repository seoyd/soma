# Sequence dataset spec

Sprint 22 adds a bounded sequence-spec layer for future model work. It does **not** export tensors or implement a neural runtime.

## What it does

- defines `SequenceDatasetConfig`
- estimates fixed-length window counts
- estimates storage bytes before heavy export work
- records a feature-schema hash
- keeps label alignment explicit
- checks whether the sequence plan remains no-lookahead-safe

## Windowing model

Each `SequenceRowRef` describes:

- the start and end row used for the input window
- the row that owns the label
- the symbol and timestamp range
- the feature schema hash
- the split/fold metadata

The label index is always aligned to the window end. The helper only uses current or past rows for the window.

## No-lookahead rule

`prior_window_features_unchanged(...)` reuses the leakage guard and verifies that mutating a future candle does not change previously computed features inside the chosen window.

## Storage budget rule

The spec is rejected conservatively when:

- `estimated_bytes > max_bytes`, or
- `estimated_bytes > storage_budget.max_total_bytes`

That keeps future sequence export bounded before any prototype work.

## CSV compatibility

`SequenceDatasetSpec::from_dataset_csv_path(...)` reads the existing exported `dataset.csv` format and excludes metadata/label columns when estimating feature bytes.
