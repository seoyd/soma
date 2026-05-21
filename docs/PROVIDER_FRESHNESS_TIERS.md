# Provider freshness tiers

Sprint 30 makes provider freshness explicit so Soma Zero does not conflate EOD, delayed, realtime, and research-only data.

## Tiers

- `Eod`: end-of-day research data
- `Historical`: backfilled or historical series without realtime claim
- `Delayed15m`: delayed market-data entitlement
- `RealtimeIex`: realtime but IEX-limited coverage
- `RealtimeSip`: broader US SIP-style coverage
- `RealtimeExchangeOfficial`: exchange/broker market-data path with official entitlement
- `RealtimeCryptoPublic`: public crypto realtime/recent candles
- `ResearchOnly`: unofficial supplemental data such as yfinance
- `Unknown`: not classified yet

## Key distinctions

- EOD is not realtime.
- Delayed data is not realtime execution data.
- `RealtimeIex` is not the same thing as full-market SIP coverage.
- Research-only data is never official readiness.

