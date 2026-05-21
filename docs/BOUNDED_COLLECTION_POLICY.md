# Bounded Collection Policy

Sprint 19 adds explicit storage and collection budgets so equity data does not grow without bound.

## Default policy

- max symbols per run: `5`
- max rows per symbol: `500`
- max total rows per run: `2000`
- max requests per run: `20`
- max raw bytes per run: `5MB`
- compact output by default
- raw archive policy: `CompactJson`
- retention policy: `KeepLastNFiles(3)`
- full history disabled by default

## Why this exists

- prevents accidental all-market or all-history collection
- keeps tests and local runs small
- makes manifest/provenance/reporting deterministic
- forces truncation and full-history decisions to be explicit

## Recorded outputs

Every collected dataset now writes `collection_budget_report.txt` and records policy/truncation metadata in the manifest.
