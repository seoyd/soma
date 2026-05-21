# External Model Bridge

Sprint 07 adds a deterministic bridge for importing predictions produced outside Rust.

## Why Rust does not train models here

This sprint does **not** train LightGBM, XGBoost, tiny nets, or Mamba-like models inside Rust. Rust stays responsible for:

- feature schema locking
- dataset export
- prediction import and validation
- walk-forward evaluation
- Chair / Risk Governor / outcome replay

## External prediction file concept

External models produce prediction rows keyed to exported dataset rows. Soma Zero then validates and routes those predictions through the existing numeric decision pipeline.

## Model artifact metadata

`ModelArtifactMeta` records:

- model ID
- model kind
- schema version and hash
- optional window summaries
- target label summary
- cost model summary

This records compatibility and provenance metadata, not a live model runtime.

## Schema validation

External predictions are treated as untrusted until validated.

Validation checks:

- feature schema version/hash
- row alignment to dataset rows
- probability bounds
- finite expected return / drawdown

## Prediction import

Sprint 07 supports deterministic CSV import/export and in-memory construction for tests.

## Conservative behavior

If a prediction is missing or invalid:

- return conservative `NoTrade`
- keep Risk Governor above the model
- do not silently coerce invalid values into tradable signals

## Leakage warning

This bridge validates schema and row alignment, but it **cannot prove** that the external training process itself was leakage-free. Training provenance remains an external responsibility for a future sprint.
