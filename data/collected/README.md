# Collected market data

This directory is reserved for **collector-generated** local market data.

## Rules

- research only
- canonical CSV + provenance + manifest + budget report stay together
- verify provider licensing before redistribution
- do not commit large collected datasets by default
- fixture/test outputs should stay under `target/`
- venue is part of the path
- bounded collection policy is the default

## Typical layout

```text
data/collected/alphavantage/nasdaq/AAPL/1d/
  raw/
  canonical/
  data_manifest.txt
  data_provenance.txt
  collection_budget_report.txt
  preflight_report.txt
```

Sprint 20 official plan runs also write a plan-level report:

```text
target/soma_official_collection/<plan_id>/
  official_collection_report.json
  official_collection_report.txt
```
