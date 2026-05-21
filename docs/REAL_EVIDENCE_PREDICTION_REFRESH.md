# Real Evidence Prediction Refresh

Sprint 75 refreshes offline prediction CSV coverage after Sprint 74 attached validated KIS real evidence. The goal is to replace the remaining stale prediction requirement with a local, deterministic, research-only refresh for `ext-model-b:1.0.0`.

The refresh remains local-only and static/read-only. Prediction CSVs must match the real sequence context, provide sane probabilities, avoid duplicate `sequence_id` rows, and keep model-card compatibility visible. This does **not** add model training, runtime inference, or live trading.
