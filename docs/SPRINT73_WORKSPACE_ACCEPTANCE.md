# Sprint 73 Workspace Acceptance

Sprint 73 restores full-workspace verification as an explicit acceptance artifact.

## Required checks

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace --quiet`
- focused Sprint 73 tests
- Sprint 73 CLI smoke commands

## Why this matters

Sprint 72 used focused validation as the final signoff basis. Sprint 73 makes full-workspace test execution visible again in a dedicated acceptance report.
