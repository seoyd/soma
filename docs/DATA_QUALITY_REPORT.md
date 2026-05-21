# Data Quality Report

`DataQualityReport` is the deterministic summary of imported candle quality.

## Validation rules

- positive OHLC prices
- non-negative volume
- OHLC ordering invariants
- duplicate timestamps
- out-of-order timestamps
- gap detection by expected timeframe step
- optional bid/ask and spread sanity checks

## Score and severity

`data_quality_score` is bounded to `[0, 1]` and penalizes:

- invalid rows
- repairs
- gaps
- extreme spread
- non-positive prices
- negative volume
- OHLC invariant violations

Severity levels:

- `Good`
- `Warning`
- `Bad`
- `Unusable`

## Risk behavior

Low-quality candle data is not silently trusted. The report is explicit, and fixtures with poor spread/volume quality push the existing feature and risk path toward deny / no-trade behavior.

## Gap and duplicate handling

- duplicates can be dropped only when explicitly allowed
- out-of-order rows can be sorted only when explicitly allowed
- gaps are recorded even when the loader still returns a series
