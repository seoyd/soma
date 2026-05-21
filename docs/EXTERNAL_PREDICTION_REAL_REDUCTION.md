# External Prediction Real Reduction

`ExternalPredictionRealReductionConfig` keeps Sprint 90 local-only and fixes the target family to `ExternalPrediction`.

The Sprint 90 reduction reports separate:

1. assertion migration from donor-lineage tests into `tests/external_prediction_family_suite.rs`,
2. fixture/setup reduction through shared harness reuse,
3. preservation of schema, model-card, duplicate, probability, and forbidden-column guarantees.

Historical donor filenames remain explicit in the plan even when the exact file is already absent from the current tree.
