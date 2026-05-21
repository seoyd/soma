# KIS market-data-only activation

- Scope: research-only, paper-only, bounded KIS market-data workflows.
- Required env vars for REST collection: `KIS_APP_KEY`, `KIS_APP_SECRET`, `KIS_BASE_URL`.
- Optional realtime env var: `KIS_WS_APPROVAL_KEY`.
- Forbidden everywhere: broker, order, account, balance, position, holdings, buying-power, cancel, execution-notification paths.
- Primary commands:
  - `soma-experiment kis-auth-readiness --config examples/soma_kis_auth_readiness.toml`
  - `soma-experiment kis-collection-plan --config examples/soma_kis_collection_plan_missing_auth.toml`
  - `soma-experiment kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml`
