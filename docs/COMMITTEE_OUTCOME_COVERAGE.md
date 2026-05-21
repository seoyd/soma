# Committee outcome coverage

Sprint 37 adds a conservative coverage layer for committee benchmarks.

Coverage cells are grouped by source kind, market, symbol, derived timeframe label, and horizon. Each cell counts:
- rows loaded,
- outcome links,
- baseline/external references,
- built NoTrade and RiskDenied counterfactuals,
- official vs research-only vs fixture-only vs crypto-only rows,
- no-lookahead-safe rows,
- missing evidence gaps.

Interpretation rules:
- official rows are the only readiness-eligible rows,
- yfinance rows remain research-only,
- fixture rows remain architecture-test-only,
- crypto-only coverage cannot imply stock readiness,
- healthy coverage is still research-only and does not imply profitability or live trading readiness.
