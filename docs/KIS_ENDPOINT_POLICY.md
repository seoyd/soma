# KIS endpoint policy

- Allowlist: OAuth token, domestic/overseas market-data price and quote categories only.
- Denylist: all broker/order/account/balance/holdings/execution categories.
- Unknown endpoints are denied by default.
- CLI: `soma-experiment kis-endpoint-policy --config examples/soma_kis_endpoint_policy.toml`.
