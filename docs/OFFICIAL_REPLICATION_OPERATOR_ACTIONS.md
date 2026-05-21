# Official replication operator actions

Sprint 39 emits a bounded operator action plan alongside replication results.

## Properties

- action IDs are deterministic and deduplicated
- env vars are listed by name only
- suggested commands remain local-only and research-only
- blockers explain missing auth, provenance, preflight, candle, or reference gaps

## Typical actions

- run provider readiness / provider reality
- set `KRX_API_KEY`, `KRX_ENDPOINT_TEMPLATE`, `ALPHAVANTAGE_API_KEY`
- provide local official candle JSON or canonical CSV files
- rebuild committee references
- rerun the official committee benchmark after official sufficiency passes
