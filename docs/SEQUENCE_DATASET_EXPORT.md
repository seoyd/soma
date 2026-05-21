# Sequence Dataset Export

Sprint 62 adds the first bounded local sequence dataset export bundle.

- export stays local-only and deterministic
- output is `dataset.csv` plus manifests and reports
- export does **not** mean training, live readiness, or model usefulness
- Mamba runtime remains deferred

Primary command:

```bash
cargo run --quiet --bin soma_experiment -- sequence-dataset-export --config examples/soma_sequence_dataset_export_small.toml
```

