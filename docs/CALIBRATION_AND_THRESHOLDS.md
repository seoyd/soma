# Calibration and Thresholds

Sprint 07 adds calibration reporting and research-only threshold search.

## Brier score

Calibration uses `p_win` against actual triple-barrier win outcomes and reports a Brier score.

## Calibration bins

Reports use deterministic probability bins and include:

- bin range
- sample count
- average predicted probability
- actual win rate
- per-bin Brier score

## ECE

Expected calibration error is computed deterministically from the same fixed bins.

## Threshold search

Threshold search is a **research artifact** only.

It searches deterministic candidates over:

- `p_win`
- `p_stop`
- `confidence`
- `no_trade_probability`
- `min_expected_return`

## Validation-only selection

If validation rows exist, threshold search uses validation rows only.

If no validation rows exist, the search is marked research-only / not-for-live.

## No live mutation

Threshold results do **not** mutate Risk Governor rules, Chair policy, or any live trading path.
