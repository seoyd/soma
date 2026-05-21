# Test Tiering

Sprint 76 introduces five deterministic tiers:

- **Quick**: `cargo check --workspace` plus focused Sprint 76 tests
- **Sprint**: fmt/check, focused sprint tests, representative CLI smoke
- **Full**: fmt/check, full workspace test, full Sprint 76 CLI smoke
- **Smoke**: representative command-family smoke only
- **Audit**: safety and determinism coverage

The full workspace test remains the final ship gate. Tiering reduces iteration cost; it does not replace full acceptance.
