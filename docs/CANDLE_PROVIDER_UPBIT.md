# Upbit Candle Provider

The first Sprint 18 provider is Upbit public candle collection.

## Supported timeframes

- `1m`
- `5m`
- `15m`
- `1h`
- `1d`

## Constraints

- public market-data only
- no API key
- no order/account endpoints
- request size capped at `200`
- reverse-ordered provider payloads are normalized to ascending canonical CSV

## Notes

- gaps are preserved, filled, or rejected according to `FillMissingPolicy`
- raw provider bodies are archived before canonicalization
- collected local copies use `OfficialApiCollected` provenance
