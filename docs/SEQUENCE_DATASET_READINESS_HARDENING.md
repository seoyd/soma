# Sequence Dataset Readiness Hardening

Sprint 61 prepares a small bounded sequence dataset export without adding Mamba runtime work.

The readiness bundle includes:

- `SequenceWindowExportPreview`
- `FeatureSchemaLockDraft`
- `LabelAlignmentAuditReport`
- `NoLookaheadSequenceProof`
- `SequenceStorageBudgetReport`

Commands:

```bash
cargo run --quiet --bin soma_experiment -- sequence-readiness-hardening --config examples/soma_sequence_readiness_hardening.toml
cargo run --quiet --bin soma_experiment -- sequence-window-preview --config examples/soma_sequence_window_preview.toml
cargo run --quiet --bin soma_experiment -- no-lookahead-sequence-proof --config examples/soma_no_lookahead_sequence_proof.toml
```

This remains export-prep only. It does not add training, inference, or live trading.

