# Feature Engine

Sprint 05 adds a deterministic Feature Engine for `soma-zero`.

## Feature set

`FeatureEngine` builds a `FeatureVector` or `FeatureFrame` from `CandleSeries` using stable `FeatureName` identifiers.

Current features include:

- price/return: `close`, `log_return_1/3/5/10/20`
- candle shape: `close_position_in_range`, `high_low_range_pct`, `candle_body_pct`, `upper_wick_pct`, `lower_wick_pct`
- trend: `ma_5`, `ma_20`, `ma_5_over_ma_20`, `close_over_ma_20`, `slope_ma_5`, `slope_ma_20`
- volume/liquidity: `volume`, `volume_z_20`, `trade_value`, `trade_value_z_20`, `volume_ratio_5_20`
- volatility: `atr_14`, `realized_vol_10`, `realized_vol_20`, `bollinger_width_20`, `range_volatility`
- execution-quality: `spread_bps`, `spread_bps_from_candle`, `liquidity_score_heuristic`, `data_quality_score`
- deterministic time features: `minute_of_day_sin`, `minute_of_day_cos`, `day_of_week_sin`, `day_of_week_cos`

## No-lookahead rule

Feature generation uses only the candle at the requested index and earlier candles. `BacktestSimulator::run()` calls `FeatureEngine::build_at(series, current_index)` during replay, so future candles are never visible to the feature path.

## Stable feature order

`FeatureName` is an enum with fixed ordering. Every `FeatureVector` carries the same ordered `feature_names` list, and `FeatureFrame` rows reuse that order. This keeps downstream consumers deterministic and makes tests able to assert exact feature positions.

## Missing data policy

`FeatureValue` is either `Value(f64)` or `Missing`.

Rules:

- insufficient history does not panic
- zero or non-finite inputs do not leak `NaN` or `inf`
- missing bid/ask, volume, or trade value is represented through `Missing` and lower data-quality score
- low-quality rows retain explicit reason codes so the signal/risk path can stay conservative

## Rolling window behavior

Sprint 05 adds deterministic rolling helpers:

- `rolling_mean`
- `rolling_std`
- `rolling_min`
- `rolling_max`
- `rolling_zscore`
- `rolling_sum`
- `safe_div`
- `clamp_finite`
- `pct_change`
- `log_return`
- `true_range`
- `atr`
- `realized_volatility`

These helpers return safe `Option`-based results on insufficient data and clamp away non-finite outputs instead of panicking.
