# DEFERRED_REAL_OBSERVATION_EXECUTION

The Sprint 117 runner writes a deterministic bundle under `target/soma_sprint117_deferred_real_observation/<execution_id>/`.

Execution order is fixed:
1. real cargo-json observation
2. real no-run observation
3. real full-workspace observation

Remote paths are rejected. The runner preserves paper-only semantics and never upgrades acceptance from deferred or diagnostic evidence.
