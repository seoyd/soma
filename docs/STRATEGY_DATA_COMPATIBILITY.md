# Strategy-data compatibility

Sprint 30 adds a compatibility gate so the repository cannot validate the wrong strategy with the wrong data.

## Allowed examples

- EOD/historical data can support `EodSwing` and `DailyPortfolioResearch`
- Alpaca IEX-limited realtime can support bounded realtime research with explicit limitations
- yfinance can support `SourceComparison` and `ModelPrototypeResearch`

## Blocked examples

- EOD data cannot validate `RealtimeScalping`
- EOD data cannot validate `RealtimeExecutionSimulation`
- `Delayed15m` data cannot validate realtime execution simulation
- yfinance cannot satisfy official readiness
- IEX-only data cannot claim full-market US coverage

## Practical interpretation

- Strategy compatibility is not profitability proof
- Provider readiness is not model readiness
- None of these gates imply live trading readiness

