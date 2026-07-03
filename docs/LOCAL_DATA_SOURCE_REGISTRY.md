# Local Data Source Registry

## Purpose

`LocalDataSourceRegistry` defines deterministic schema profiles for sanitized
local CSV text. It contains no URL, downloader, provider client, account,
broker, order, or cancellation configuration.

Supported source kinds:

- `SyntheticFixture`,
- `KoreanStockCsv`,
- `UsStockCsv`,
- `BtcCryptoCsv`.

`Unknown` is always rejected.

## Profiles

Every profile declares required and optional columns, timestamp handling,
numeric scales, strict single-symbol policy, allowed local source markers,
private-marker rejection, expected cadence, cadence tolerance, quality
thresholds, and reason codes.

| Kind | Timestamp | Additional optional fields |
| --- | --- | --- |
| Synthetic fixture | `timestamp_ms` | `trade_value`, `source` |
| Korean stock | `timestamp_ms` or synthetic UTC `date` + `time` | `trade_value`, `market`, `source`, `currency` |
| US stock | `timestamp_ms` or synthetic UTC `date` + `time` | `adjusted_close`, `trade_value`, `market`, `source`, `currency` |
| BTC crypto | `timestamp_ms` | `quote_volume`, `trade_count`, `source`, `exchange`, `currency` |

`adjusted_close` is validated but never replaces canonical `close`.
`quote_volume` becomes optional canonical trade value. Price and volume scales
are explicit and default to `1.0`.

## Validation

Profiles reject:

- unknown kinds,
- empty or duplicate schema fields,
- non-positive or non-finite scales,
- disabled private-marker checks,
- URL text,
- broker or order endpoint text,
- forbidden private columns.

CSV parsing additionally rejects malformed headers and rows, unsafe source
markers, multiple symbols, duplicate or reversed timestamps, invalid OHLC,
non-finite values, and private material.

## Normalization

Accepted rows normalize to `HistoricalReplayDataset` with an internal
`synthetic:<source-kind>` marker, then to the existing `CandleSeries`.
`LocalDataQualitySummary` records row counts, timestamps, symbol, source kind,
trade-value presence, monotonicity, and close range.

The normalized dataset enters the existing three-agent paper replay and owner
report path. It cannot bypass Chair or Risk Governor.

## Batch Replay

`run_local_dataset_batch_replay` resolves every batch item through this
registry. A source kind and non-empty profile name must identify the same
canonical profile. Accepted sources proceed to the historical adapter;
rejected sources retain their reason codes in the source performance table.

Strict mode stops on the first rejection. Continuing mode records the
rejection and proceeds to later sources. Neither mode silently skips invalid
data. The batch is bounded by source count, rows per source, and an exact
three-agent limit.

Endpoint-like metadata or columns, including URL, generic endpoint, broker
endpoint, and order endpoint markers, are rejected before profile parsing.
Accepted sources retain the resolved profile for consistency diagnostics; a
profile-name mismatch is visible in the rejected diagnostic row.

## Boundaries

All profiles are local, read-only, sanitized, and paper-only. No registry API
performs file discovery, network IO, live download, exchange access, Toss
access, broker access, or order execution.

A future real-data adapter would require a separate reviewed import boundary.
It must sanitize into this schema before registry parsing; no endpoint belongs
in a source profile.

Current cadence contracts are local 60-second fixture rules. They do not
implement exchange sessions, holidays, timezones, or weekend handling. Korean
and US profiles explicitly report that calendar validation is deferred.
