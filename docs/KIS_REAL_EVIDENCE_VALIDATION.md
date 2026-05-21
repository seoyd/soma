# KIS Real Evidence Validation

Sprint 74 validates local KIS canonical CSV evidence conservatively before any evaluation-style follow-up.

## Validation requirements

- canonical CSV columns must be present
- provenance must be attached
- preflight must be attached
- manifest/source class must be attached
- OHLC invariants must hold
- remote URL-like evidence is unsafe

## Interpretation

Passing validation means the local evidence is structurally usable for research follow-up only. It does not imply profitability, deployability, or live readiness.
