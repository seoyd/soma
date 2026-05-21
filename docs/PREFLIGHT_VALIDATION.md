# Preflight validation

Sprint 17 adds a deterministic **preflight validator** before real-evidence reruns.

## Final statuses

- `ReadyForRealEvidence`
- `NeedsColumnMapping`
- `NeedsMoreRows`
- `DataQualityTooLow`
- `MissingFile`
- `UnsupportedFormat`
- `AmbiguousFormat`
- `NotRealLocalEligible`

## What the report includes

- detected CSV format
- provenance
- data-quality summary
- data-manifest preview
- row counts
- estimated walk-forward folds
- estimated outcome records
- estimated comparable variants
- blockers and warnings

## Conservative rules

- if the file is missing, stop with `MissingFile`
- if the format is unsupported or ambiguous, do not guess silently
- if quality is bad/unusable, readiness estimate becomes zero
- if walk-forward cannot produce folds, outcome estimate becomes zero
- if `user_supplied=false`, the dataset is not real-local readiness eligible
- synthetic/test evidence still does not count toward real-market readiness
