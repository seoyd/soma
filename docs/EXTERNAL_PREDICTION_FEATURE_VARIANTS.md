# External Prediction Feature Variants

Sprint 90 records the repeated feature variants that appear in the current ExternalPrediction surface.

- `default+test-fixtures` is safe to reduce when it only removes duplicated setup.
- `default+research-only` remains explicit when collapsing it would blur safety or runtime-deferred boundaries.

Feature collapse is safe only when it does not hide schema/model-card checks, runtime deferral, or evaluation restrictions. If that safety line is unclear, the variant must remain separate.
