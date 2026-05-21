# KIS Auth Closure

Sprint 58 adds a thin, secret-safe closure layer over the existing KIS auth readiness path.

## Goals

- check `KIS_APP_KEY`, `KIS_APP_SECRET`, `KIS_BASE_URL`, and optional realtime approval state
- render env var names and a redacted base-url preview only
- distinguish dry-run readiness from live/realtime readiness
- keep the flow market-data-only and research-only

## CLI

```bash
cargo run --quiet --bin soma_experiment -- kis-auth-close --config examples/soma_kis_auth_close.toml
```

## Outputs

- `kis_auth_closure.json`
- `kis_auth_closure.txt`

## Safety

- no secret values are rendered
- missing auth does not trigger network activity
- readiness does not imply data quality, profitability, or live trading readiness
