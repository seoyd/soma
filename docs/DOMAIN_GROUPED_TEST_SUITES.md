# Domain Grouped Test Suites

Sprint 85 collapses representative integration binaries into six grouped suites:

- `tests/complete_row_closure_suite.rs`
- `tests/artifact_rendering_suite.rs`
- `tests/persona_operational_suite.rs`
- `tests/workspace_safety_guard_suite.rs`
- `tests/workspace_cli_safety_suite.rs`
- `tests/workspace_determinism_suite.rs`

These suites preserve representative assertions while reducing the remaining test-binary surface. They do not replace the full workspace gate.

