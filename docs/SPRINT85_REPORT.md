# Sprint 85 Report

Implemented:

- workspace-wide test surface audit and remaining binary inventory
- remaining family classifier and grouped domain suite plan/report
- shared fixture harness adoption report
- workspace smoke policy v2, acceptance attempt v3, full gate recovery v3, blocker drilldown, and Control Tower workspace gate panel v2
- grouped suites for complete row closure, artifact rendering, persona operational status, safety guards, CLI safety, and determinism

Moved coverage into grouped suites:

- `tests/complete_row_closure_v2.rs` -> `tests/complete_row_closure_suite.rs`
- `tests/complete_row_closure_runner.rs` -> `tests/complete_row_closure_suite.rs`
- `tests/artifact_render_cache_plan.rs` -> `tests/artifact_rendering_suite.rs`
- `tests/artifact_rendering_cost.rs` -> `tests/artifact_rendering_suite.rs`
- `tests/dashboard_secret_redaction.rs` -> `tests/artifact_rendering_suite.rs`
- `tests/dashboard_determinism.rs` -> `tests/artifact_rendering_suite.rs`
- `tests/persona_operational_status.rs` -> `tests/persona_operational_suite.rs`
- `tests/six_persona_readiness.rs` -> `tests/persona_operational_suite.rs`
- `tests/live_safety.rs` -> `tests/workspace_safety_guard_suite.rs`
- `tests/core_runtime_state.rs` -> `tests/workspace_safety_guard_suite.rs`
- `tests/control_tower_ui_readiness.rs` -> `tests/workspace_safety_guard_suite.rs`
- `tests/dashboard_serve_safety.rs` -> `tests/workspace_safety_guard_suite.rs`
- `tests/ui_framework_decision.rs` -> `tests/workspace_safety_guard_suite.rs`
- `tests/complete_row_closure_v2_cli_safety.rs` -> `tests/workspace_cli_safety_suite.rs`
- `tests/complete_row_closure_v2_determinism.rs` -> `tests/workspace_determinism_suite.rs`

Status:

- remaining workspace binaries reduced from 16 to 7 in the targeted Sprint 85 slice
- keep-separate candidate retained: `tests/persona_readiness.rs`
- shared fixture harness adoption: ready with warnings
- full workspace final gate: still blocked unless the full workspace actually finishes
- runtime, training, live, broker, order, and account paths remain deferred/absent

