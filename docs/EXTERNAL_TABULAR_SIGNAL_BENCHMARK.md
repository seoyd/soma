# External tabular signal benchmark

Sprint 24 keeps external model work narrow and research-only.

## Supported paths

1. baseline-only official benchmark
2. existing prediction CSV import
3. optional Python training bridge that writes predictions outside Rust

Rust does **not** train a neural model. Python is optional, and the Rust runtime does not depend on Python being present.

## Validation rules

- prediction CSV path must be local
- schema validation can be strict
- row alignment must match exported dataset expectations
- calibration must be reported
- external improvement cannot be judged by net return alone
- Risk Governor remains an absolute veto

## Stage summary

`ExternalTabularBenchmarkStage` records:

- whether training was requested
- whether training actually ran
- backend used (`python` or existing CSV)
- prediction validation summary
- schema validity
- row alignment validity
- reason codes

If Python is unavailable, baseline-only operation still works. If an external CSV is invalid, the result stays conservative instead of silently passing.
