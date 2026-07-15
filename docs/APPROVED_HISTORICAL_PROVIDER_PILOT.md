# Approved Historical Provider Pilot

The Sprint 32 pilot supports one provider: Upbit public quotation daily candles for BTC-market evidence. It is a deliberately narrow, read-only path; it is not an account, order, portfolio, or streaming integration.

## Official Contract Lock

The selected operation is `GET https://api.upbit.com/v1/candles/days`. Upbit documents `market` as required, `to` as an exclusive ISO-8601 UTC boundary, and `count` with a maximum of 200. The provider creates a day candle only when trades occurred, so missing days are retained as an evidence limitation rather than fabricated bars. The documented candle group limit is ten requests per second; this pilot plans one bounded request, with only the configured bounded retry policy available for a transport failure.

## Local Configuration And Consent

Copy `config/examples/historical_provider.example.toml` to `config/local/historical_provider.local.toml`. The local configuration is ignored by Git and contains the configured market symbol, bounded UTC range, page size, target row count, page limit, timeout, retry cap, response-size cap, and local snapshot directory. It contains no credential fields because the quotation operation is public.

The command is disabled unless both the local configuration sets `network_consent = "manual_local_smoke"` and `manual_smoke_enabled = true`, and the operator passes `--allow-network`.

```bash
cargo run -- --historical-provider-smoke-config config/local/historical_provider.local.toml --allow-network
```

The command prints only a sanitized status, provider ID, row count, page count, snapshot ID, digest prefix, inventory result, campaign sufficiency, and reason codes. It does not print a local path or raw response.

## Transport And Data Boundary

The adapter permits HTTPS GET only to the fixed daily-candle endpoint, rejects invalid market symbols before transport, uses bounded curl timeout and response size, and does not follow an arbitrary URL. It never sends credentials, uses account or order APIs, logs a response body, polls in the background, or runs as part of Cargo tests.

The response is parsed into canonical daily OHLCV rows, sorted chronologically, and rejects wrong symbols, empty data, duplicate timestamps, invalid timestamps, non-finite values, invalid OHLC, and negative volume. The existing Data Acquisition Broker then creates a credential-free immutable `DataSnapshot`.

## Local Snapshot And Learning Boundary

The first verified page is atomically written below `data/local_snapshots/upbit/`, re-read, and digest-verified. Raw provider payloads are never stored. If more rows are required, the same endpoint is paged backwards with an exclusive oldest-candle cursor, a configured page limit, repeated-cursor/page detection, and deterministic duplicate handling. A conflicting duplicate rejects the merged result; an identical duplicate is visibly deduplicated. The merged snapshot is written and verified separately. Snapshot inventory and the existing Momentum campaign remain separate: a campaign starts only when the existing chronological sufficiency gates are met.
