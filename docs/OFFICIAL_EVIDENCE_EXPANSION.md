# Official evidence expansion

Sprint 25 adds a bounded orchestration layer for expanding official evidence without turning the repository into a live or broad-market collector.

## Flow

1. provider auth preflight
2. bounded official collection or existing report load
3. venue coverage report
4. core-check-gated core benchmark
5. evidence delta and storage delta
6. conservative next-step recommendation

## Guardrails

- local paths only
- bounded symbol, row, request, and byte budgets
- no broker, order, or account APIs
- no live trading
- no runtime LLM
- no all-symbol or full-history expansion

## Readiness interpretation

- `MissingAuth`: required venue access is still blocked
- `MissingOfficialData`: not enough ready official evidence exists yet
- `CryptoOnly`: official evidence is real but limited to crypto
- `MoreOfficialEvidence`: coverage is expanding, but breadth is still the next priority
- `HoldCurrentScope` or improvement-oriented recommendations stay conservative and research-only

## Delta reports

The expansion report includes:

- official evidence delta against the previous benchmark when available
- storage delta against the current byte budget
- auth setup guides for providers that still need local configuration

## Example commands

```bash
cargo run --bin soma_experiment -- evidence-expand --config examples/soma_official_evidence_expansion_crypto_only.toml
cargo run --bin soma_experiment -- evidence-expand --config examples/soma_official_evidence_expansion_multi_venue.toml
```
