# Offline Evidence Attachment

Sprint 72 adds a local-only bundle that attaches conservative offline evidence on top of the Sprint 71 operator briefing state.

## What it includes

- attachment registry with `Official`, `ResearchOnly`, and `DiagnosticOnly` source classes
- secret-like string scan before fragments are written
- bounded artifact count and byte limits
- static HTML / JSON / TXT outputs under `target/soma_offline_evidence_attachment/<attachment_id>/`

## What it does not do

- broker, order, or account access
- runtime inference
- training or model mutation
- live execution

## Main command

```bash
cargo run --quiet --bin soma_experiment -- offline-evidence-attach --config examples/soma_offline_evidence_attach.toml
```

The bundle stays deterministic, paper-only, research-only, and read-only.
