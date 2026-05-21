# Official data signal evidence

Sprint 24 only counts **official-ready** evidence toward signal claims.

## Official-ready requirements

- collected from bounded official-market data flows
- local canonical CSV exists
- preflight result is ready for evidence
- dataset export keeps feature/label separation
- evidence stays provenance-aware

## Coverage interpretation

- Upbit-only coverage is labeled `CryptoOnly`
- missing KRX auth means no Korean-equity claim
- missing AlphaVantage or Alpaca auth means no US-equity claim
- mock fixtures are excluded from official-ready selection

## Why this matters

The benchmark question is not “can any CSV produce numbers?” It is:

> after core hardening, can bounded official evidence support a calibrated, cost-aware, risk-governed external tabular candidate?

That means evidence quality and coverage are part of the result, not side notes.
