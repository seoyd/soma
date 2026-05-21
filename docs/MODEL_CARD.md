# Model Card

Sprint 08 adds deterministic model card generation for local research runs.

## Core fields

- model ID and model kind
- training script version
- feature schema version/hash
- dataset path
- prediction path
- fold IDs
- train / validation / test row counts
- target label summary
- cost model summary
- backend used
- hyperparameters
- threshold selection method
- validation metrics
- known limitations
- leakage controls
- optional created_at_ms if passed explicitly

## Leakage controls

Model cards must record:

- train-only fitting
- validation-only threshold tuning
- test-only prediction/evaluation
- label-column exclusion from feature inputs

## Limitations

The model card must warn that Rust can validate prediction schema and alignment, but it cannot prove that the external training process itself was leakage-free unless provenance is supplied.
