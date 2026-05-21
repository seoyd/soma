# Mamba3Fin readiness

Sprint 22 does **not** implement Mamba3, Mamba3Fin-lite inference, or neural training inside Rust.

This layer asks a narrower question:

> Is the current bounded official-data benchmark strong enough to justify a later **external-first** Mamba3Fin-lite prototype?

## What exists now

- official benchmark reports from Sprint 21
- cross-dataset consistency checks
- sequence dataset sizing and no-lookahead checks
- candidate-spec and escalation gates
- external prediction bridge as the only prototype path

## What is still missing

Current Soma Zero does **not** have:

- Mamba3-style expressive recurrence
- complex-valued state update
- MIMO state-space runtime
- hardware-aware scan / streaming-state runtime
- Rust-native neural inference
- Rust-native neural training

## Why the audit is conservative

- crypto-only evidence is labeled as crypto-only
- missing KRX or AlphaVantage auth blocks broader equity claims
- calibration, drawdown, and risk behavior can veto escalation
- sequence export must stay storage-bounded before any prototype work
- the final gate can approve only an **external prediction file** prototype, never a Rust-native Mamba runtime

## CLI

```bash
cargo run --bin soma_experiment -- mamba-readiness --config examples/soma_mamba_readiness_crypto_only.toml
```

Outputs:

- `mamba_readiness_benchmark_report.json`
- `mamba_readiness_benchmark_report.txt`

These reports are research-only and local-file-only.
