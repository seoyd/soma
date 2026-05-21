# Acceptance Truth Gate V11

Focused pass is not full pass. No-run is not full pass. Cargo build is not full pass. CLI smoke is not full pass. Verification is not full pass.

`FullWorkspaceAccepted` is only valid when `cargo test --workspace --quiet` actually finishes and passes.
