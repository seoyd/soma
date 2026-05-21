# Committee Materialization v2

Committee materialization v2 converts local artifacts into `CommitteeScenarioRow` values that the replay and benchmark stack can consume without adding any live trading or runtime-LLM path.

## Artifact resolver

- `CommitteeArtifactResolver` classifies local inputs as fixture, yfinance, evidence lane, core benchmark, official benchmark, source-aware benchmark, committee bundle, canonical CSV, preflight report, or unknown.
- Unknown inputs are reason-coded with `CommitteeArtifactUnknown` instead of panicking.
- Resolution stays deterministic and local-path-only.

## Row-level vs summary-derived

- Row-level materialization is preferred whenever the artifact exposes `rows`, `records`, `yfinance_symbols`, canonical CSV rows, or committee bundle scenario rows.
- Summary-derived fallback remains available only when `allow_summary_derived_rows = true`.
- Fallback rows are explicitly reason-coded with `CommitteeSummaryFallbackUsed`.

## Source boundaries

- Fixture inputs stay fixture-only and architecture-test-oriented.
- Yfinance inputs stay research-only and readiness-ineligible.
- Official-like benchmark inputs require provenance when `require_provenance = true`.
- Canonical CSV materialization remains local-file-only and does not imply broker connectivity.

## Provenance and readiness

- Imported artifacts are treated as untrusted until schema/provenance clues exist.
- Official readiness counts only row-level, readiness-eligible evidence.
- Materialization confidence is bounded and conservative by source class.

