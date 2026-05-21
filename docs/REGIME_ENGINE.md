# Regime Engine

Sprint 05 adds a rule-based `RegimeClassifier` for deterministic market-state labeling.

## Classifier scope

The classifier maps current feature state into one of:

- `TrendUp`
- `TrendDown`
- `Range`
- `HighVolatility`
- `Panic`
- `RiskOn`
- `RiskOff`
- `Unknown`

It consumes only the current `FeatureVector` plus the already-seen candle window from replay.

## Precedence rules

The current precedence is:

1. `Unknown` gate when history is too short or feature data quality is too low
2. `Panic`
3. `HighVolatility`
4. `RiskOff`
5. `RiskOn`
6. `TrendDown`
7. `TrendUp`
8. `Range`
9. fallback `Unknown`

This ordering is deterministic and tested explicitly.

## Rule summary

- `Panic`: sharp negative short-horizon return plus volatility stress, with either a volume spike or a deeper 5-bar selloff
- `HighVolatility`: elevated realized volatility or ATR stress
- `RiskOff`: negative recent return with weak trend posture or stressed volatility
- `RiskOn`: positive recent return, positive volume confirmation, controlled volatility
- `TrendDown`: price and fast MA both below the slower MA with negative recent return
- `TrendUp`: price and fast MA both above the slower MA with positive recent return
- `Range`: small recent return, controlled volatility, and close positioned away from candle extremes

## Limitations

- this is not an ML classifier
- thresholds are hand-tuned conservative defaults
- the classifier is meant for deterministic backtests and safe gating, not alpha discovery
- classification quality depends on the upstream feature/data-quality path

## Deferred work

Future ML regime classification is explicitly deferred. Sprint 05 does not implement LightGBM, XGBoost, Mamba3Fin, or any online regime learner.
