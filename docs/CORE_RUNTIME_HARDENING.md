# Core runtime hardening

Sprint 23 adds an explicit runtime state layer so future research features cannot quietly bypass safety boundaries.

## Runtime modes

- `Research`
- `Backtest`
- `Paper`
- `CollectOnly`
- `EvaluateOnly`
- `DiagnosticsOnly`
- `LiveDisabled`

There is **no active live-trading mode**.

## Runtime stages

- `Init`
- `LoadConfig`
- `ValidateConfig`
- `LoadData`
- `ValidateData`
- `BuildFeatures`
- `GenerateSignals`
- `ChairDecision`
- `RiskEvaluation`
- `PaperExecution`
- `OutcomeEvaluation`
- `ReportGeneration`
- `Completed`
- `Failed`

## Legal transition rules

- `PaperExecution` is allowed only for `Research`, `Backtest`, and `Paper`
- `RiskEvaluation` must happen before `PaperExecution`
- `OutcomeEvaluation` requires an explicit decision-record path
- `Failed` blocks later execution unless the mode is `DiagnosticsOnly`

This is a hardening layer, not a live-execution layer.
