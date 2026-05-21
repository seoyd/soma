# Sequence Core Candidate Registry

Sprint 79 adds a shared **sequence-core candidate registry** for:

- `Mamba3Fin`
- `GatedDeltaNet`

The registry is **contract-only**. It checks common input tensor compatibility, shared prediction heads, and risk integration requirements. It does **not** implement runtime inference or model training.

The comparison plan remains offline/research-only and exists to keep both neural candidates inside the same storage and evaluation boundary before any runtime discussion.

