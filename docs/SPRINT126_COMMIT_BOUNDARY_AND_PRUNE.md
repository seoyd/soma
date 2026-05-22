# Sprint 126 Commit Boundary and Prune Classification

## Core commit boundary

Group A is the AI member product path and should be staged/committed before any legacy prune:

- `src/league/minimal_ai_committee_core.rs`
- `src/bin/soma_experiment.rs`
- `src/league/mod.rs`
- `src/lib.rs`
- `tests/minimal_ai_committee_core.rs`
- `examples/soma_minimal_ai_committee_core.toml`
- `examples/minimal_ai_committee_core_sample.json`
- `examples/minimal_ai_committee_multi_market_sample.json`
- `examples/minimal_ai_committee_offline_member_sample.json`
- `examples/investor_archetype_style_cards.sample.json`
- `docs/SPRINT123_MEMBER_CORE_CONTRACT.md`
- `docs/SPRINT124_THREE_MEMBER_PILOT.md`
- `docs/SPRINT125_ARCHETYPE_STYLE_CARDS.md`
- `docs/SPRINT126_COMMIT_BOUNDARY_AND_PRUNE.md`

These preserve DataRouter, member brain boundary, deferred core specs, lazy activation, offline adapter, three-member pilot, style cards, Risk Governor, paper-only learning journal, and real-archetype local intake.

## Legacy prune classification

Rule used here: deleted standalone legacy tests are Group B only and must not be staged with the
AI core commit. A standalone file deletion is acceptable only when the same assertion is retained in
the surviving target `cargo test --test workspace_timeout_reduction_queue --quiet`; otherwise it
must be restored or deferred before any prune commit.

| Path | Classification | Reason |
| --- | --- | --- |
| `tests/acceptance_truth_gate_v19.rs` | SafetyCriticalKeep | Full-acceptance truth assertion is safety-adjacent. If deleted, the surviving target is `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/cargo_json_failure_reason_analysis_v1.rs` | SafeDeleteDuplicate | Cargo JSON report analysis is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/cargo_json_target_blocker_extraction_v1.rs` | SafeDeleteDuplicate | Deterministic blocker extraction is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/control_tower_timeout_reduction_queue_panel.rs` | SafetyCriticalKeep | Read-only/no-train/no-live/no-order assertions are safety sentinels. If deleted, the surviving target is `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/sprint117_baseline_truth_import.rs` | SafeDeleteDuplicate | Baseline truth import assertions are duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/sprint118_cli_safety.rs` | SafetyCriticalKeep | CLI no train/live/runtime/broker/order/account assertions are safety sentinels. If deleted, the surviving target is `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/sprint118_determinism.rs` | SafetyCriticalKeep | Determinism assertion is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`; keep deletion separate from Group A. |
| `tests/truthful_full_workspace_attempt_v19.rs` | SafetyCriticalKeep | Full workspace acceptance truth is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`; keep deletion separate from Group A. |
| `tests/truthful_no_run_attempt_v19.rs` | SafetyCriticalKeep | No-run truth/recovery behavior is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`; keep deletion separate from Group A. |
| `tests/workspace_timeout_evidence_matrix_v4.rs` | SafetyCriticalKeep | Timeout evidence/non-acceptance assertion is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`; keep deletion separate from Group A. |
| `tests/workspace_timeout_reduction_hypothesis_v1.rs` | SafeDeleteDuplicate | Timeout hypothesis report assertion is duplicated in `cargo test --test workspace_timeout_reduction_queue --quiet`. |
| `tests/workspace_timeout_reduction_queue.rs` | DeferPrune | Modified legacy timeout queue; keep separate from core feature commit. |

No additional legacy deletion was performed in Sprint 126. The modified consolidated legacy test
should be reviewed or committed separately from the AI member core.

## Real archetype intake

The intake path stays local-only and converts future owner-provided material into safe style cards:

- no network path
- no 18 live-agent activation
- no training or live inference
- no private strategy, impersonation, or guaranteed-return claims
- required do-not-learn guards
- required source confidence
- `ReviewRequired` source confidence must remain review-required
