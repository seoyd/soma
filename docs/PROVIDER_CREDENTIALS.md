# Provider credentials

Sprint 29 stores **env var names only**. Secret values must never be persisted or printed.

## Required environment variables

| Provider | Required env vars | Optional env vars | Notes |
| --- | --- | --- | --- |
| KRX Open API | `KRX_API_KEY` | `KRX_ENDPOINT_TEMPLATE` required for live request construction | Primary Korean equity source |
| data.go.kr FSC stock price | `DATA_GO_KR_SERVICE_KEY` | none | Endpoint profile must be approved explicitly |
| KIS market-data-only | `KIS_APP_KEY`, `KIS_APP_SECRET` | `KIS_BASE_URL` | No order/account endpoints |
| AlphaVantage | `ALPHAVANTAGE_API_KEY` | none | Compact bounded default |
| Alpaca market data | `ALPACA_API_KEY_ID`, `ALPACA_API_SECRET_KEY` | none | Historical bars only, no trading/account flow |
| Polygon | `POLYGON_API_KEY` | none | Professional paid provider card |
| Nasdaq Data Link | `NASDAQ_DATA_LINK_API_KEY` | none | Professional paid provider card |

## Rules

- Do not commit or print credential values.
- Missing auth is an operator action, not silent success.
- Missing endpoint metadata must be reason-coded.
- Secrets stay outside TOML configs and repository files.

## Suggested setup flow

1. Export env vars locally.
2. Run `provider-readiness` with a local config.
3. Review missing-auth and deferred-provider actions.
4. Only then run bounded collection/evidence flows.

