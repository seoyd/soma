# Workspace Acceptance Truth Closure

Full workspace acceptance still requires a real finished and passing `cargo test --workspace --quiet` run. `can_claim_full_acceptance` must remain false when that did not happen.

Focused Sprint 99 tests are useful, but they stay separate from the full workspace gate. No fake pass, fake timing, or silent substitution is allowed.
