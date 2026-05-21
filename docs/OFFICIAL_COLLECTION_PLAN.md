# Official collection plan

Sprint 20 adds a bounded orchestration layer above the Sprint 18/19 collector.

## Goal

- collect a **small whitelist** of official-market datasets
- keep rows, requests, and bytes bounded at the plan level
- skip missing-auth entries conservatively instead of failing the whole run
- write a single collection report for downstream evidence execution

## Main types

- `OfficialCollectionPlan`
- `OfficialCollectionEntry`
- `OfficialCollectionRunner`
- `OfficialCollectionReport`

## Entry behavior

Each entry can override symbol-specific limits while inheriting plan defaults.

- `provider_kind`, `symbol`, `venue`, `asset_class`, `timeframe`
- optional `start` / `end`
- optional `max_rows` / `max_requests`
- optional `auth_config_ref`
- optional `endpoint_template`
- optional `fixture_path` for deterministic tests only

## Status model

- `Collected`
- `SkippedMissingAuth`
- `SkippedBudgetExceeded`
- `FailedProvider`
- `FailedPreflight`
- `DiagnosticOnly`

`SkippedMissingAuth` is the default conservative path for KRX and AlphaVantage when auth metadata is configured but the required env var is absent and `continue_on_missing_auth = true`.

## CLI

```bash
cargo run --bin soma_experiment -- collect-plan --config examples/soma_official_collection_compact.toml
```

The runner writes:

- `official_collection_report.json`
- `official_collection_report.txt`
- per-entry collector outputs under `<output_root>/<plan_id>/...`
