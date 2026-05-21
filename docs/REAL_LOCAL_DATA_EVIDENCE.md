# Real Local Data Evidence

Sprint 16 separates **real local evidence** from synthetic/test evidence.

## Why Sprint 15 was not enough

Sprint 15 closed the numeric evidence gap, but it used synthetic/test fixtures. That is useful for deterministic pipeline coverage, not for market-readiness interpretation.

## Evidence source taxonomy

- `RealLocal`
- `SyntheticFixture`
- `TestFixture`
- `GeneratedSynthetic`
- `ExternalPredictionOnly`
- `Unknown`

Only `RealLocal` may contribute to readiness-style evidence, and only when data-quality and provenance gates pass.

## RealLocal requirements

A dataset counts as real local evidence only when:

- it is explicitly marked `RealLocal`
- it points to a local path
- the file exists
- it is user supplied or explicitly allowed as a controlled test override
- `downloaded_by_soma == false`
- data quality is `Good` or `Warning`
- it produces enough walk-forward outcomes for the configured gate

## Readiness eligibility

Synthetic/test evidence may support smoke tests and backtest research. It does **not** count as real-market readiness evidence.
