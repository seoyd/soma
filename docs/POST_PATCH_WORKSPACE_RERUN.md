# Post-Patch Workspace Rerun

Sprint 107 keeps honest workspace reruns separate from focused validation. The no-run rerun is `cargo test --workspace --no-run --quiet`; the full rerun is `cargo test --workspace --quiet`.

Timeout handling remains explicit. A no-run timeout cannot pass, a full timeout cannot pass, and no-run is not full acceptance.
