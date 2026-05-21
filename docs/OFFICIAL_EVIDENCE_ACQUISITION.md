# Official evidence acquisition

Sprint 26 adds a bounded auth-aware acquisition layer on top of Sprint 25 expansion.

## Flow

1. run provider auth preflight
2. decide which providers are actually ready
3. build a bounded collection plan from those ready providers
4. collect only local research data within symbol, row, request, and byte limits
5. compare previous and current collection readiness
6. optionally run official evidence expansion
7. emit operator actions and next commands

## Missing auth handling

- `Upbit` can run without auth and stays valid for crypto-only evidence
- `KrxOpenApi` runs only when both `KRX_API_KEY` and `KRX_ENDPOINT_TEMPLATE` are ready
- `AlphaVantage` runs only when `ALPHAVANTAGE_API_KEY` is ready
- missing auth is reason-coded and produces operator actions instead of being treated as success

## Coverage interpretation

- Upbit-only remains `CryptoOnly`
- Korean equity claims need KRX-ready official data
- US equity claims need AlphaVantage-ready official data
- auth-ready alone is not enough; collection still has to pass bounded evidence and preflight gates

## yfinance note

Sprint 26 does **not** add `yfinance` to the runtime path. The current rules keep the runtime Rust-only and avoid adding a Python-backed provider shortcut. For no-auth operation, this sprint uses the existing Upbit public path and explicit operator guidance.
