# Manual Historical Daily CSV Import

## Purpose

Manual historical daily import lets the owner place sanitized local daily CSV
text into the paper evaluation path without adding a downloader, network
provider, broker, account, order, or secret boundary. The parser is pure: it
accepts CSV text and config, then returns a validated dataset or reason-coded
rejection.

This contract is for local historical daily bars only. It does not claim
profitability or live trading readiness.

## Allowed Location

Owner-provided larger datasets should remain local unless a separate sanitized
sample policy allows committing them. The preferred local directory is:

- `data/historical/sanitized/`
- `data/historical/evidence_packs/` for local manifest files

Small deterministic unit-test CSV strings may remain inline in tests. The
production parser does not require a path and performs no filesystem access.

## Required Columns

- `symbol`
- `date`
- `open`
- `high`
- `low`
- `close`
- `volume`

`date` must be `YYYY-MM-DD`. Other formats are rejected.

## Optional Columns

- `adjusted_close`
- `trade_value`
- `currency`
- `market`
- `source`
- `split_factor`
- `dividend`

`adjusted_close` is controlled by `ManualAdjustedClosePolicy`: ignore it, use
it for return-only candle conversion, or reject it if present.

## Rules

- daily data only,
- sanitized local data only,
- read-only parser,
- no intraday calendar validation in this sprint,
- exchange calendar and session validation deferred,
- strict single-symbol mode by default,
- monotonic dates required by default,
- duplicate dates rejected by default,
- invalid OHLC rejected,
- non-finite values rejected,
- non-positive required prices rejected,
- negative volume or optional numeric values rejected.

## Forbidden Material

The import rejects secret-like, private, account, order, raw-provider,
endpoint, live-provider, exchange-secret, environment-file, private-mapping,
and temporary instruction-file markers. It also rejects endpoint-like columns
and URLs.

No API keys, account IDs, order IDs, raw provider responses, live endpoint
fields, broker endpoint fields, or private mappings belong in this CSV.

## Output

`ManualHistoricalDailyDataset` records:

- dataset ID,
- source kind,
- single symbol,
- validated daily rows,
- date range,
- `LocalDataQualitySummary`,
- `sanitized=true`,
- `local_only=true`,
- reason codes.

`to_daily_candle_series` converts the dataset into `CandleSeries` with
`Timeframe::OneDay`. It does not call Chair, Risk Governor, broker, network,
or any runtime model.

## Evidence Pack Wrapper

`HistoricalEvidencePackManifest` can list multiple owner-provided sanitized
local daily CSV sources. The evidence pack loader wraps this importer rather
than replacing it:

- each enabled source is parsed as a single-symbol daily dataset,
- disabled and rejected sources remain visible,
- US, Korean, and BTC daily source kinds map to the existing local daily CSV
  kinds,
- test-only CSV text can be used for deterministic unit tests,
- production use supports local CSV paths without adding a downloader.

The loader rejects unsafe paths before reading and rejects unsafe CSV text
before parsing. It adds no live provider, no broker, no order path, no network
client, and no credential requirement.

## Owner Trial Wrapper

`run_owner_historical_evidence_trial` is the owner-facing wrapper around the
evidence pack loader. It keeps the import contract unchanged:

- local sanitized daily CSV only,
- no downloader,
- no network,
- no broker,
- no orders,
- no credentials,
- no profitability claim,
- no live-readiness claim.

If no owner manifest is supplied, the wrapper returns
`NoOwnerEvidencePackFound` and an action checklist instead of creating fake
data. If a source is unsafe, the source remains visible as rejected whenever
the manifest itself can be parsed safely.

The local-candidate wrapper checks only the approved manifest names in
`data/historical/evidence_packs/`. It can emit the existing rendered triage
text below the ignored `reports/local/evidence_trials/` directory after a
private-marker safety check. This remains local filesystem behavior only.

## Autonomous Acquisition Relationship

Manual CSV import remains the offline replay, audit, and reproducibility path.
The autonomous read-only acquisition broker uses separate normalized snapshots
and can replay them without network access; it does not reinterpret local CSV
fixtures as live provider evidence.
