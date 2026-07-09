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
