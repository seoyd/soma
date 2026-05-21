# Research Campaign

Sprint 12 adds a deterministic campaign layer above Sprint 11 batch matrices.

## Concept

A campaign runs one or more local batch matrix configs, aggregates their evidence, archives a snapshot, optionally compares against a previous campaign, and emits a hardened readiness decision.

## Scope

- local TOML configs only
- research / paper / backtest only
- no live API, broker, WebSocket, or runtime LLM path
- Python remains optional; baseline-only campaigns still work without it

## Flow

1. Load matrix configs.
2. Run each matrix with `BatchExperimentRunner`.
3. Build a campaign aggregate.
4. Archive a deterministic snapshot in the evidence store.
5. Compute diff/regression state if a previous report exists.
6. Emit a conservative readiness decision.

## Determinism

- stable ordering by matrix ID
- stable fingerprints from config/report content
- no wall clock unless explicitly passed through config
