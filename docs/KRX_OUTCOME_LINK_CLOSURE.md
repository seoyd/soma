# KRX Outcome Link Closure

Sprint 50 closes KRX outcome-link gaps only when local KRX candles support safe no-lookahead references.

## Outcome linkage flow

1. load official-ready KRX candle series
2. require future-window sufficiency from the preregistered barrier profile registry
3. generate bounded KRX outcome links
4. generate `NoTrade` and `RiskDenied` counterfactual counts only after outcome links exist
5. keep downstream diversity, committee, and core summaries conservative when `outcome_links_after=0`

## Conservative behavior

- no-lookahead unsafe rows are rejected
- missing future windows keep `StillMissingFutureWindows`
- missing outcome links keep committee/core blocked
- sparse KRX rows do not imply profitability or live readiness

The closure report is still research-only, paper-only, and market-data-only.
