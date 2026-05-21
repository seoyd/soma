# Prediction Schema

Sprint 07 adds a model-agnostic prediction row format.

## PredictionRow fields

Each row contains:

- `row_id`
- `symbol`
- `timestamp_ms`
- `timeframe`
- `fold_id`
- `split_kind`
- `model_id`
- `p_win`
- `p_stop`
- `expected_return`
- `expected_drawdown`
- `confidence`
- `no_trade_probability`
- `horizon_bars`
- `reason_codes`

## CSV header

CSV import/export uses a deterministic header order matching the field list above.

## Probability validation

The following must stay within `[0, 1]`:

- `p_win`
- `p_stop`
- `confidence`
- `no_trade_probability`

`expected_return` and `expected_drawdown` must be finite.

## Row alignment

Prediction rows align to exported dataset rows through `row_id`, plus symbol / timestamp / timeframe checks.

Sprint 07 validation reports:

- missing rows
- extra rows
- timestamp mismatches
- schema mismatches

## Reason codes

Typical prediction import reason codes include:

- `InvalidPrediction`
- `InvalidProbability`
- `PredictionSchemaMismatch`
- `PredictionAlignmentMismatch`
- `PredictionParseFailed`
- `MissingRequiredColumn`
