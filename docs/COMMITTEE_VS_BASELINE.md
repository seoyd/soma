# Committee vs Baseline

`CommitteeVsBaselineComparison` summarizes how committee outcomes differ from reference signals when those references exist.

## Supported references

- `baseline_signal_summary`
- `external_prediction_summary`
- `no_trade_counterfactual`
- `risk_denial_counterfactual`

## Conservative behavior

- If no baseline reference exists, status is `NoBaselineReference`.
- If no outcome reference exists, status is `NoOutcomeReference`.
- The no-trade baseline is always represented so committee activity can be compared against a passive conservative baseline.
- Yfinance-only sets remain research-only even if comparison counts are available.

## Limits

The comparison is count-based and deterministic. It does **not** claim profitability without explicit outcome evidence.

