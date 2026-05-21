# Workspace No-Run Recovery

`cargo test --workspace --no-run --quiet` is a compile-only acceptance-recovery checkpoint. It is useful because it narrows workspace-scale build blockers without pretending the full test suite has passed.

No-run completion is **not** full acceptance. A long-running or timed-out no-run attempt must be reported as blocked, still compiling, or timed out rather than passed.

Sprint 106 keeps long-running compile handling explicit so the report can stay honest even when the workspace remains open.
