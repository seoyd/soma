# Sprint 29 report

## Summary

Sprint 29 finalized the official provider onboarding layer for Korean and US equities without weakening the repository's safety boundaries.

Implemented:

- provider catalog and priority map
- credential profiles with env-var-name-only policy
- provider selection policy
- data.go.kr fixture parser v0
- KIS market-data-only request builder/stub
- Alpaca historical bars fixture parser v0
- official provider readiness report
- `provider-catalog`, `provider-readiness`, `provider-select` CLI

## Provider readiness interpretation

- This sprint measures **provider acquisition readiness only**.
- It does **not** claim data quality readiness.
- It does **not** claim model usefulness or profitability.
- It does **not** claim live trading readiness.

## Recommended operator actions

1. Configure `KRX_API_KEY` and `KRX_ENDPOINT_TEMPLATE` for primary Korean-equity onboarding.
2. Configure `ALPHAVANTAGE_API_KEY` for bounded US-equity onboarding.
3. Add `DATA_GO_KR_SERVICE_KEY` for Korean fallback coverage.
4. Add `KIS_APP_KEY` / `KIS_APP_SECRET` only for market-data-only fallback.
5. Treat Polygon/Nasdaq/Koscom as documented or paid follow-up providers until explicitly upgraded.

## Risk review

- No runtime LLM was added.
- No live trading path was added.
- No broker/order/account API was added.
- No secret values are stored or printed by the new readiness flow.
- `yfinance` remains research-only.

## Tests

- provider catalog
- credential profiles
- provider selection
- data.go.kr parser/import status
- KIS market-data-only stub
- Alpaca parser/import status
- provider readiness report
- provider readiness CLI safety

## Next sprint recommendation

Use the new provider readiness report to drive the next bounded official evidence run, then deepen real official dataset collection quality for the newly onboarded Korean and US provider stack.

