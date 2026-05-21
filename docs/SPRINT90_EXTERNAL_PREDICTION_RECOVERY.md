# Sprint 90 External Prediction Recovery

Sprint 90 follows Sprint 89 because `CandleExpansionOps` was already reduced and the remaining queue now starts with `ExternalPrediction`.

This pass stays narrow and honest:

1. reduce only the `ExternalPrediction` family,
2. keep schema, model-card, duplicate, probability, forbidden-column, and runtime-deferred guards explicit,
3. keep no-run and full-workspace outcomes separate.

`cargo test --workspace --no-run --quiet` is still compile-only evidence. Full recovery only exists when `cargo test --workspace --quiet` actually finishes and passes. No Sprint 90 artifact should imply a fake pass.
