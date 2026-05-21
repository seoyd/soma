# AI signal usefulness gates

Sprint 21 keeps AI-signal claims conservative.

## A signal is not useful just because net return went up

The benchmark blocks overclaiming unless the result also satisfies:

- enough official outcomes
- acceptable calibration
- no material drawdown worsening
- schema-valid predictions
- no leakage warning
- stable Risk Governor behavior
- bounded storage

## Gate set

- `SchemaValid`
- `EnoughOutcomes`
- `CalibrationAcceptable`
- `DrawdownNotWorse`
- `NetReturnNotWorse`
- `ProfitFactorAcceptable`
- `RiskGovernorStable`
- `NoLeakageWarnings`
- `DataQualityAcceptable`
- `StorageBudgetAcceptable`

## Result interpretation

- `BaselineEvaluated`: the baseline path ran, but no external model is being promoted
- `ExternalModelEvaluated`: external predictions ran, but the evidence is still not strong enough
- `UsefulCandidate`: the external model cleared the configured gates in research mode
- `PoorCalibration`, `PoorRiskBehavior`, `WorseThanBaseline`, `InsufficientOutcomes`: explicit block reasons

## Conservative defaults

- missing official data stays blocked
- mock/test-only runs stay `PipelineOnly`
- crypto-only evidence stays crypto-only
- Risk Governor veto remains final
