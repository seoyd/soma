# KRX Provider

`KrxOpenApi` is a Sprint 19 daily/EOD provider foundation.

## Scope

- Korean equity OHLCV
- daily/EOD only
- no intraday/live trading
- no broker/account endpoints

## Auth

- requires auth key
- service approval may be required
- env-var names are stored, never secret values

## Implementation note

Exact live endpoint details are not guessed silently. The collector supports a configurable endpoint template and fully offline fixture parsing.
