# Prediction History Expansion

Sprint 72 extends prediction history conservatively from local CSV/model-card evidence without turning the system into a training or runtime inference flow.

## Inputs

- existing prediction history pack or Sprint 71 evidence gap context
- local prediction CSV files
- local model cards
- local sequence export manifest

## Conservative behavior

- missing sequence context remains visible
- added rows narrow evidence gaps but do not certify promotion
- the output may still remain `StillNeedMorePredictions`

## Main command

```bash
cargo run --quiet --bin soma_experiment -- prediction-history-expand --config examples/soma_prediction_history_expand.toml
```
