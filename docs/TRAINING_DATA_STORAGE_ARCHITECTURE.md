# Training Data Storage Architecture

Sprint 78 freezes the storage layout needed before any model-training path can exist.

Directory layout:

- `data/raw/`
- `data/canonical/`
- `data/features/`
- `data/labels/`
- `data/sequences/`
- `data/predictions/`
- `data/model_cards/`
- `data/evaluations/`
- `data/registry/`

The architecture requires manifests and lineage for:

- feature schema hash
- label manifest hash
- split policy
- no-lookahead proof
- provenance refs
- preflight refs

The lineage chain is deterministic from raw ingest through evaluation, and the versioning policy stays explicit so schema, label, split, provenance, and preflight changes force a new dataset version.

