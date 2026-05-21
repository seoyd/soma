# Equity Market Data Providers

Sprint 19 extends the collector with bounded equity-provider support.

## Implemented now

- `KrxOpenApi` daily/EOD provider foundation
- `AlphaVantage` compact daily provider
- `MockFixture` replay path for offline tests

## Deferred or stubbed

- `Alpaca` historical bars: metadata/stub only
- `KoreaInvestmentMarketData`: deferred metadata only
- no trading/account/broker endpoints

## Boundary

- market-data-only collection
- local file output only
- bounded request/row/byte budgets
- preflight is still required before evidence reuse
- no live trading readiness claim
