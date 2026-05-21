# Sprint 30 report

## Summary

Sprint 30 adds explicit provider freshness, cost, entitlement, strategy-compatibility, recommendation, and reality-report layers so Soma Zero cannot confuse EOD, delayed, realtime, IEX-only, approval-pending, and research-only data.

## Implemented

- freshness tiers and provider freshness profiles
- cost/approval profiles
- entitlement preflight
- strategy-data compatibility gate
- provider recommendation engine
- provider reality report
- `provider-reality`, `strategy-data-check`, `provider-recommend` CLI
- examples, docs, tests

## Reality interpretation

- KRX approval pending is explicit and blocks Korean official collection claims.
- AlphaVantage compact/free is treated as EOD/historical, not realtime.
- Alpaca Basic is treated as IEX-limited realtime, not full-market SIP coverage.
- yfinance stays research-only.

## Risk review

- no live trading
- no broker/order/account APIs
- no runtime LLM
- no Mamba runtime
- no secret persistence
- no yfinance official readiness

## Next sprint recommendation

Drive the next official-data workflow from the new reality report: close KRX approval/auth gaps, keep AlphaVantage in EOD mode, and decide whether paid SIP/Polygon coverage is necessary before any US realtime research claims.

