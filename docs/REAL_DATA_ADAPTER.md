# Real Data Adapter

Sprint 09 adds a **local-only** market data adapter layer.

## Scope

- load local OHLCV CSV files
- normalize symbol and timeframe metadata
- validate untrusted input before it reaches evaluation
- output `CandleSeries` compatible with the existing Rust pipeline

## Supported CSV formats

- `GenericOhlcv`
- `BinanceKline`
- `UpbitCandle`
- `KrxOhlcv`
- `Custom { column_map }`

Current timestamp support is intentionally narrow and deterministic:

- `Millis`
- `Seconds`

`Iso8601Utc` and richer parsing are deferred.

## Safety rules

- no live API calls
- no WebSocket feed
- no broker integration
- no silent acceptance of invalid rows in strict mode
- sort/duplicate repair only when explicitly enabled

## Output

The loader produces:

- normalized `CandleSeries`
- `DataQualityReport`
- `DataManifest`
- parse issues and reason codes for invalid input
