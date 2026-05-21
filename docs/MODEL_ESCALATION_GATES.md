# Model escalation gates

Sprint 22 does not jump straight from a promising benchmark to a Mamba implementation.

## Why the gate exists

One improved net-return number is not enough. A later prototype must still clear:

- official cross-dataset consistency
- enough outcome count
- calibration quality
- Risk Governor stability
- sequence-spec readiness
- storage budget

## Possible outcomes

- `ImproveOfficialDataFirst`
- `ImproveFeatureSetFirst`
- `ImproveSignalModelFirst`
- `ImproveRiskGovernorFirst`
- `BuildSequenceDatasetFirst`
- `BuildMamba3FinExternalPrototype`
- `KeepBaselineAndExternalBridge`
- `Blocked`

## Important constraints

- Rust-native Mamba inference is deferred in this sprint
- the only allowed prototype path is `ExternalPredictionFile`
- crypto-only prototype approval needs an explicit config flag
- missing auth or missing equity evidence blocks broader readiness claims

## Recommended interpretation

- improve data first when coverage/auth/equity evidence is weak
- improve signal model first when calibration or benchmark quality is weak
- improve risk first when denial-rate / emergency-stop behavior is unstable
- build a prototype only after consistency, sequence, and storage checks all stay bounded
