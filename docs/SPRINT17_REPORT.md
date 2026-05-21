# Sprint 17 report

## Implemented items

- `LocalDataOnboardingConfig`
- `CsvFormatDetector`
- `PreflightValidator`
- `EvidenceTargetEstimate`
- `GeneratedConfigBundle`
- `RealEvidenceRerunPlan`
- local-only CLI commands:
  - `data-preflight`
  - `onboard-data`

## Tests

- onboarding config validation
- CSV format detection
- preflight final statuses
- conservative evidence estimates
- config generation gating
- rerun plan generation
- CLI safety

## Whether real local data is provided

No committed real market CSV is added in-repo. Sprint 17 builds the onboarding and rerun layer so users can safely add their own local CSV outside the repository defaults.

## Current readiness

The repository remains conservative:

- Sprint 16 current real-only result stays `MissingRealLocalData`
- Sprint 17 only improves onboarding, diagnostics, and rerun ergonomics
- no live-trading or real-money readiness is claimed

## Deferred items

- actual user-provided real market CSV
- multi-dataset real-local evidence accumulation
- later design review only after real-only gates are truly met

## Next sprint recommendation

Use Sprint 18 for **real local CSV ingestion and rerunning real-evidence with actual user data**, not for persona expansion or live trading work.
