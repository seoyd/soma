# Official data source recommendations

Sprint 29 defines a conservative provider stack for bounded, local-file-only market-data onboarding.

## Korean equities

Recommended priority:

1. `KrxOpenApi`
2. `DataGoKrFscStockPrice`
3. `KisMarketDataOnly`
4. `KoscomProfessional` (optional/professional)

Notes:

- KRX is the primary official exchange path.
- data.go.kr is the public-government fallback with fixture parser support.
- KIS is **market-data-only**; order/account endpoints remain forbidden.
- Koscom remains documented/professional only in this sprint.

## US equities

Recommended priority:

1. `AlphaVantage`
2. `AlpacaMarketData`
3. `PolygonProfessional`
4. `NasdaqDataLink`

Notes:

- AlphaVantage remains the default compact bounded path.
- Alpaca is limited to historical bars fixture/parser v0 and market-data-only use.
- Polygon and Nasdaq Data Link are provider cards plus credential guidance unless a trivial fixture path exists.

## Crypto

Recommended priority:

1. `Upbit`
2. `Binance` (deferred)
3. `Korbit` (optional/deferred)

## Research-only reminder

- `yfinance` remains research-only.
- `yfinance` is never counted as official readiness.
- Provider availability does not imply data quality, model usefulness, or live trading readiness.

