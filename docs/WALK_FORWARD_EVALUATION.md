# Walk-forward Evaluation

Sprint 06 adds deterministic walk-forward evaluation for `soma-zero`.

## Time-ordered folds

`WalkForwardSplit` generates folds with fixed, time-ordered windows:

- train
- optional validation
- optional embargo
- test

There is no random split and no shuffling. Fold generation is deterministic for the same `CandleSeries` and `WalkForwardConfig`.

## Train / validation / test windows

`WalkForwardConfig` controls:

- `train_window_bars`
- `validation_window_bars`
- `test_window_bars`
- `step_bars`
- `embargo_bars`
- `min_train_bars`
- `max_folds`
- `allow_partial_last_fold`

Train windows exist now even though Sprint 06 does not train a model yet. They prepare future offline training and schema-locked dataset export.

## Purge / embargo

Embargo bars sit between validation/train history and the test window. This reduces fold-boundary leakage risk.

Rows whose triple-barrier label horizon would cross the end of their split are marked `Unsafe` instead of being treated as normal train/validation/test samples.

## No random split

Walk-forward evaluation is strictly chronological:

- no random train/test split
- no permutation
- no hidden look-ahead through shuffling

## No-lookahead policy

- features use only current and past candles
- labels may inspect future candles only inside explicit triple-barrier label evaluation
- evaluator fold reporting uses prefix replay plus test-window filtering so test rows retain historical context without borrowing future fold data
