# Core readiness

Sprint 23 adds a combined `CoreReadinessReport` that merges:

- runtime state report
- contract registry report
- determinism report
- reason-code audit
- audit summary
- risk invariant report
- live safety report
- performance budget report

## Possible statuses

- `ReadyForMoreOfficialEvidence`
- `ReadyForExternalModelPrototype`
- `ReadyForSequenceDatasetBuild`
- `NotReadyDueToContractDrift`
- `NotReadyDueToRiskInvariantFailure`
- `NotReadyDueToNondeterminism`
- `NotReadyDueToAuditGap`
- `NotReadyDueToLiveSafetyGap`
- `NeedMoreCoreHardening`

## Conservative meaning

Core readiness does **not** mean:

- live trading readiness
- real-money readiness
- Mamba3 implementation
- profitability

It only means future research layers can be added without obviously breaking the current safety contracts.
