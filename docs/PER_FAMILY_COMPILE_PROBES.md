# Per-Family Compile Probes

Sprint 88 adds per-family compile, no-run, and execution probes for the seven remaining blocker families. These probes are intentionally separate from the full workspace gate.

- `per-family-compile-probe` reports focused compile observations.
- `per-family-no-run-probe` reports compile-only/no-run observations.
- `per-family-execution-probe` reports focused execution observations.

Passing a family probe does **not** mean the workspace passed. The workspace remains accepted only when `cargo test --workspace --quiet` finishes and passes.
