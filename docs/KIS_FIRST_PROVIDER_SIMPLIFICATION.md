# KIS First Provider Simplification

Sprint 52 makes KIS the default primary provider for Korean and US equity **market-data-only** flows.

- **KIS primary**: operational default for bounded Korean/US equity market data.
- **KRX reference/fallback**: retained for Korean equity validation and continuity.
- **AlphaVantage fallback**: retained for bounded US fallback use.
- **yfinance research-only**: kept visible for research and diagnostics only.
- **Upbit crypto-only**: optional and never promoted into official non-crypto evidence.

This is an operational priority change, **not** a performance proof. Broker, order, account, balance, holdings, and position APIs remain out of scope.
