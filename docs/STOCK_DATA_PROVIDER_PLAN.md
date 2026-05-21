# Stock Data Provider Plan

Sprint 18 keeps stock-provider work conservative.

## Current state

- KRX daily snapshot normalization exists through `import-krx-snapshot`
- no authenticated broker/account stock API is enabled
- no credentials are stored in the repo

## Deferred plan

1. keep KRX/KIS adapters market-data-only
2. require deterministic fixture coverage before enabling a provider
3. refuse account/trading/auth scope
4. keep collected stock data on the same canonical CSV + provenance + preflight path
