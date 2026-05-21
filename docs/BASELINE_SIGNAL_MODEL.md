# Baseline Signal Model

Sprint 05 adds `BaselineSignalModel`, a conservative rule-based signal layer.

## Purpose

The baseline model is not intended to be profitable by itself. Its job is to provide a deterministic, auditable bridge from:

- `FeatureVector`
- `RegimeDecision`
- `CostModel`

to standard `SignalOutput`.

## Conservative behavior

The model starts from a strong `NoTrade` bias and adjusts only when conditions are supportive.

It penalizes:

- low `data_quality_score`
- wide spread
- weak liquidity
- high realized volatility
- non-positive edge after cost
- `Unknown` and `Panic` regimes

It only improves the trade case when trend/risk-on conditions align with positive momentum, volume confirmation, and acceptable spread.

## Output shape

The model fills:

- `p_win`
- `p_stop`
- `expected_return`
- `expected_drawdown`
- `confidence`
- `no_trade_probability`
- `horizon_bars`

and marks the source as `baseline_rule_v0`.

## Integration path

`BacktestSimulator::run()` now follows:

1. build features with `FeatureEngine`
2. classify regime with `RegimeClassifier`
3. evaluate `BaselineSignalModel`
4. pass the signal into persona voting, Chair, Risk Governor, and outcome evaluation

This keeps the simulator feature-driven while preserving the existing paper-only execution path and Risk Governor veto.

## Deferred model path

Sprint 05 does not train LightGBM, XGBoost, or Mamba3Fin models. A future sprint can add an external model inference interface on top of the same `FeatureVector` contract without changing the deterministic replay foundations added here.
