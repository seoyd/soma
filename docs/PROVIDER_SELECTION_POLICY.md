# Provider selection policy

Sprint 29 adds a deterministic, bounded provider selection layer.

## Priority and fallback

### Korean equities

1. `KrxOpenApi`
2. `DataGoKrFscStockPrice`
3. `KisMarketDataOnly`
4. `KoscomProfessional`

### US equities

1. `AlphaVantage`
2. `AlpacaMarketData`
3. `PolygonProfessional`
4. `NasdaqDataLink`

### Crypto

1. `Upbit`
2. `Binance`
3. `Korbit`

## Rules

- Korean equity readiness never falls back to `yfinance`.
- US equity selection may end in `ResearchOnlyFallback`, but that is not official readiness.
- Professional paid sources are allowed only when explicitly enabled.
- Deferred/documented-only providers stay operator-action items until upgraded.
- Full-history and all-symbol collection remain denied by default.

## Conservative interpretation

- Provider selection is acquisition readiness only.
- Acquisition readiness is not data-quality readiness.
- Data-quality readiness is not model usefulness.
- None of this implies live trading or profitability.

