# Provider auth preflight

Sprint 25 adds a deterministic auth preflight for official market-data providers before bounded collection runs.

## Scope

- reports env var **names** only
- checks presence or absence only
- never prints or stores raw secret values
- keeps public providers like Upbit as `NotRequired`

## Provider rules

- `Upbit` and other public-only providers stay `NotRequired`
- `KrxOpenApi` requires a key env var and an endpoint template env var
- `AlphaVantage` requires an API key env var
- `Alpaca` stays optional or deferred for Sprint 25

## Status interpretation

- `Ready`: required env var names are configured and present
- `MissingAuth`: one or more required auth env vars are absent
- `MissingEndpointTemplate`: the provider needs a templated endpoint env var and it is absent
- `NotRequired`: public data path, so no auth gate applies
- `Deferred`: optional provider is intentionally not enabled yet
- `UnsafeSecretExposure`: a raw secret-like value was passed where an env var name should have been used

## Safety rules

The report may include names like `KRX_API_KEY` or `ALPHAVANTAGE_API_KEY`, but it must not include the secret values stored in those variables. Missing auth is reason-coded and surfaced as a warning or blocker instead of being silently treated as success.

## CLI

```bash
cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml
```
