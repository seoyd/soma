# Contract freeze

Sprint 23 adds a lightweight `CoreContractRegistry` so major existing schemas/configs have stable version identifiers.

Tracked contracts include:

- `FeatureSchema`
- `PredictionSchema`
- `DatasetSchema`
- `ExperimentConfig`
- `OfficialCollectionPlan`
- `OfficialAiBenchmarkConfig`
- `SequenceDatasetSpec`
- `Mamba3FinCandidateSpec`
- `RiskGovernorConfig`
- `ChairConfig`
- `ReasonCodeSet`
- `AuditEventSchema`

## Why this matters

- schema drift should not be silent
- version mismatches should be reason-coded
- future Mamba/persona/evidence work should not mutate core contracts accidentally

This registry is conservative bookkeeping, not a migration engine.
