# External Prediction Import V2

Sprint 63 adds a bounded local CSV import lane for externally generated research predictions.

- required columns: `sequence_id`, `model_id`, `model_version`, `prediction_timestamp_ms`
- optional validated columns: probabilities, expected return/drawdown, confidence, predicted label, rank score, reason code
- `sequence_id` must match the exported sequence dataset
- duplicate sequence predictions per `model_id + model_version` are rejected
- account, order, and secret-style columns are rejected
- model card validation remains required by default

This import path is **research-only**, **paper-only**, **local-only**, and does **not** mean training or live inference.

