# Safe Test Binary Consolidation

Sprint 106 allows only **safe** consolidation planning.

- No assertion deletion
- No safety sentinel deletion
- Move assertions before removing any target
- Keep sample-backed estimates separate from measured deltas

If a family is high-risk, safety-heavy, CLI-safety-related, or determinism-sensitive, the plan keeps it isolated unless an equal or stronger guard replaces it.
