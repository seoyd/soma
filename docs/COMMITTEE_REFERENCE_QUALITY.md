# Committee Reference Quality

`ReferencePackQualityReport` summarizes whether a generated pack is usable for conservative benchmarking work.

## Counts tracked
- outcome references
- baseline references
- no-trade counterfactuals
- risk-denied counterfactuals
- official-ready references
- research-only references
- fixture references
- diagnostic-only references
- rejected references
- no-lookahead-safe references

## Conservative statuses
- `HealthyReferencePack`
- `NeedMoreOutcomeReferences`
- `NeedMoreBaselineReferences`
- `NeedMoreNoTradeCounterfactuals`
- `NeedMoreRiskDeniedCounterfactuals`
- `NeedMoreCandleData`
- `NeedBetterTimestampAlignment`
- `TooManyDiagnosticOnlyReferences`
- `ResearchOnlyReferences`
- `FixtureOnlyReferences`
- `InsufficientReferenceQuality`

## Interpretation
Healthy does not imply live readiness or profitability.
Official readiness still requires official or real-local evidence boundaries, and controlled fixture passes stay controlled-only.
