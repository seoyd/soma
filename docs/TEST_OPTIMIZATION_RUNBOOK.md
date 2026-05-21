# Test Optimization Runbook

Sprint 77 keeps the same development structure as Sprint 76 and adds repeated-timing guidance:

- **dev loop**: fast compile + focused Sprint 77 tests + one representative CLI smoke
- **sprint loop**: fmt/check + focused Sprint 77 coverage + representative smoke
- **full acceptance loop**: fmt/check + `cargo test --workspace --quiet` + Sprint 77 CLI smoke

Optional local-only accelerators remain optional:

- `cargo-nextest` if available locally
- `sccache` with local cache only
