# Developer Speed Runbook

Sprint 76 separates the day-to-day loop from the final ship gate.

- **Day-to-day**: `cargo check --workspace`, a changed test target, and a single CLI smoke
- **Sprint loop**: fmt/check, focused Sprint 76 tests, representative CLI smoke
- **Final ship**: fmt/check, `cargo test --workspace --quiet`, and the current Sprint 76 CLI smoke set

Optional local accelerators:

- `cargo-nextest` when installed locally
- `sccache` with a local cache directory only
