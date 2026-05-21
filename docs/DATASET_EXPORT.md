# Dataset Export

Sprint 06 adds a deterministic dataset export layer for future offline model training.

## DatasetRow

Each `DatasetRow` contains:

- `symbol`
- `timestamp_ms`
- `timeframe`
- `fold_id`
- `split_kind`
- `regime`
- `data_quality_score`
- feature values in stable feature order
- optional label columns from triple-barrier evaluation
- optional reason codes

## FeatureFrame to DatasetFrame

The export path reuses the Sprint 05 `FeatureEngine` and preserves its stable feature ordering. `DatasetFrame.feature_names` is the canonical ordered schema for all rows.

## Labels

Labels are explicit and separate from features:

- `label_outcome`
- `label_net_return_pct`
- `label_gross_return_pct`
- `label_bars_held`
- `label_first_hit`

Rows near fold boundaries whose label horizon would cross the split end are marked `Unsafe`.

## Feature schema lock

Sprint 06 adds `FeatureSchema` with:

- schema version
- ordered feature names
- feature count
- stable checksum
- creator string

Validation fails when feature order or membership changes.

## CSV export

`DatasetFrame::to_csv_string()` provides deterministic CSV output:

- deterministic header order
- deterministic row order
- explicit `MISSING` markers
- fixed float formatting
- no `NaN` / `inf`

## Deferred path

JSONL export is deferred because this sprint does not add `serde_json`. Future offline LightGBM / XGBoost pipelines can build on the same `DatasetFrame` and `FeatureSchema` boundary.
