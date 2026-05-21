# AlphaVantage Provider

`AlphaVantage` is the first bounded US equity provider in Sprint 19.

## Scope

- daily compact by default
- optional intraday fixture parsing path
- no trading/account endpoints

## Auth

- API key via env var name
- query-param auth
- tests do not require a real key

## Compact policy

- compact mode is the default safe path
- row caps still apply after parsing
- full-history style requests remain blocked unless explicitly allowed
