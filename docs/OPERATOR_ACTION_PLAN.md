# Operator action plan

Sprint 26 generates a deterministic operator action plan that tells the human operator what to do next without exposing any secret values.

## Required env vars

- `ALPHAVANTAGE_API_KEY` for bounded US equity evidence
- `KRX_API_KEY` for bounded Korean equity evidence
- `KRX_ENDPOINT_TEMPLATE` for Korean equity endpoint routing

Only env var **names** appear in reports. Secret values must stay local and must not be committed.

## Example commands

```bash
cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml
cargo run --bin soma_experiment -- official-acquire --config examples/soma_official_evidence_acquisition_crypto_only.toml
cargo run --bin soma_experiment -- official-acquire --config examples/soma_official_evidence_acquisition_multi_venue.toml
```

## Research-only warning

The action plan may suggest collection and evidence commands, but it must never suggest live trading, broker, order, or account operations.
