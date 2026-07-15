# Momentum Real Historical Evidence

## Evidence Classification

Real historical evidence must be immutable, sanitized, credential-free daily OHLCV with a valid digest, chronological timestamps, accepted quality metadata, valid OHLCV values, and a provider or owner-sanitized provenance record. Mock inputs and unmarked local replay data are synthetic/test-only and are excluded from market evidence and verdict counts.

The inventory reports each candidate as `Ready`, `InsufficientRows`, `InvalidChronology`, `InvalidDigest`, `Mutable`, `Unsafe`, `SyntheticTestOnly`, or `UnsupportedDataset`. It preserves rejected snapshot identifiers and never substitutes a fixture for an unavailable market series.

## Existing Snapshot First

The inventory runs before any provider decision. Ready snapshots are grouped by market and symbol, while insufficient, corrupt, mutable, unsafe, unsupported, and synthetic inputs remain visible as rejected evidence.

For a single approved BTC series, campaign sufficiency is computed from the existing campaign configuration: configured history minimum, two purge gaps, train/validation/test rows, step rows, and required evaluated windows. A row target is not evidence of campaign readiness by itself.

## Provider Gate

When inventory is insufficient, the gate accepts only an enabled, approved, read-only broker provider with daily OHLCV support for a configured market and universe. It does not invent URLs, credentials, endpoints, symbols, or provider responses. If no provider qualifies, it returns `NoApprovedHistoricalProviderConfigured`.

For a qualifying provider, the existing broker receives one configured daily request per symbol. It remains responsible for response-size limits, schema and numeric validation, credential redaction, immutable snapshot storage, and digest verification. No account, order, cancel, streaming, quote-polling, or raw-response path is part of this flow. A frozen evidence pack is created only after the broker work has completed; training makes no provider call.

## Evidence Pack And Campaigns

Accepted series form a deterministic frozen pack with a digest. Provider calls are outside the pack and never occur during training. Each series uses its own existing walk-forward campaign, frozen encoder, train-only normalization, cold/warm comparison, baselines, drift reporting, and ShadowOnly versions.

Before a campaign consumes a series, it re-verifies the same canonical semantic dataset digest used by snapshot acquisition and frozen-pack verification. The campaign result includes an ordered sanitized safety trace with the first rejecting gate and exact reason code. A valid immutable series may run offline Shadow evaluation while promotion, voting, and execution remain separately blocked.

Probability-collapse forensics uses the same frozen series without provider access. Candidate registration is fixed before validation, representation fitting is train-only, and only the selected validation candidate may open the future test partition.

The current single-series path reports a per-series outcome only when a real frozen pack and campaign result exist. It cannot establish broad cross-market value, profitability, promotion readiness, or live readiness.

## Aggregate Verdict

Cross-series evidence counts Mamba-versus-constant windows, Mamba and linear series wins, Brier deltas, high-confidence-error deltas, drift, and warm/cold outcomes. A positive Mamba verdict requires configurable repeated evidence across enough real series and windows; a single favorable series is insufficient. Synthetic data cannot influence this verdict.

When evidence is absent, synthetic-only, short, rejected, or a provider harvest fails, the result remains a reason-coded insufficiency rather than a market conclusion. The report identifies accepted/rejected series, per-series campaign status, Mamba-versus-linear metrics, warm-start status, drift, the aggregate verdict, and the next required evidence.

## Boundaries

The result is evidence only. It does not claim profitability, market edge, live readiness, promotion readiness, active voting, execution authority, or official Mamba conformance. Official oracle conformance remains blocked.
