# Soma Zero

Lightweight, numeric, **LLM-free** automated trading OS foundation in Rust.

## Sprint 93 additions

Sprint 93 attributes the real workspace timeout conservatively after Sprint 92 while keeping the same envelope: **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**.

- `sprint93-timeout-attribution`
- `real-timeout-attribution`
- `real-no-run-diagnostic-pass`
- `real-full-diagnostic-pass`
- `cargo-message-capture`
- `active-rustc-snapshot`
- `target-dir-growth`
- `cargo-target-progress-timeline`
- `quiet-vs-diagnostic-gate`
- `krx-non-primary-proof`
- `unknown-timeout-closure`
- `workspace-timeout-attribution-decision`
- `dashboard-renderer-entry-release-gate`
- `dashboard-renderer-reduction-hold`
- `workspace-gate-recovery-v10`
- `remaining-blocker-queue-v9`
- `safety-coverage-preservation-v9`
- `control-tower-timeout-attribution`

```bash
cargo run --quiet --bin soma_experiment -- sprint93-timeout-attribution --config examples/soma_sprint93_timeout_attribution.toml
cargo run --quiet --bin soma_experiment -- dashboard-renderer-entry-release-gate --config examples/soma_dashboard_renderer_entry_release_gate.toml
cargo run --quiet --bin soma_experiment -- control-tower-timeout-attribution --config examples/soma_control_tower_timeout_attribution.toml
```

Related docs:

- `docs/SPRINT93_TIMEOUT_ATTRIBUTION.md`
- `docs/SPRINT92_KRX_WARNING_CLOSURE.md`
- `docs/DASHBOARD_RENDERER_ENTRY_GATE.md`

These commands remain **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, runtime LLM, Mamba runtime, Gated DeltaNet runtime, model training, or browser execution.

## Sprint 92 additions

Sprint 92 closes the Sprint 91 `KrxEvidence` warning-backed state as explicitly as current local evidence allows while keeping the same envelope: **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**.

- `sprint92-krx-warning-close`
- `krx-warning-closure`
- `krx-secret-safety-isolation`
- `krx-raw-archive-redaction-coverage`
- `krx-manual-review-close`
- `krx-genuine-reduction-gate`
- `krx-queue-advancement-gate`
- `krx-real-gate-cause-drilldown`
- `krx-no-run-timeout-cause`
- `krx-full-workspace-timeout-cause`
- `dashboard-renderer-entry-gate`
- `dashboard-renderer-readiness-precheck`
- `measured-target-delta-v8`
- `real-no-run-gate-attempt-v7`
- `real-full-workspace-gate-attempt-v10`
- `workspace-gate-recovery-v9`
- `remaining-blocker-queue-v8`
- `safety-coverage-preservation-v8`
- `control-tower-krx-warning-closure`

```bash
cargo run --quiet --bin soma_experiment -- sprint92-krx-warning-close --config examples/soma_sprint92_krx_warning_close.toml
cargo run --quiet --bin soma_experiment -- krx-warning-closure --config examples/soma_krx_warning_closure.toml
cargo run --quiet --bin soma_experiment -- control-tower-krx-warning-closure --config examples/soma_control_tower_krx_warning_closure.toml
```

Related docs:

- `docs/SPRINT92_KRX_WARNING_CLOSURE.md`
- `docs/KRX_SECRET_SAFETY_ISOLATION.md`
- `docs/KRX_GENUINE_REDUCTION_GATE.md`
- `docs/DASHBOARD_RENDERER_ENTRY_GATE.md`
- `docs/SPRINT92_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, runtime LLM, Mamba runtime, Gated DeltaNet runtime, model training, or browser execution.

## Sprint 91 additions

Sprint 91 performs a conservative `KrxEvidence` reduction pass after Sprint 90 while keeping the same envelope: **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**.

- `sprint91-krx-evidence-recover`
- `krx-evidence-real-reduction-plan`
- `krx-evidence-assertion-migration`
- `krx-evidence-fixture-setup-reduction`
- `krx-evidence-auth-boundary-preservation`
- `krx-evidence-endpoint-template-preservation`
- `krx-evidence-source-boundary-preservation`
- `krx-evidence-market-data-only-preservation`
- `krx-evidence-compile-impact`
- `krx-evidence-no-run-rerun`
- `krx-evidence-full-gate-rerun`
- `seven-blocker-queue-progress-v7`
- `measured-target-delta-v7`
- `real-no-run-gate-attempt-v6`
- `real-full-workspace-gate-attempt-v9`
- `workspace-gate-recovery-v8`
- `remaining-blocker-queue-v7`
- `safety-coverage-preservation-v7`
- `control-tower-krx-evidence-recovery`

```bash
cargo run --quiet --bin soma_experiment -- sprint91-krx-evidence-recover --config examples/soma_sprint91_krx_evidence_recover.toml
cargo run --quiet --bin soma_experiment -- krx-evidence-real-reduction-plan --config examples/soma_krx_evidence_real_reduction_plan.toml
cargo run --quiet --bin soma_experiment -- control-tower-krx-evidence-recovery --config examples/soma_control_tower_krx_evidence_recovery.toml
```

Related docs:

- `docs/SPRINT91_KRX_EVIDENCE_RECOVERY.md`
- `docs/KRX_EVIDENCE_REAL_REDUCTION.md`
- `docs/KRX_MARKET_DATA_ONLY_BOUNDARY.md`
- `docs/KRX_EVIDENCE_PRESERVATION_GATES.md`
- `docs/SPRINT91_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, runtime LLM, Mamba runtime, Gated DeltaNet runtime, model training, or browser execution.

## Sprint 90 additions

Sprint 90 performs a conservative `ExternalPrediction` reduction pass after Sprint 89 while keeping the same envelope: **local-only, deterministic, research-only, paper-only, and read-only**.

- `sprint90-external-prediction-recover`
- `external-prediction-real-reduction-plan`
- `external-prediction-assertion-migration`
- `external-prediction-fixture-setup-reduction`
- `external-prediction-feature-variant-reduction`
- `external-prediction-compile-impact`
- `external-prediction-no-run-rerun`
- `external-prediction-full-gate-rerun`
- `external-prediction-schema-preservation`
- `external-prediction-model-card-preservation`
- `external-prediction-evaluation-preservation`
- `seven-blocker-queue-progress-v6`
- `measured-target-delta-v6`
- `real-no-run-gate-attempt-v5`
- `real-full-workspace-gate-attempt-v8`
- `workspace-gate-recovery-v7`
- `remaining-blocker-queue-v6`
- `safety-coverage-preservation-v6`
- `control-tower-external-prediction-recovery`

```bash
cargo run --quiet --bin soma_experiment -- sprint90-external-prediction-recover --config examples/soma_sprint90_external_prediction_recover.toml
cargo run --quiet --bin soma_experiment -- external-prediction-real-reduction-plan --config examples/soma_external_prediction_real_reduction_plan.toml
cargo run --quiet --bin soma_experiment -- control-tower-external-prediction-recovery --config examples/soma_control_tower_external_prediction_recovery.toml
```

Related docs:

- `docs/SPRINT90_EXTERNAL_PREDICTION_RECOVERY.md`
- `docs/EXTERNAL_PREDICTION_REAL_REDUCTION.md`
- `docs/EXTERNAL_PREDICTION_FEATURE_VARIANTS.md`
- `docs/EXTERNAL_PREDICTION_PRESERVATION_GATES.md`
- `docs/SPRINT90_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, runtime LLM, Mamba runtime, Gated DeltaNet runtime, model training, or browser execution.

## Sprint 71 additions

Sprint 71 adds a static **operator briefing mode** over the existing model-ops trace stack:

- `operator-briefing`
- `owner-action-checklist`
- `operator-decision-queue`
- `briefing-delta`
- `leaderboard-warning-closure`
- `retirement-evidence-completion`
- `control-tower-briefing`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, or training.

## Sprint 72 additions

Sprint 72 extends the Sprint 71 briefing stack with conservative **offline evidence attachment** and **direct-watch readiness hardening**:

- `offline-evidence-attach`
- `prediction-history-expand`
- `retirement-regression-pack`
- `evidence-gap-close-v2`
- `owner-checklist-close`
- `direct-watch-score`
- `briefing-readiness-gate`

```bash
cargo run --quiet --bin soma_experiment -- offline-evidence-attach --config examples/soma_offline_evidence_attach.toml
cargo run --quiet --bin soma_experiment -- direct-watch-score --config examples/soma_direct_watch_score.toml
cargo run --quiet --bin soma_experiment -- briefing-readiness-gate --config examples/soma_briefing_readiness_gate.toml
```

Related docs:

- `docs/OFFLINE_EVIDENCE_ATTACHMENT.md`
- `docs/PREDICTION_HISTORY_EXPANSION.md`
- `docs/RETIREMENT_REGRESSION_EVIDENCE_PACK.md`
- `docs/DIRECT_WATCH_READINESS.md`
- `docs/SPRINT72_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, or training.

## Sprint 73 additions

Sprint 73 closes the remaining `ext-model-b:1.0.0` fixture prediction-history gap and restores full-workspace acceptance as a visible final report:

- `ext-model-b-prediction-close`
- `prediction-coverage-finalize`
- `evidence-gap-final-close`
- `direct-watch-final-gate`
- `control-tower-final-refresh`
- `sprint73-workspace-acceptance`

```bash
cargo run --quiet --bin soma_experiment -- ext-model-b-prediction-close --config examples/soma_ext_model_b_prediction_close.toml
cargo run --quiet --bin soma_experiment -- direct-watch-final-gate --config examples/soma_direct_watch_final_gate.toml
cargo run --quiet --bin soma_experiment -- sprint73-workspace-acceptance --config examples/soma_sprint73_workspace_acceptance.toml
```

Related docs:

- `docs/EXT_MODEL_B_PREDICTION_GAP_CLOSURE.md`
- `docs/PREDICTION_COVERAGE_FINALIZATION.md`
- `docs/DIRECT_WATCH_FINAL_GATE.md`
- `docs/SPRINT73_WORKSPACE_ACCEPTANCE.md`
- `docs/SPRINT73_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, or training.

## Sprint 74 additions

Sprint 74 extends Sprint 73 into **real evidence follow-up** while keeping the same conservative operating envelope: local-only, research-only, paper-only, market-data-only, and static/read-only.

- `real-evidence-followup`
- `real-evidence-attach`
- `kis-real-evidence-validate`
- `real-provenance-audit`
- `real-preflight-audit`
- `real-outcome-readiness`
- `real-sequence-readiness`
- `real-modelops-impact`
- `control-tower-warning-reduce`
- `direct-watch-warning-rationale`
- `real-evidence-runbook`

```bash
cargo run --quiet --bin soma_experiment -- real-evidence-followup --config examples/soma_real_evidence_followup.toml
cargo run --quiet --bin soma_experiment -- kis-real-evidence-validate --config examples/soma_kis_real_evidence_validate.toml
cargo run --quiet --bin soma_experiment -- real-evidence-runbook --config examples/soma_real_evidence_runbook.toml
```

Related docs:

- `docs/REAL_EVIDENCE_FOLLOWUP.md`
- `docs/KIS_REAL_EVIDENCE_VALIDATION.md`
- `docs/CONTROL_TOWER_WARNING_REDUCTION.md`
- `docs/REAL_EVIDENCE_OPERATOR_RUNBOOK.md`
- `docs/SPRINT74_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, market-data-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, or training.

## Sprint 75 additions

Sprint 75 uses the Sprint 74 real-evidence attachment to refresh offline prediction coverage, re-run conservative external reevaluation, and reduce the remaining `ModelPredictionsStale` warning without changing the research-only envelope.

- `real-prediction-requirements`
- `real-prediction-refresh-plan`
- `real-prediction-import`
- `real-external-reevaluate`
- `real-leaderboard-refresh`
- `real-modelops-refresh`
- `model-predictions-stale-close`
- `control-tower-warning-close-v2`
- `direct-watch-post-evidence-gate`
- `real-modelops-runbook`

```bash
cargo run --quiet --bin soma_experiment -- real-prediction-requirements --config examples/soma_real_prediction_requirements.toml
cargo run --quiet --bin soma_experiment -- real-modelops-refresh --config examples/soma_real_modelops_refresh.toml
cargo run --quiet --bin soma_experiment -- direct-watch-post-evidence-gate --config examples/soma_direct_watch_post_evidence_gate.toml
```

Related docs:

- `docs/REAL_EVIDENCE_PREDICTION_REFRESH.md`
- `docs/REAL_EXTERNAL_REEVALUATION.md`
- `docs/MODEL_PREDICTIONS_STALE_CLOSURE.md`
- `docs/DIRECT_WATCH_POST_EVIDENCE_GATE.md`
- `docs/SPRINT75_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, or training.

## Sprint 76 additions

Sprint 76 modernizes the pinned Rust toolchain, audits the Cargo workspace, adds deterministic test tiers, inventories slow/heavy paths, and publishes a developer speed runbook without weakening the final full-workspace gate.

- `rust-toolchain-modernize`
- `toolchain-version-report`
- `cargo-workspace-audit`
- `test-tier-plan`
- `test-runtime-budget`
- `slow-test-inventory`
- `cli-smoke-tiering`
- `developer-speed-runbook`
- `workspace-acceptance-v2`

```bash
cargo run --quiet --bin soma_experiment -- rust-toolchain-modernize --config examples/soma_rust_toolchain_modernize.toml
cargo run --quiet --bin soma_experiment -- toolchain-version-report --config examples/soma_toolchain_version_report.toml
cargo run --quiet --bin soma_experiment -- workspace-acceptance-v2 --config examples/soma_workspace_acceptance_v2.toml
```

Related docs:

- `docs/RUST_TOOLCHAIN_MODERNIZATION.md`
- `docs/TEST_TIERING.md`
- `docs/BUILD_TEST_PERFORMANCE.md`
- `docs/DEVELOPER_SPEED_RUNBOOK.md`
- `docs/SPRINT76_REPORT.md`

These commands remain **local-only, deterministic, stable-only, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime LLM, Mamba runtime, or model training.

## Sprint 77 additions

Sprint 77 keeps the stable 1.95.0 toolchain from Sprint 76 and adds repeated timing, fixture/setup cost analysis, CLI smoke cost reduction planning, and acceptance v3 without weakening the full-workspace ship gate.

- `repeated-workspace-timing`
- `test-binary-cost`
- `fixture-setup-cost`
- `artifact-render-cost`
- `cli-smoke-cost-reduce`
- `fixture-dedup-plan`
- `fixture-cache-plan`
- `artifact-render-cache-plan`
- `test-support-refactor-plan`
- `dev-loop-savings-estimate`
- `workspace-acceptance-v3`

```bash
cargo run --quiet --bin soma_experiment -- repeated-workspace-timing --config examples/soma_repeated_workspace_timing.toml
cargo run --quiet --bin soma_experiment -- cli-smoke-cost-reduce --config examples/soma_cli_smoke_cost_reduce.toml
cargo run --quiet --bin soma_experiment -- workspace-acceptance-v3 --config examples/soma_workspace_acceptance_v3.toml
```

Related docs:

- `docs/REPEATED_WORKSPACE_TIMING.md`
- `docs/FIXTURE_SETUP_DEDUP.md`
- `docs/CLI_SMOKE_COST_REDUCTION.md`
- `docs/TEST_OPTIMIZATION_RUNBOOK.md`
- `docs/SPRINT77_REPORT.md`

These commands remain **local-only, deterministic, stable-only, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime LLM, Mamba runtime, or model training.

## Sprint 78 additions

Sprint 78 brings core/Mamba3/committee/storage contracts back to the center without implementing runtime inference, model training, live execution, or persona expansion.

- `core-completion-v2`
- `mamba3fin-core-contract`
- `mamba3-runtime-readiness`
- `committee-completion-gate`
- `committee-materialization-plan-v2`
- `training-data-storage-decision`
- `training-data-registry-spec`
- `training-data-layout-plan`
- `training-data-lineage-spec`
- `mamba3-implementation-roadmap`

```bash
cargo run --quiet --bin soma_experiment -- core-completion-v2 --config examples/soma_core_completion_v2.toml
cargo run --quiet --bin soma_experiment -- committee-completion-gate --config examples/soma_committee_completion_gate.toml
cargo run --quiet --bin soma_experiment -- training-data-storage-decision --config examples/soma_training_data_storage_decision.toml
```

Related docs:

- `docs/CORE_COMPLETION_V2.md`
- `docs/MAMBA3FIN_CORE_CONTRACT.md`
- `docs/COMMITTEE_COMPLETION_GATE.md`
- `docs/TRAINING_DATA_STORAGE_ARCHITECTURE.md`
- `docs/STORAGE_FORMAT_DECISION.md`
- `docs/SPRINT78_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime LLM, Mamba runtime, model training, or six/twelve persona activation.

## Sprint 79 additions

Sprint 79 adds **Gated DeltaNet as a sequence-core candidate contract**, introduces a shared candidate registry next to Mamba3Fin, and materializes local training-storage placeholder artifacts without adding runtime inference, training, or live execution.

- `sequence-core-registry`
- `gated-deltanet-core-contract`
- `gated-deltanet-readiness`
- `sequence-core-comparison-plan`
- `sequence-core-external-contract`
- `training-storage-materialize`
- `training-storage-integrity`
- `model-family-storage-contract`
- `control-tower-sequence-core`

```bash
cargo run --quiet --bin soma_experiment -- sequence-core-registry --config examples/soma_sequence_core_registry.toml
cargo run --quiet --bin soma_experiment -- training-storage-materialize --config examples/soma_training_storage_materialize.toml
cargo run --quiet --bin soma_experiment -- control-tower-sequence-core --config examples/soma_control_tower_sequence_core.toml
```

Related docs:

- `docs/SEQUENCE_CORE_CANDIDATE_REGISTRY.md`
- `docs/GATED_DELTANET_CORE_CONTRACT.md`
- `docs/TRAINING_DATA_STORAGE_MATERIALIZATION.md`
- `docs/SEQUENCE_CORE_EXTERNAL_PROTOTYPE_CONTRACT.md`
- `docs/SPRINT79_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime LLM, Mamba runtime, Gated DeltaNet runtime, model training, or persona expansion.

## Current status

The repository now has a single active layer:

1. **Active Soma Zero v0 kernel** — the root `soma-zero` crate

## Soma Zero v0 runtime path

The active MVP path is:

1. `MarketSnapshot`
2. deterministic feature derivation
3. `MockSignalEngine`
4. 3 numeric investor delegates
   - `momentum_trend_fast`
   - `value_quality_filter`
   - `cycle_risk_skeptic`
5. Chair v0
6. Risk Governor v0
7. paper-only broker
8. deterministic audit events

## Non-negotiable rules

- no runtime LLM
- every live decision must be numeric
- default action is `NoTrade`
- Risk Governor has absolute veto
- paper execution only
- no real broker execution path
- no live self-mutation

## Sprint 52 additions

Sprint 52 adds:

- **KIS-first provider simplification** for Korean/US equity market-data-only workflows
- **Soma Control Tower v0** read-only local monitoring UI
- deterministic `provider-simplify`, `dashboard-snapshot`, and `dashboard-render` commands

```bash
cargo run --quiet --bin soma_experiment -- provider-simplify --config examples/soma_provider_simplify_kis_primary.toml
cargo run --quiet --bin soma_experiment -- dashboard-snapshot --config examples/soma_dashboard_source_kis_control_tower.toml
cargo run --quiet --bin soma_experiment -- dashboard-render --config examples/soma_dashboard_render_static.toml
```

These remain local-only, paper-only, read-only, and never enable broker/order/account/live execution controls.

## Sprint 53 additions

Sprint 53 adds:

- **Owner Input Layer** for structured audited owner opinions
- **Human Confirmation Protocol v1** for paper-only confirmation rules
- **Control Tower owner panel** plus owner-aware candidate / human-confirm / audit views
- deterministic `owner-input-validate`, `owner-review-queue`, `owner-apply-input`, `owner-impact-report`, and `owner-thesis-book` commands

```bash
cargo run --quiet --bin soma_experiment -- owner-input-validate --config examples/soma_owner_input_note.toml
cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml
cargo run --quiet --bin soma_experiment -- owner-impact-report --config examples/soma_owner_impact_report.toml
cargo run --quiet --bin soma_experiment -- dashboard-snapshot --config examples/soma_dashboard_source_with_owner_panel.toml
```

The owner **can express an opinion**, but the default safe path is structured audited `OwnerInput`. Freeform conversation is not required, and `PaperConfirm` remains paper-only.

## Sprint 55 additions

Sprint 55 adds formal research-only audit/gating commands:

- `core-completion-audit`
- `sequence-readiness`
- `mamba-readiness-v2`
- `model-escalation-decision`
- optional `mamba-prototype-plan`

```bash
cargo run --quiet --bin soma_experiment -- core-completion-audit --config examples/soma_core_completion_audit.toml
cargo run --quiet --bin soma_experiment -- sequence-readiness --config examples/soma_sequence_readiness.toml
cargo run --quiet --bin soma_experiment -- mamba-readiness-v2 --config examples/soma_mamba_readiness_v2_blocked.toml
cargo run --quiet --bin soma_experiment -- model-escalation-decision --config examples/soma_model_escalation_decision.toml
```

These commands audit readiness only. They do **not** add live trading, broker/order/account paths, runtime LLM, or Rust-native Mamba runtime/training.

## Sprint 56 additions

Sprint 56 operationalizes the existing Trinity committee into a deterministic paper-only loop:

- candidate generation from local evidence
- candidate lifecycle state machine
- committee cycle runner with Chair v0, Risk Governor, and audited owner review
- simulated paper lifecycle reporting
- Control Tower v1 operational loop / Trinity / lifecycle monitor panels

```bash
cargo run --quiet --bin soma_experiment -- candidate-generate --config examples/soma_candidate_generate_kis.toml
cargo run --quiet --bin soma_experiment -- committee-cycle --config examples/soma_committee_cycle_single_candidate.toml
cargo run --quiet --bin soma_experiment -- trinity-operational-loop --config examples/soma_trinity_operational_loop_kis.toml
cargo run --quiet --bin soma_experiment -- paper-lifecycle-report --config examples/soma_paper_lifecycle_report.toml
cargo run --quiet --bin soma_experiment -- operational-audit-timeline --config examples/soma_operational_audit_timeline.toml
```

This remains **local-only, deterministic, research-only, paper-only, and monitor-only**. It does **not** add live trading, broker execution, account state, runtime LLM, Mamba runtime, or 6/12/18 active personas.

## Sprint 57 additions

Sprint 57 deepens KIS official evidence and refreshes the existing Control Tower v1 from local artifacts:

- KIS evidence depth before/after aggregation
- ordered operational runbook
- refreshed Control Tower HTML/JSON/TXT output
- Trinity loop and paper lifecycle overlay from local reports

```bash
cargo run --quiet --bin soma_experiment -- kis-evidence-depth-run --config examples/soma_kis_evidence_depth_run.toml
cargo run --quiet --bin soma_experiment -- control-tower-refresh --config examples/soma_control_tower_refresh_after_kis_depth.toml
cargo run --quiet --bin soma_experiment -- operational-runbook --config examples/soma_operational_runbook_kis_loop.toml
```

This still does **not** add live trading, real orders, broker/account APIs, runtime LLM, or persona expansion.

## Sprint 58 additions

Sprint 58 closes KIS auth, adds a bounded market-data smoke, and refreshes Control Tower plus the local runbook from the new artifacts:

- KIS auth closure with redacted endpoint previews
- market-data-only dry-run and collection plan v2
- bounded KIS smoke bundle with env isolation and secret audit
- Control Tower auto-refresh with smoke status attachment
- operational runbook v2 with exact local CLI sequence

```bash
cargo run --quiet --bin soma_experiment -- kis-auth-close --config examples/soma_kis_auth_close.toml
cargo run --quiet --bin soma_experiment -- kis-market-data-dry-run --config examples/soma_kis_market_data_dry_run.toml
cargo run --quiet --bin soma_experiment -- kis-collection-plan-v2 --config examples/soma_kis_collection_plan_v2_fixture.toml
cargo run --quiet --bin soma_experiment -- kis-market-data-smoke --config examples/soma_kis_market_data_smoke_fixture.toml
cargo run --quiet --bin soma_experiment -- control-tower-auto-refresh --config examples/soma_control_tower_auto_refresh.toml
cargo run --quiet --bin soma_experiment -- operational-runbook-v2 --config examples/soma_operational_runbook_v2.toml
```

This remains **market-data-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account APIs, runtime LLM, or Mamba runtime.

## Sprint 59 additions

Sprint 59 adds a conservative **system integration review / benchmark diff / manual ship gate** layer over the existing Core + UI + Chair + Trinity stack:

- `system-review` for a deterministic readiness bundle
- `system-benchmark-diff` for local artifact drift checks
- `manual-ship-checklist` for required paper-ops acceptance items
- `system-ship-gate` for a conservative paper-ops-monitoring gate

```bash
cargo run --quiet --bin soma_experiment -- system-review --config examples/soma_system_review_full.toml
cargo run --quiet --bin soma_experiment -- system-benchmark-diff --config examples/soma_system_benchmark_diff.toml
cargo run --quiet --bin soma_experiment -- manual-ship-checklist --config examples/soma_manual_ship_checklist.toml
cargo run --quiet --bin soma_experiment -- system-ship-gate --config examples/soma_system_ship_gate.toml
```

These commands remain **research-only, paper-only, local-only, and deterministic**. They do **not** add live trading, broker execution, order/account endpoints, runtime LLM, Mamba runtime, or persona expansion.

## Sprint 60 additions

Sprint 60 hardens evidence and review ergonomics **without** switching UI frameworks or enabling Mamba runtime work:

- `evidence-hardening` for a conservative evidence / ergonomics / UI / Mamba timing bundle
- `outcome-link-coverage` for local outcome-link depth and no-lookahead checks
- `counterfactual-coverage` for NoTrade / RiskDenied depth and totals
- `review-ergonomics` for owner queue clarity and paper-only workflow visibility
- `ui-framework-decision` for static-now / Tauri+Svelte-later guidance
- `mamba-application-timing` for deferred runtime timing gates

```bash
cargo run --quiet --bin soma_experiment -- evidence-hardening --config examples/soma_evidence_hardening.toml
cargo run --quiet --bin soma_experiment -- outcome-link-coverage --config examples/soma_outcome_link_coverage.toml
cargo run --quiet --bin soma_experiment -- counterfactual-coverage --config examples/soma_counterfactual_coverage.toml
cargo run --quiet --bin soma_experiment -- review-ergonomics --config examples/soma_review_ergonomics.toml
cargo run --quiet --bin soma_experiment -- ui-framework-decision --config examples/soma_ui_framework_decision.toml
cargo run --quiet --bin soma_experiment -- mamba-application-timing --config examples/soma_mamba_application_timing.toml
```

This sprint stays **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, or a Tauri/Svelte implementation.

## Sprint 61 additions

Sprint 61 closes bounded KIS evidence gaps and hardens sequence dataset preparation **without** adding Mamba runtime or a heavier UI stack:

- `kis-evidence-expansion-plan-v2` for bounded official evidence expansion planning
- `kis-evidence-closure` for the Sprint 61 evidence / owner / sequence bundle
- `outcome-link-depth-close-v2` for TP / SL / TE depth and no-lookahead checks
- `owner-review-discipline-v2` for stricter paper-only owner review discipline
- `sequence-readiness-hardening`, `sequence-window-preview`, and `no-lookahead-sequence-proof` for bounded sequence export readiness

```bash
cargo run --quiet --bin soma_experiment -- kis-evidence-expansion-plan-v2 --config examples/soma_kis_evidence_expansion_plan_v2.toml
cargo run --quiet --bin soma_experiment -- kis-evidence-closure --config examples/soma_kis_evidence_closure.toml
cargo run --quiet --bin soma_experiment -- outcome-link-depth-close-v2 --config examples/soma_outcome_link_depth_close_v2.toml
cargo run --quiet --bin soma_experiment -- owner-review-discipline-v2 --config examples/soma_owner_review_discipline_v2.toml
cargo run --quiet --bin soma_experiment -- sequence-readiness-hardening --config examples/soma_sequence_readiness_hardening.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, or training commands.

## Sprint 62 additions

Sprint 62 exports the first bounded sequence dataset artifact and freezes schema/label contracts **without** adding Mamba runtime or training:

- `sequence-dataset-export` for bounded local `dataset.csv` + manifest generation
- `sequence-dataset-quality` for deterministic export quality checks
- `sequence-dataset-drift` for manifest drift detection
- `sequence-dataset-replay-check` for deterministic replay verification
- `external-bridge-readiness` for prediction CSV import/evaluation readiness
- `mamba3fin-prototype-gate` for planning-only external prototype gating

```bash
cargo run --quiet --bin soma_experiment -- sequence-dataset-export --config examples/soma_sequence_dataset_export_small.toml
cargo run --quiet --bin soma_experiment -- sequence-dataset-quality --config examples/soma_sequence_dataset_quality.toml
cargo run --quiet --bin soma_experiment -- sequence-dataset-drift --config examples/soma_sequence_dataset_drift.toml
cargo run --quiet --bin soma_experiment -- sequence-dataset-replay-check --config examples/soma_sequence_dataset_replay_check.toml
cargo run --quiet --bin soma_experiment -- external-bridge-readiness --config examples/soma_external_bridge_readiness.toml
cargo run --quiet --bin soma_experiment -- mamba3fin-prototype-gate --config examples/soma_mamba3fin_prototype_gate.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, Rust-native inference/training, or a Tauri/Svelte dependency.

## Sprint 63 additions

Sprint 63 adds external prediction CSV import and deterministic offline evaluation on top of the bounded sequence export:

- `external-prediction-import-v2` for local prediction CSV + model card validation
- `external-model-evaluate` for deterministic offline metrics
- `external-vs-trinity` for diagnostic comparison against baseline / Trinity / NoTrade / RiskDenied
- `external-prediction-ablation` for deterministic stress checks
- `external-model-promotion-gate` for research-only gating
- `mamba3fin-contract` for the planning-only external prototype contract

```bash
cargo run --quiet --bin soma_experiment -- external-prediction-import-v2 --config examples/soma_external_prediction_import_v2_valid.toml
cargo run --quiet --bin soma_experiment -- external-model-evaluate --config examples/soma_external_model_evaluate.toml
cargo run --quiet --bin soma_experiment -- external-vs-trinity --config examples/soma_external_vs_trinity.toml
cargo run --quiet --bin soma_experiment -- external-prediction-ablation --config examples/soma_external_prediction_ablation.toml
cargo run --quiet --bin soma_experiment -- external-model-promotion-gate --config examples/soma_external_model_promotion_gate.toml
cargo run --quiet --bin soma_experiment -- mamba3fin-contract --config examples/soma_mamba3fin_contract.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Sprint 64 additions

Sprint 64 adds a conservative external artifact registry and offline leaderboard layer on top of Sprint 63:

- `external-artifact-registry` for local artifact registration and contract checks
- `external-evaluation-history` for version-by-version offline metric history
- `calibration-drift` for offline calibration regression tracking
- `external-model-version-comparison` for latest-vs-previous comparison
- `conservative-external-leaderboard` for conservative offline ranking
- `external-registry-audit` for local-only artifact safety scanning

```bash
cargo run --quiet --bin soma_experiment -- external-artifact-registry --config examples/soma_external_artifact_registry.toml
cargo run --quiet --bin soma_experiment -- external-evaluation-history --config examples/soma_external_evaluation_history.toml
cargo run --quiet --bin soma_experiment -- calibration-drift --config examples/soma_calibration_drift.toml
cargo run --quiet --bin soma_experiment -- external-model-version-comparison --config examples/soma_external_model_version_comparison.toml
cargo run --quiet --bin soma_experiment -- conservative-external-leaderboard --config examples/soma_conservative_external_leaderboard.toml
cargo run --quiet --bin soma_experiment -- external-registry-audit --config examples/soma_external_registry_audit.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Sprint 65 additions

Sprint 65 turns the Sprint 64 artifact stack into a conservative external model research-ops workflow:

- `external-model-research-ops` for the full offline lifecycle / review / watchlist / risk bundle
- `external-model-review-queue` for deterministic model review items
- `external-model-watchlist` for owner-safe tracking state
- `model-comparability-matrix` for bounded compatibility checks
- `artifact-completeness` for conservative artifact scoring
- `model-risk-profile` for offline evidence-risk summaries
- `model-leaderboard-changelog` for deterministic rank/change tracking

```bash
cargo run --quiet --bin soma_experiment -- external-model-research-ops --config examples/soma_external_model_research_ops.toml
cargo run --quiet --bin soma_experiment -- external-model-review-queue --config examples/soma_external_model_review_queue.toml
cargo run --quiet --bin soma_experiment -- external-model-watchlist --config examples/soma_external_model_watchlist.toml
cargo run --quiet --bin soma_experiment -- model-comparability-matrix --config examples/soma_model_comparability_matrix.toml
cargo run --quiet --bin soma_experiment -- artifact-completeness --config examples/soma_artifact_completeness.toml
cargo run --quiet --bin soma_experiment -- model-risk-profile --config examples/soma_model_risk_profile.toml
cargo run --quiet --bin soma_experiment -- model-leaderboard-changelog --config examples/soma_model_leaderboard_changelog.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Sprint 66 additions

Sprint 66 turns the Sprint 65 model ops bundle into a conservative offline review-closure layer:

- `model-review-close` for deterministic pending-review closure
- `prediction-history-pack` for bounded multi-version prediction history
- `model-ops-decision-log` for explicit owner/policy/risk/coverage decisions
- `model-ops-operator-qa` for read-only safety and readiness review
- `model-ops-regression-guard` for offline baseline/current regression checks
- `control-tower-model-ops-refresh` for refreshed static model ops visibility

```bash
cargo run --quiet --bin soma_experiment -- model-review-close --config examples/soma_model_review_close.toml
cargo run --quiet --bin soma_experiment -- prediction-history-pack --config examples/soma_prediction_history_pack.toml
cargo run --quiet --bin soma_experiment -- model-ops-decision-log --config examples/soma_model_ops_decision_log.toml
cargo run --quiet --bin soma_experiment -- model-ops-operator-qa --config examples/soma_model_ops_operator_qa.toml
cargo run --quiet --bin soma_experiment -- model-ops-regression-guard --config examples/soma_model_ops_regression_guard.toml
cargo run --quiet --bin soma_experiment -- control-tower-model-ops-refresh --config examples/soma_control_tower_model_ops_refresh.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Sprint 67 additions

Sprint 67 turns the Sprint 66 model ops outputs into a conservative per-version rollup layer:

- `model-ops-rollup` for one offline summary card per model version
- `model-regression-explain` for human-readable regression causes
- `operator-qa-rollup` for deduplicated QA summaries
- `decision-log-rollup` for per-version decision aggregation
- `model-risk-rollup` for conservative risk interpretation
- `model-action-priority` for copy-only next-action ordering
- `control-tower-model-ops-rollup` for refreshed static Control Tower summary cards

```bash
cargo run --quiet --bin soma_experiment -- model-ops-rollup --config examples/soma_model_ops_rollup.toml
cargo run --quiet --bin soma_experiment -- model-regression-explain --config examples/soma_model_regression_explain.toml
cargo run --quiet --bin soma_experiment -- operator-qa-rollup --config examples/soma_operator_qa_rollup.toml
cargo run --quiet --bin soma_experiment -- decision-log-rollup --config examples/soma_decision_log_rollup.toml
cargo run --quiet --bin soma_experiment -- model-risk-rollup --config examples/soma_model_risk_rollup.toml
cargo run --quiet --bin soma_experiment -- model-action-priority --config examples/soma_model_action_priority.toml
cargo run --quiet --bin soma_experiment -- control-tower-model-ops-rollup --config examples/soma_control_tower_model_ops_rollup.toml

## Sprint 68 additions

Sprint 68 adds a static trace drill-down layer over the Sprint 67 model ops rollup:

- `model-ops-trace` for the full local trace bundle
- `model-trace-index` for artifact lineage only
- `model-decision-conflicts` for conservative decision disagreement summaries
- `model-regression-trace` for baseline/current regression evidence
- `model-qa-trace` for operator QA evidence linkage
- `model-action-trace` for copy-only action rationale
- `model-version-diff-trace` for deterministic per-version diff summaries

```bash
cargo run --quiet --bin soma_experiment -- model-ops-trace --config examples/soma_model_ops_trace.toml
cargo run --quiet --bin soma_experiment -- model-trace-index --config examples/soma_model_trace_index.toml
cargo run --quiet --bin soma_experiment -- model-decision-conflicts --config examples/soma_model_decision_conflicts.toml
cargo run --quiet --bin soma_experiment -- model-regression-trace --config examples/soma_model_regression_trace.toml
cargo run --quiet --bin soma_experiment -- model-qa-trace --config examples/soma_model_qa_trace.toml
cargo run --quiet --bin soma_experiment -- model-action-trace --config examples/soma_model_action_trace.toml
cargo run --quiet --bin soma_experiment -- model-version-diff-trace --config examples/soma_model_version_diff_trace.toml
```
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Sprint 69 additions

Sprint 69 hardens the Sprint 68 trace layer with baseline snapshot coverage, comparison target closure, and trace completeness auditing:

- `baseline-snapshot-coverage` for baseline/current coverage readiness
- `comparison-target-registry` for explicit comparison-target mapping
- `missing-comparison-targets` for conservative target-gap reporting
- `trace-completeness-audit` for static trace dimension coverage
- `downgrade-evidence-audit` for conservative downgrade evidence checks
- `snapshot-diff-integrity` for deterministic snapshot integrity review
- `control-tower-trace-coverage` for the static Control Tower coverage panel

```bash
cargo run --quiet --bin soma_experiment -- baseline-snapshot-coverage --config examples/soma_baseline_snapshot_coverage.toml
cargo run --quiet --bin soma_experiment -- comparison-target-registry --config examples/soma_comparison_target_registry.toml
cargo run --quiet --bin soma_experiment -- missing-comparison-targets --config examples/soma_missing_comparison_targets.toml
cargo run --quiet --bin soma_experiment -- trace-completeness-audit --config examples/soma_trace_completeness_audit.toml
cargo run --quiet --bin soma_experiment -- downgrade-evidence-audit --config examples/soma_downgrade_evidence_audit.toml
cargo run --quiet --bin soma_experiment -- snapshot-diff-integrity --config examples/soma_snapshot_diff_integrity.toml
cargo run --quiet --bin soma_experiment -- control-tower-trace-coverage --config examples/soma_control_tower_trace_coverage.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Sprint 70 additions

Sprint 70 extends the existing offline trace stack with unexpected diff triage, contract alignment explanation, owner review closure, and warning reduction:

- `unexpected-diff-triage` for the combined Sprint 70 triage report
- `snapshot-diff-classify` for deterministic unexpected-diff classification
- `contract-alignment-audit-v2` for dataset/schema/label alignment explanation
- `owner-review-close-v2` for conservative owner review closure
- `trace-warning-reduce` for explicit warning reduction accounting
- `downgrade-evidence-closure-plan` for conservative downgrade closure planning
- `diff-root-cause` for offline root-cause summaries
- `model-version-review-disposition` for final research-only per-version disposition
- `control-tower-diff-triage` for the static Control Tower diff triage panel

```bash
cargo run --quiet --bin soma_experiment -- unexpected-diff-triage --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- snapshot-diff-classify --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- contract-alignment-audit-v2 --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- owner-review-close-v2 --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- trace-warning-reduce --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- downgrade-evidence-closure-plan --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- diff-root-cause --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- model-version-review-disposition --config examples/soma_unexpected_diff_triage.toml
cargo run --quiet --bin soma_experiment -- control-tower-diff-triage --config examples/soma_unexpected_diff_triage.toml
```

This sprint remains **research-only, paper-only, local-first, deterministic, and read-only**. It does **not** add live trading, broker/order/account paths, runtime LLM, Mamba runtime, model training, live inference, or a Tauri/Svelte dependency.

## Quick start

### Run the Soma Zero safety tests

```bash
cargo test
```

### Run the workspace checks

```bash
cargo check --workspace
cargo test --workspace --quiet
```

### Run the ablation lab

```bash
cargo run --bin soma_experiment -- ablation --config examples/soma_ablation_feature_lab.toml
```

### Run the Sprint 14 evidence-gap router

```bash
cargo run --bin soma_experiment -- sprint14 --from-ablation target/soma_ablations/ablation_feature_lab/ablation_report.json --out target/soma_sprint14
```

### Run the Sprint 15 evidence-closure campaign

```bash
cargo run --bin soma_experiment -- evidence-close --config examples/soma_evidence_closure.toml
```

### Run the Sprint 16 real-evidence recheck

```bash
cargo run --bin soma_experiment -- real-evidence --config examples/soma_real_evidence_closure.toml
```

### Run the Sprint 17 local data preflight

```bash
cargo run --bin soma_experiment -- data-preflight --input data/local/BTCUSDT_1m.csv --out target/soma_data_onboarding --symbol BTC-USDT --timeframe 1m
```

### Import a KRX daily market snapshot into stored OHLCV

```bash
cargo run --bin soma_experiment -- import-krx-snapshot --input data/local/data_3609_20260510.csv --out target/krx_snapshot_store --symbol 060310
```

### Run the Sprint 17 onboarding pack

```bash
cargo run --bin soma_experiment -- onboard-data --config examples/soma_data_onboarding.toml
```

### Run the Sprint 19 bounded market-data collector

```bash
cargo run --bin soma_experiment -- collect-candles --provider alphavantage --symbol AAPL --venue NASDAQ --timeframe 1d --out data/collected --outputsize compact --max-rows 100 --api-key-env-var ALPHAVANTAGE_API_KEY
```

### Run the Sprint 20 bounded official collection plan

```bash
cargo run --bin soma_experiment -- collect-plan --config examples/soma_official_collection_compact.toml
```

### Run the Sprint 20 official evidence pack

```bash
cargo run --bin soma_experiment -- evidence-run --from-collection target/soma_official_collection/sprint20_official_compact/official_collection_report.json --out target/soma_official_collection/sprint20_official_compact/official_evidence_run
```

### Run the Sprint 21 official AI benchmark

```bash
cargo run --bin soma_experiment -- ai-benchmark --config examples/soma_ai_benchmark_upbit_only.toml
```

### Run the Sprint 22 Mamba readiness audit

```bash
cargo run --bin soma_experiment -- mamba-readiness --config examples/soma_mamba_readiness_crypto_only.toml
```

### Run the Sprint 23 core hardening check

```bash
cargo run --bin soma_experiment -- core-check --config examples/soma_core_check.toml
```

### Run the Sprint 24 core-checked benchmark

```bash
cargo run --bin soma_experiment -- core-benchmark --config examples/soma_core_checked_benchmark_baseline_only.toml
```

### Run the Sprint 25 provider auth preflight

```bash
cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml
```

### Run the Sprint 25 venue coverage report

```bash
cargo run --bin soma_experiment -- official-coverage --config examples/soma_venue_coverage_targets.toml
```

### Run the Sprint 25 official evidence expansion

```bash
cargo run --bin soma_experiment -- evidence-expand --config examples/soma_official_evidence_expansion_crypto_only.toml
```

### Run the Sprint 26 official evidence acquisition

```bash
cargo run --bin soma_experiment -- official-acquire --config examples/soma_official_evidence_acquisition.toml
```

### Run the Sprint 27 yfinance research adapter

```bash
python research/yfinance_fetch.py --config research/configs/yfinance_research_compact.toml
cargo run --bin soma_experiment -- yfinance-import --config examples/soma_yfinance_import.toml
cargo run --bin soma_experiment -- yahoo-research --config examples/soma_yfinance_research_benchmark.toml
```

To compare official-ready evidence counts against yfinance research-only counts:

```bash
cargo run --bin soma_experiment -- official-vs-yfinance --yfinance-report target/soma_yahoo_research/aapl_msft_yfinance_research/yahoo_research_evidence_report.json
```

### Run the Sprint 28 source-aware benchmark

```bash
cargo run --bin soma_experiment -- source-benchmark --config examples/soma_source_benchmark_yfinance_only.toml
```

### Run the Sprint 29 provider catalog and readiness flow

```bash
cargo run --bin soma_experiment -- provider-catalog
cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml
cargo run --bin soma_experiment -- provider-select --market korean-equity
```

### Run the Sprint 30 provider freshness and compatibility flow

```bash
cargo run --bin soma_experiment -- provider-reality --config examples/soma_provider_reality.toml
cargo run --bin soma_experiment -- strategy-data-check --provider alphavantage --use-case eod-swing
cargo run --bin soma_experiment -- provider-recommend --market us-equity --use-case realtime-scalping --budget free-only
```

### Run the Sprint 31 provider-reality evidence executor

```bash
cargo run --bin soma_experiment -- evidence-plan --config examples/soma_evidence_plan_crypto_only.toml
cargo run --bin soma_experiment -- evidence-execute --config examples/soma_evidence_plan_crypto_only.toml
cargo run --bin soma_experiment -- readiness-matrix --config examples/soma_readiness_matrix.toml
```

### Run the Sprint 32 minimal committee MVP

```bash
cargo run --bin soma_experiment -- committee-smoke --config examples/soma_committee_smoke_fixture.toml
cargo run --bin soma_experiment -- committee-smoke --config examples/soma_committee_smoke_crypto_only.toml
cargo run --bin soma_experiment -- committee-smoke --config examples/soma_committee_smoke_yfinance_research.toml
cargo run --bin soma_experiment -- persona-cards
```

### Run the Sprint 33 committee diagnostics flow

```bash
cargo run --bin soma_experiment -- committee-load-scenarios --config examples/soma_committee_load_fixture.toml
cargo run --bin soma_experiment -- committee-replay --config examples/soma_committee_replay_fixture.toml
cargo run --bin soma_experiment -- committee-diagnostics --config examples/soma_committee_diagnostics_fixture.toml
```

### Run the Sprint 34 Committee V1 bundle

```bash
cargo run --bin soma_experiment -- committee-v1 --config examples/soma_committee_v1_fixture.toml
cargo run --bin soma_experiment -- committee-v1 --config examples/soma_committee_v1_yfinance.toml
```

This flow stays **research-only**:

- `core-check` gates execution
- local official evidence only
- optional Python training stays outside Rust
- yfinance remains unofficial and readiness-ineligible
- no broker, account, or live trading commands are added
- provider readiness means acquisition setup only, not model readiness

### Run the Sprint 35 materialization and benchmark flows

```bash
cargo run --bin soma_experiment -- committee-materialize --config examples/soma_committee_materialize_fixture.toml
cargo run --bin soma_experiment -- committee-materialize --config examples/soma_committee_materialize_evidence_lane.toml
cargo run --bin soma_experiment -- committee-benchmark --config examples/soma_committee_benchmark_fixture.toml
cargo run --bin soma_experiment -- committee-benchmark --config examples/soma_committee_benchmark_crypto_only.toml
```

These Sprint 35 flows also stay **research-only**:

- materialization is local-artifact-only
- benchmark execution remains paper-only and core-check-gated
- yfinance remains research-only
- fixture remains architecture-test-only
- crypto-only evidence is not treated as cross-market readiness
- no broker, account, live trading, runtime-LLM, or Mamba command is added

### Run the Sprint 36 official committee flows

```bash
cargo run --bin soma_experiment -- committee-pack-official --config examples/soma_committee_pack_evidence_lane.toml
cargo run --bin soma_experiment -- committee-pack-official --config examples/soma_committee_pack_controlled_official.toml
cargo run --bin soma_experiment -- committee-link-outcomes --config examples/soma_committee_link_outcomes_fixture.toml
cargo run --bin soma_experiment -- committee-official-benchmark --config examples/soma_committee_official_benchmark_controlled.toml
cargo run --bin soma_experiment -- committee-official-benchmark --config examples/soma_committee_official_benchmark_crypto_only.toml
```

These Sprint 36 flows stay **research-only**:

- official row-level evidence still does not imply live-trading readiness
- no-lookahead violations block official readiness
- yfinance remains research-only
- fixture remains architecture-test-only
- crypto-only official evidence remains crypto-only
- no broker, account, live trading, runtime-LLM, or Mamba path is added

### Run the Sprint 37 committee outcome coverage flows

```bash
cargo run --bin soma_experiment -- committee-outcome-coverage --config examples/soma_committee_outcome_coverage_controlled.toml
cargo run --bin soma_experiment -- committee-counterfactual-audit --config examples/soma_committee_counterfactual_audit_fixture.toml
cargo run --bin soma_experiment -- committee-performance-matrix --config examples/soma_committee_performance_matrix_controlled.toml
```

These Sprint 37 flows also stay **research-only**:

- outcome coverage and counterfactual depth are benchmark diagnostics only
- deterministic local candle inputs only; remote paths are rejected
- yfinance remains research-only and fixture evidence remains architecture-test-only
- crypto-only evidence remains crypto-only and cannot imply cross-market readiness
- no broker, account, runtime-LLM, Mamba, or live-trading command is added

### Run the Sprint 38 committee reference-pack flows

```bash
cargo run --bin soma_experiment -- committee-build-references --config examples/soma_committee_build_references_controlled.toml
cargo run --bin soma_experiment -- committee-align-candles --config examples/soma_committee_align_candles_fixture.toml
cargo run --bin soma_experiment -- committee-sufficiency-close --config examples/soma_committee_sufficiency_close_controlled.toml
```

These Sprint 38 flows stay **research-only**:

- generated references must come from local candles and remain no-lookahead safe
- yfinance stays research-only and fixture evidence stays controlled-only
- controlled fixture closure does not equal official readiness
- crypto-only passes remain crypto-only and do not imply cross-market readiness
- no broker, account, runtime-LLM, Mamba, or live-trading command is added

### Run the Sprint 39 official replication flows

```bash
cargo run --bin soma_experiment -- official-artifact-inventory --config examples/soma_official_artifact_inventory.toml
cargo run --bin soma_experiment -- official-row-inject --config examples/soma_official_row_inject.toml
cargo run --bin soma_experiment -- official-replication --config examples/soma_official_replication_aapl_controlled_official.toml
```

Sprint 39 stays **local-only, research-only, and paper-only**:

- official replication never accepts remote config paths
- operator actions only name env vars and never print secret values
- controlled, fixture, yfinance, and crypto-only evidence remain separated from non-crypto official evidence
- bundles are written under `target/soma_official_replication/<replication_id>/`
- no live broker, account, runtime-LLM, or real-money path is introduced

### Run the Sprint 40 core performance scorecard

```bash
cargo run --bin soma_experiment -- core-performance --config examples/soma_core_performance_controlled.toml
cargo run --bin soma_experiment -- core-bottleneck --config examples/soma_core_bottleneck.toml
cargo run --bin soma_experiment -- core-regression --config examples/soma_core_regression.toml
```

Sprint 40 stays **research-only and paper-only**:

- controlled evidence remains diagnostic-only
- yfinance remains research-only
- fixture evidence remains architecture-test-only
- crypto-only evidence remains crypto-only
- no live broker, account, runtime-LLM, Mamba runtime, or real-money path is introduced

Related docs:
- `docs/CORE_PERFORMANCE_SCORECARD.md`
- `docs/COMMITTEE_VALUE_ATTRIBUTION.md`
- `docs/RISK_GOVERNOR_VALUE.md`
- `docs/CORE_BOTTLENECK_REPORT.md`
- `docs/SPRINT40_REPORT.md`

### Run the Sprint 41 comparable evidence and counterfactual depth flows

```bash
cargo run --bin soma_experiment -- comparable-evidence --config examples/soma_comparable_evidence_official_replication.toml
cargo run --bin soma_experiment -- counterfactual-depth-plan --config examples/soma_counterfactual_depth_plan.toml
cargo run --bin soma_experiment -- counterfactual-depth-close --config examples/soma_counterfactual_depth_close_official_replication.toml
```

Sprint 41 stays **local-only, research-only, and paper-only**:

- comparable rows keep official / controlled / crypto / yfinance / fixture boundaries
- controlled evidence remains diagnostic-only
- crypto-only evidence remains crypto-only
- yfinance remains research-only
- fixture evidence remains architecture-test-only
- scorecard reruns remain research-only and never imply live trading
- no broker, order, account, runtime-LLM, or Mamba runtime command is added

Related docs:
- `docs/COMPARABLE_COMMITTEE_EVIDENCE.md`
- `docs/COUNTERFACTUAL_DEPTH_PLAN.md`
- `docs/COUNTERFACTUAL_DEPTH_CLOSURE.md`
- `docs/SCENARIO_MATERIALIZATION_WEAK_CLOSURE.md`
- `docs/SPRINT41_REPORT.md`

### Run the Sprint 42 candle coverage flows

```bash
cargo run --bin soma_experiment -- candle-pack --config examples/soma_candle_pack_official_controlled.toml
cargo run --bin soma_experiment -- candle-coverage-match --config examples/soma_candle_coverage_close_official_replication.toml
cargo run --bin soma_experiment -- comparable-backfill --config examples/soma_comparable_backfill_official_replication.toml
cargo run --bin soma_experiment -- candle-coverage-close --config examples/soma_candle_coverage_close_official_replication.toml
```

Sprint 42 stays **local-only, research-only, and paper-only**:

- official candle readiness requires provenance plus ready preflight
- timeframe and timestamp mismatches are explicit and conservative
- yfinance stays research-only
- fixture evidence stays architecture-test-only
- controlled evidence stays diagnostic-only
- crypto official coverage stays crypto-only
- no broker, order, account, runtime-LLM, or Mamba runtime command is added

Related docs:
- `docs/OFFICIAL_CANDLE_COVERAGE_PACK.md`
- `docs/TIMEFRAME_TIMESTAMP_ALIGNMENT.md`
- `docs/COMPARABLE_EVIDENCE_BACKFILL.md`
- `docs/CANDLE_COVERAGE_CLOSURE.md`
- `docs/SPRINT42_REPORT.md`

### Run the Sprint 43 official candle expansion flows

```bash
cargo run --bin soma_experiment -- candle-gap-map --config examples/soma_candle_gap_map_official_replication.toml
cargo run --bin soma_experiment -- candle-expansion-plan --config examples/soma_candle_expansion_plan_missing_auth.toml
cargo run --bin soma_experiment -- candle-expand --config examples/soma_candle_expand_official_replication.toml
```

Sprint 43 stays **local-only, research-only, and paper-only**:

- official candle gaps stay explicit and deterministic
- local canonical CSV reuse is preferred before provider collection
- missing auth, approval, endpoint template, provenance, preflight, and CSV prerequisites emit operator actions
- controlled, yfinance, fixture, and crypto-only evidence do not self-promote into non-crypto official readiness
- no broker, order, account, runtime-LLM, or Mamba runtime command is added

Related docs:
- `docs/OFFICIAL_CANDLE_GAP_MAP.md`
- `docs/OFFICIAL_CANDLE_EXPANSION_PLAN.md`
- `docs/CANDLE_EXPANSION_CLOSURE.md`
- `docs/SPRINT43_REPORT.md`

### Run the Sprint 44 official candle join-audit flows

```bash
cargo run --bin soma_experiment -- candle-join-audit --config examples/soma_candle_join_audit_official_replication.toml
cargo run --bin soma_experiment -- candle-join-repair-plan --config examples/soma_candle_join_audit_symbol_mismatch.toml
cargo run --bin soma_experiment -- official-ready-match-close --config examples/soma_official_ready_match_close_official_replication.toml
cargo run --bin soma_experiment -- candle-lineage --config examples/soma_candle_join_audit_official_replication.toml
```

Sprint 44 stays **local-only, research-only, and paper-only**:

- join repair uses explicit local symbol/timeframe/timestamp maps only
- source class cannot be promoted through aliasing or closure repair
- no-lookahead unsafe matches remain rejected
- controlled evidence stays diagnostic-only, yfinance stays research-only, fixture evidence stays architecture-test-only, and crypto-only evidence stays crypto-only
- closure output under `target/soma_official_ready_match_closure/<closure_id>/` is an audit bundle, not a live-readiness claim
- no broker, order, account, runtime-LLM, or Mamba runtime command is added

Related docs:
- `docs/OFFICIAL_CANDLE_JOIN_AUDIT.md`
- `docs/MATCH_KEY_NORMALIZATION.md`
- `docs/CANDLE_JOIN_REPAIR_ACTIONS.md`
- `docs/OFFICIAL_READY_MATCH_CLOSURE.md`
- `docs/SPRINT44_REPORT.md`

### Run the Sprint 45 complete-row closure flows

```bash
cargo run --bin soma_experiment -- official-ready-row-inventory --config examples/soma_official_ready_row_inventory_official_replication.toml
cargo run --bin soma_experiment -- scenario-materialize-v3 --config examples/soma_scenario_materialize_v3_official_replication.toml
cargo run --bin soma_experiment -- complete-row-close --config examples/soma_complete_row_close_official_replication.toml
```

Sprint 45 stays **local-only, research-only, and paper-only**:

- official-ready match counts stay separate from complete comparable row counts
- complete rows still require scenario, outcome, baseline, NoTrade, RiskDenied, and no-lookahead-safe evidence
- controlled evidence stays diagnostic-only, crypto-only stays crypto-only, yfinance stays research-only, and fixture evidence stays architecture-test-only
- backfill can close evidence gaps but cannot promote source class or imply profitability
- no broker, order, account, runtime-LLM, or Mamba runtime command is added

Related docs:
- `docs/OFFICIAL_READY_ROW_INVENTORY.md`
- `docs/SCENARIO_MATERIALIZATION_V3.md`
- `docs/COMPLETE_COMPARABLE_ROW_CLOSURE.md`
- `docs/OUTCOME_LINKAGE_V2.md`
- `docs/SPRINT45_REPORT.md`

### Run the Sprint 46 future-window and closure flows

```bash
cargo run --bin soma_experiment -- future-window-requirements --config examples/soma_future_window_requirements_official_replication.toml
cargo run --bin soma_experiment -- future-window-extension-plan --config examples/soma_future_window_extension_plan_official_replication.toml
cargo run --bin soma_experiment -- outcome-linkage-v3 --config examples/soma_outcome_linkage_v3_official_replication.toml
cargo run --bin soma_experiment -- counterfactual-complete-v2 --config examples/soma_counterfactual_complete_v2_official_replication.toml
cargo run --bin soma_experiment -- complete-row-close-v2 --config examples/soma_complete_row_close_v2_official_replication.toml
```

Sprint 46 stays **local-only, research-only, and paper-only**:

- future-window sufficiency only closes evidence plumbing gaps; it does not prove profitability
- outcome linkage remains no-lookahead-safe and does not prove model usefulness
- counterfactual completion depends on outcome linkage and still does not imply live readiness
- complete-row closure v2 measures evidence completeness, not trading readiness
- controlled evidence stays diagnostic-only, crypto-only stays crypto-only, yfinance stays research-only, and fixture evidence stays architecture-test-only
- no broker, order, account, runtime-LLM, Mamba runtime, or six/12/18-persona activation command is added

Related docs:
- `docs/FUTURE_WINDOW_REQUIREMENTS.md`
- `docs/OFFICIAL_FUTURE_WINDOW_EXTENSION.md`
- `docs/OUTCOME_LINKAGE_V3.md`
- `docs/COUNTERFACTUAL_COMPLETION_V2.md`
- `docs/SPRINT46_REPORT.md`

### Run the Sprint 48 diversity sweep flows

```bash
cargo run --bin soma_experiment -- barrier-profiles --config examples/soma_barrier_profiles_primary.toml
cargo run --bin soma_experiment -- official-diversity-gap-map --config examples/soma_official_diversity_gap_map_multi_row.toml
cargo run --bin soma_experiment -- official-diversity-row-select --config examples/soma_official_diversity_row_select_multi_row.toml
cargo run --bin soma_experiment -- outcome-diversity-audit --config examples/soma_outcome_diversity_audit_multi_row.toml
cargo run --bin soma_experiment -- balanced-outcome-coverage --config examples/soma_balanced_outcome_coverage_multi_row.toml
cargo run --bin soma_experiment -- diversity-sufficiency-v2 --config examples/soma_diversity_sufficiency_v2_multi_row.toml
cargo run --bin soma_experiment -- official-evidence-diversity-sweep --config examples/soma_official_evidence_diversity_sweep_multi_row.toml
```

Sprint 48 stays **local-only, research-only, and paper-only**:

- two all-take-profit official rows remain plumbing-only and are not enough for committee research readiness
- mixed outcome labels can improve diversity status, but never imply profitability or live readiness
- diagnostic and exploratory barrier profiles cannot satisfy official sufficiency
- crypto-only, yfinance, controlled, and fixture evidence remain segregated from official non-crypto sufficiency
- no broker, order, account, runtime-LLM, or Mamba runtime path is added

Related docs:
- `docs/BARRIER_PROFILE_REGISTRY.md`
- `docs/OFFICIAL_EVIDENCE_DIVERSITY_GAP_MAP.md`
- `docs/OUTCOME_DIVERSITY_AUDIT.md`
- `docs/DIVERSITY_AWARE_SUFFICIENCY_V2.md`
- `docs/SPRINT48_REPORT.md`

### Run the Sprint 49 KRX official activation flows

```bash
cargo run --bin soma_experiment -- krx-auth-readiness --config examples/soma_krx_auth_readiness.toml
cargo run --bin soma_experiment -- krx-symbol-whitelist --config examples/soma_krx_symbol_whitelist_compact.toml
cargo run --bin soma_experiment -- krx-evidence-plan --config examples/soma_krx_official_activate_missing_auth.toml
cargo run --bin soma_experiment -- krx-official-activate --config examples/soma_krx_official_activate_local_import.toml
cargo run --bin soma_experiment -- krx-official-activate --config examples/soma_krx_official_activate_diversity_rerun.toml
```

Sprint 49 stays **local-first, research-only, market-data-only, and paper-only**:

- env-var auth is secret-safe and only used for bounded KRX collection when explicitly enabled
- local canonical CSV import remains valid even when env auth is absent
- provenance and preflight gate official readiness
- downstream reruns remain conservative when outcome-linked evidence is sparse
- no broker, order, account, runtime-LLM, Mamba runtime, or live-trading path is added

Related docs:
- `docs/KRX_OFFICIAL_EVIDENCE_ACTIVATION.md`
- `docs/KRX_SYMBOL_WHITELIST.md`
- `docs/KRX_COLLECTION_AND_PREFLIGHT.md`
- `docs/KRX_DOWNSTREAM_RERUN.md`
- `docs/SPRINT49_REPORT.md`

### Run the Sprint 50 KRX bounded collection closure flows

```bash
cargo run --bin soma_experiment -- krx-collection-dry-run --config examples/soma_krx_collection_dry_run.toml
cargo run --bin soma_experiment -- krx-collection-plan --config examples/soma_krx_collection_plan_missing_auth.toml
cargo run --bin soma_experiment -- krx-collection-close --config examples/soma_krx_collection_close_fixture_replay.toml
cargo run --bin soma_experiment -- krx-collection-close --config examples/soma_krx_collection_close_local_import.toml
cargo run --bin soma_experiment -- krx-candle-sufficiency --config examples/soma_krx_candle_sufficiency.toml
cargo run --bin soma_experiment -- krx-outcome-link-close --config examples/soma_krx_outcome_link_close.toml
```

Sprint 50 stays **local-first, research-only, market-data-only, and bounded**:

- dry runs use env-var presence only and keep endpoint previews redacted
- fixture replay is architecture-only and does not count as official readiness
- local import can validate canonical/provenance/preflight inputs before sufficiency and outcome closure
- downstream summaries remain conservative when official candles or outcome links are still missing
- no broker, order, account, runtime-LLM, Mamba runtime, or live-trading path is added

Related docs:
- `docs/KRX_BOUNDED_COLLECTION_SMOKE.md`
- `docs/KRX_RAW_SCHEMA_AND_CANONICALIZATION.md`
- `docs/KRX_CANDLE_SUFFICIENCY.md`
- `docs/KRX_OUTCOME_LINK_CLOSURE.md`
- `docs/SPRINT50_REPORT.md`

### Run the paper-only demo CLI

```bash
cargo run -- --help
```

## Repository map

### Active Soma Zero kernel

- `src/core`
- `src/signal`
- `src/league`
- `src/chair`
- `src/risk`
- `src/paper`
- `src/backtest`
- `tests/mvp.rs`

### Deferred legacy workspace

No legacy model-system crates remain in the active workspace. The old legacy archive has also been removed from the repository, so only the active Soma Zero path remains in-tree.

## Cleanup and audit docs

- `docs/ARCHITECTURE.md`
- `docs/REPO_AUDIT.md`
- `docs/CLEANUP_PLAN.md`
- `docs/DEFERRED_MODULES.md`
- `docs/ABLATION_LAB.md`
- `docs/SPRINT14_DECISION.md`
- `docs/SPRINT14_REPORT.md`
- `docs/EVIDENCE_GAP_PLAN.md`
- `docs/EVIDENCE_CLOSURE_CAMPAIGN.md`
- `docs/MINIMUM_EVIDENCE_PLAN_UPDATE.md`
- `docs/OFFICIAL_ARTIFACT_INVENTORY.md`
- `docs/OFFICIAL_ROW_INJECTION.md`
- `docs/OFFICIAL_EVIDENCE_REPLICATION.md`
- `docs/OFFICIAL_REPLICATION_OPERATOR_ACTIONS.md`
- `docs/SPRINT39_REPORT.md`
- `docs/OFFICIAL_CANDLE_GAP_MAP.md`
- `docs/OFFICIAL_CANDLE_EXPANSION_PLAN.md`
- `docs/CANDLE_EXPANSION_CLOSURE.md`
- `docs/SPRINT43_REPORT.md`
- `docs/SPRINT15_REPORT.md`
- `docs/REAL_LOCAL_DATA_EVIDENCE.md`
- `docs/DATA_PROVENANCE.md`
- `docs/SYNTHETIC_VS_REAL_EVIDENCE.md`
- `docs/SPRINT16_REPORT.md`
- `docs/REAL_DATA_ONBOARDING.md`
- `docs/CSV_FORMAT_PROFILES.md`
- `docs/PREFLIGHT_VALIDATION.md`
- `docs/SPRINT17_REPORT.md`
- `docs/MARKET_DATA_COLLECTOR.md`
- `docs/CANDLE_PROVIDER_UPBIT.md`
- `docs/STOCK_DATA_PROVIDER_PLAN.md`
- `docs/SPRINT18_REPORT.md`
- `docs/EQUITY_MARKET_DATA_PROVIDERS.md`
- `docs/BOUNDED_COLLECTION_POLICY.md`
- `docs/CANDLE_PROVIDER_KRX.md`
- `docs/CANDLE_PROVIDER_ALPHAVANTAGE.md`
- `docs/CANDLE_PROVIDER_ALPACA.md`
- `docs/SPRINT19_REPORT.md`
- `docs/CORE_CHECKED_BENCHMARK.md`
- `docs/EXTERNAL_TABULAR_SIGNAL_BENCHMARK.md`
- `docs/OFFICIAL_DATA_SIGNAL_EVIDENCE.md`
- `docs/PROVIDER_AUTH_PREFLIGHT.md`
- `docs/VENUE_COVERAGE_TARGETS.md`
- `docs/OFFICIAL_EVIDENCE_EXPANSION.md`
- `docs/SPRINT24_REPORT.md`
- `docs/SPRINT25_REPORT.md`
- `docs/OFFICIAL_EVIDENCE_ACQUISITION.md`
- `docs/OPERATOR_ACTION_PLAN.md`
- `docs/PREVIOUS_COLLECTION_COMPARISON.md`
- `docs/SPRINT26_REPORT.md`
- `docs/YFINANCE_RESEARCH_ADAPTER.md`
- `docs/SPRINT27_REPORT.md`
- `docs/SOURCE_AWARE_BENCHMARK.md`
- `docs/SOURCE_MISMATCH_REPORT.md`
- `docs/YFINANCE_BENCHMARK_LIMITS.md`
- `docs/SPRINT28_REPORT.md`
- `docs/OFFICIAL_DATA_SOURCE_RECOMMENDATIONS.md`
- `docs/PROVIDER_CREDENTIALS.md`
- `docs/PROVIDER_SELECTION_POLICY.md`
- `docs/SPRINT29_REPORT.md`
- `docs/PROVIDER_FRESHNESS_TIERS.md`
- `docs/PROVIDER_COST_AND_ENTITLEMENT.md`
- `docs/STRATEGY_DATA_COMPATIBILITY.md`
- `docs/SPRINT30_REPORT.md`
- `docs/MINIMAL_INVESTOR_COMMITTEE.md`
- `docs/PERSONA_CARD_LITE.md`
- `docs/CHAIR_V0.md`
- `docs/COMMITTEE_SMOKE_TEST.md`
- `docs/SPRINT32_REPORT.md`
- `docs/COMMITTEE_SCENARIO_LOADING.md`
- `docs/COMMITTEE_REPLAY.md`
- `docs/COMMITTEE_OUTCOME_COVERAGE.md`
- `docs/COMMITTEE_COUNTERFACTUAL_AUDIT.md`
- `docs/COMMITTEE_PERFORMANCE_EVIDENCE_MATRIX.md`
- `docs/COMMITTEE_EVIDENCE_SUFFICIENCY_GATE.md`
- `docs/CHAIR_RISK_DIAGNOSTICS.md`
- `docs/SIX_PERSONA_DESIGN_READINESS.md`
- `docs/SPRINT33_REPORT.md`
- `docs/COMMITTEE_V1_OPERATIONAL_MVP.md`
- `docs/COMMITTEE_DECISION_QUALITY.md`
- `docs/CHAIR_RISK_CALIBRATION.md`
- `docs/COMMITTEE_V1_READINESS.md`
- `docs/SPRINT34_REPORT.md`
- `docs/SPRINT37_REPORT.md`
- `docs/EXECUTABLE_EVIDENCE_PLAN.md`
- `docs/EVIDENCE_READINESS_MATRIX.md`
- `docs/PROVIDER_REALITY_EXECUTOR.md`
- `docs/SPRINT31_REPORT.md`
- `data/README.md`
- `data/collected/README.md`
- `data/local/PUT_REAL_CSV_HERE.md`
- `cleanup_manifest.toml`

## Quarantine policy

This repository used a conservative cleanup flow:

1. prove unused
2. quarantine first
3. run tests
4. delete only after another safe cycle

That archive phase is now complete and the quarantined material has been deleted from the repository.

## Legacy note

Older README content described the repository as a hybrid LLM/model systems project. That legacy stack no longer exists in the active repository; the active product direction is now **Soma Zero**: a minimal numeric trading kernel with paper-only execution and survival-first risk controls.

## Sprint 51 KIS market-data-only flow

KIS is the primary operational market-data provider for Korean equity and credential-ready US equity research flows, while KRX remains the Korean reference/fallback provider. The KIS flow stays research-only, paper-only, bounded, local-first, and market-data-only.

Smoke commands:
- `cargo run --quiet --bin soma_experiment -- kis-auth-readiness --config examples/soma_kis_auth_readiness.toml`
- `cargo run --quiet --bin soma_experiment -- kis-endpoint-policy --config examples/soma_kis_endpoint_policy.toml`
- `cargo run --quiet --bin soma_experiment -- kis-symbol-whitelist --config examples/soma_kis_symbol_whitelist_compact.toml`
- `cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml`

## Sprint 54

Soma Control Tower v1 adds a richer local-only dashboard bundle with KIS market-data readiness monitoring, next-action planning, owner action drafts, and static HTML/JSON/TXT outputs. It remains deterministic, paper-only, secret-redacted, and read-only by default.
## Sprint 80: Sequence Core Prototype Comparison

Sprint 80 adds an offline prototype-comparison layer for `Mamba3Fin` and `GatedDeltaNet` on top of the Sprint 79 sequence-core registry and materialized storage contract.

- compare external prediction CSV + model card artifacts only
- expand Trinity committee evidence packs without activating 6/12 personas
- populate training-data registry manifests with local artifact references only
- keep runtime/training/live inference/live trading deferred or forbidden

## Sprint 81: Prototype Interpretation Hardening

Sprint 81 adds a conservative interpretation layer above the Sprint 80 prototype bundle.

- weight official/research/diagnostic/fixture evidence explicitly
- audit committee reference depth, traces, and representativeness
- preserve NoTrade and RiskDenied as first-class defensive axes
- check training artifact lineage completeness and reference depth
- keep winner/decision gates diagnostic-only with runtime/training/live inference still deferred

## Sprint 82: Official Evidence Depth Expansion

Sprint 82 extends Sprint 81 with deeper official/reference coverage while keeping the same conservative envelope.

- `official-evidence-depth-expand`
- `committee-reference-close`
- `official-scenario-pack-v3`
- `official-outcome-pack-v3`
- `official-baseline-pack-v3`
- `official-notrade-pack-v3`
- `official-riskdenied-pack-v3`
- `defensive-counterfactual-depth`
- `official-reference-quality`
- `official-reference-diversity`
- `official-reference-no-lookahead`
- `official-reference-source-boundary`
- `sequence-core-confidence-rerun`
- `sequence-core-decision-gate-v2`
- `control-tower-evidence-depth`

```bash
cargo run --quiet --bin soma_experiment -- official-evidence-depth-expand --config examples/soma_official_evidence_depth_expand.toml
cargo run --quiet --bin soma_experiment -- committee-reference-close --config examples/soma_committee_reference_close.toml
cargo run --quiet --bin soma_experiment -- sequence-core-decision-gate-v2 --config examples/soma_sequence_core_decision_gate_v2.toml
```

Related docs:

- `docs/OFFICIAL_EVIDENCE_DEPTH_EXPANSION.md`
- `docs/COMMITTEE_REFERENCE_CLOSURE.md`
- `docs/DEFENSIVE_COUNTERFACTUAL_DEPTH.md`
- `docs/SEQUENCE_CORE_CONFIDENCE_RERUN.md`
- `docs/SPRINT82_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, Gated DeltaNet runtime, or model training.

## Sprint 84: Test Cost Reduction and Grouped Acceptance Suites

Sprint 84 follows Sprint 83 by reducing the targeted Sprint 82/83 integration-test binary surface, adding a shared fixture harness, and rebuilding the honest final gate on top of grouped suites.

- `sprint84-test-cost-reduce`
- `test-binary-consolidate`
- `shared-fixture-harness-report`
- `representative-smoke-harness`
- `exhaustive-smoke-manifest`
- `safety-smoke-manifest`
- `cli-smoke-execution-policy`
- `test-runtime-before-after`
- `workspace-final-gate-v2`
- `control-tower-test-cost`

```bash
cargo run --quiet --bin soma_experiment -- sprint84-test-cost-reduce --config examples/soma_sprint84_test_cost_reduce.toml
cargo run --quiet --bin soma_experiment -- test-binary-consolidate --config examples/soma_test_binary_consolidate.toml
cargo run --quiet --bin soma_experiment -- workspace-final-gate-v2 --config examples/soma_workspace_final_gate_v2.toml
```

## Sprint 85: Workspace-Wide Gate Recovery and Remaining Binary Collapse

Sprint 85 extends Sprint 84 by auditing the remaining workspace-wide integration-test bottlenecks, collapsing representative families into grouped domain suites, and rebuilding the full acceptance recovery bundle with blocker drilldown.

- `sprint85-workspace-gate-recover`
- `workspace-test-surface-audit`
- `remaining-test-binary-inventory`
- `domain-suite-plan`
- `shared-fixture-adoption`
- `workspace-smoke-policy-v2`
- `workspace-acceptance-attempt-v3`
- `full-gate-recovery-v3`
- `workspace-blocker-drilldown`
- `control-tower-workspace-gate-v2`

```bash
cargo run --quiet --bin soma_experiment -- sprint85-workspace-gate-recover --config examples/soma_sprint85_workspace_gate_recovery.toml
cargo run --quiet --bin soma_experiment -- workspace-test-surface-audit --config examples/soma_workspace_test_surface_audit.toml
cargo run --quiet --bin soma_experiment -- control-tower-workspace-gate-v2 --config examples/soma_control_tower_workspace_gate_v2.toml
```

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, model training, browser execution controls, Mamba runtime, or Gated DeltaNet runtime.

Related docs:

- `docs/SPRINT84_TEST_COST_REDUCTION.md`
- `docs/SHARED_FIXTURE_HARNESS.md`
- `docs/CLI_SMOKE_EXECUTION_POLICY.md`
- `docs/WORKSPACE_FINAL_GATE_V2.md`
- `docs/SPRINT84_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, Gated DeltaNet runtime, or model training.

## Sprint 86: Residual Workspace Binary Collapse and Final Gate Recovery

Sprint 86 follows Sprint 85 by collapsing the remaining residual workspace integration families into conservative grouped suites, adding compile-only and `cargo test --no-run` interpretation surfaces, and rebuilding the final gate recovery bundle without overstating full-workspace success.

- `sprint86-residual-gate-recover`
- `residual-binary-audit`
- `residual-family-classifier`
- `residual-consolidation-plan`
- `legacy-integration-migration`
- `compile-only-workspace-attempt`
- `cargo-test-no-run-gate`
- `full-workspace-attempt-v4`
- `full-gate-recovery-v4`
- `residual-blocker-drilldown-v2`
- `workspace-binary-delta-v2`
- `safety-coverage-preservation-v2`
- `control-tower-workspace-gate-v3`

```bash
cargo run --quiet --bin soma_experiment -- sprint86-residual-gate-recover --config examples/soma_sprint86_residual_gate_recover.toml
cargo run --quiet --bin soma_experiment -- residual-binary-audit --config examples/soma_residual_binary_audit.toml
cargo run --quiet --bin soma_experiment -- control-tower-workspace-gate-v3 --config examples/soma_control_tower_workspace_gate_v3.toml
```

Related docs:

- `docs/SPRINT86_RESIDUAL_GATE_RECOVERY.md`
- `docs/RESIDUAL_INTEGRATION_SUITE_MIGRATION.md`
- `docs/COMPILE_ONLY_WORKSPACE_ATTEMPT.md`
- `docs/SAFETY_COVERAGE_PRESERVATION_V2.md`
- `docs/SPRINT86_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, model training, browser execution, Mamba runtime, or Gated DeltaNet runtime.

## Sprint 87: Workspace Compile Graph Surgery and Broad Family Consolidation

Sprint 87 follows Sprint 86 by auditing compile-graph fanout, consolidating the next compile-heavy integration families into broad grouped suites, and keeping compile-only, no-run, and full-workspace truth separate.

- `sprint87-compile-gate-recover`
- `workspace-compile-graph-audit`
- `test-target-fanout`
- `dev-dependency-fanout`
- `feature-unification-audit`
- `compile-family-classifier-v2`
- `compile-heavy-consolidation-plan`
- `compile-only-attempt-v2`
- `no-run-acceptance-gate-v2`
- `full-workspace-attempt-v5`
- `compile-gate-recovery`
- `compile-blocker-drilldown-v3`
- `test-target-delta-v3`
- `safety-coverage-preservation-v3`
- `control-tower-compile-gate-v4`

```bash
cargo run --quiet --bin soma_experiment -- sprint87-compile-gate-recover --config examples/soma_sprint87_compile_gate_recover.toml
cargo run --quiet --bin soma_experiment -- workspace-compile-graph-audit --config examples/soma_workspace_compile_graph_audit.toml
cargo run --quiet --bin soma_experiment -- control-tower-compile-gate-v4 --config examples/soma_control_tower_compile_gate_v4.toml
```

Related docs:

- `docs/SPRINT87_COMPILE_GATE_RECOVERY.md`
- `docs/WORKSPACE_COMPILE_GRAPH_AUDIT.md`
- `docs/BROAD_INTEGRATION_FAMILY_SUITES.md`
- `docs/NO_RUN_VS_FULL_WORKSPACE_GATE.md`
- `docs/SPRINT87_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, model training, browser execution, Mamba runtime, or Gated DeltaNet runtime.

## Sprint 88: Seven Blocker Family Isolation and Measured Gate Recovery

Sprint 88 follows Sprint 87 by turning the remaining workspace gate into an explicit seven-family recovery queue, adding per-family compile/no-run/execution probes, preserving `committee_cli_safety` isolation, and keeping real no-run/full workspace attempts honest.

- `sprint88-seven-blocker-recover`
- `seven-blocker-family-recovery`
- `per-family-compile-probe`
- `per-family-no-run-probe`
- `per-family-execution-probe`
- `candle-expansion-recovery`
- `external-prediction-recovery`
- `krx-evidence-recovery`
- `dashboard-renderer-recovery`
- `committee-cli-safety-isolation`
- `baseline-signal-recovery`
- `counterfactual-backfill-recovery`
- `dev-dependency-impact-probe`
- `feature-variant-impact-probe`
- `measured-target-delta-v4`
- `real-no-run-gate-attempt-v3`
- `real-full-workspace-gate-attempt-v6`
- `workspace-gate-recovery-v5`
- `remaining-blocker-queue-v4`
- `control-tower-seven-blocker`

```bash
cargo run --quiet --bin soma_experiment -- sprint88-seven-blocker-recover --config examples/soma_sprint88_seven_blocker_recover.toml
cargo run --quiet --bin soma_experiment -- seven-blocker-family-recovery --config examples/soma_seven_blocker_family_recovery.toml
cargo run --quiet --bin soma_experiment -- control-tower-seven-blocker --config examples/soma_control_tower_seven_blocker.toml
```

Related docs:

- `docs/SPRINT88_SEVEN_BLOCKER_RECOVERY.md`
- `docs/PER_FAMILY_COMPILE_PROBES.md`
- `docs/SEVEN_BLOCKER_FAMILY_RECOVERY.md`
- `docs/COMMITTEE_CLI_SAFETY_ISOLATION.md`
- `docs/SPRINT88_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, runtime LLM, model training, browser execution, Mamba runtime, or Gated DeltaNet runtime.

## Sprint 89: CandleExpansionOps Real Reduction and Queue Advancement

Sprint 89 follows Sprint 88 by focusing the first real blocker-family reduction pass on `CandleExpansionOps`, keeping assertion/safety preservation explicit, and advancing the seven-blocker queue only when the candle evidence is honest.

- `sprint89-candle-recover`
- `candle-real-reduction-plan`
- `candle-assertion-migration`
- `candle-fixture-setup-reduction`
- `candle-compile-impact`
- `candle-no-run-rerun`
- `candle-full-gate-rerun`
- `seven-blocker-queue-progress-v5`
- `measured-target-delta-v5`
- `real-no-run-gate-attempt-v4`
- `real-full-workspace-gate-attempt-v7`
- `workspace-gate-recovery-v6`
- `remaining-blocker-queue-v5`
- `safety-coverage-preservation-v5`
- `control-tower-candle-recovery`

```bash
cargo run --quiet --bin soma_experiment -- sprint89-candle-recover --config examples/soma_sprint89_candle_recover.toml
cargo run --quiet --bin soma_experiment -- measured-target-delta-v5 --config examples/soma_measured_target_delta_v5.toml
cargo run --quiet --bin soma_experiment -- control-tower-candle-recovery --config examples/soma_control_tower_candle_recovery.toml
```

Related docs:

- `docs/SPRINT89_CANDLE_RECOVERY.md`
- `docs/CANDLE_EXPANSION_REAL_REDUCTION.md`
- `docs/MEASURED_GATE_RERUNS.md`
- `docs/SEVEN_BLOCKER_QUEUE_PROGRESS.md`
- `docs/SPRINT89_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, runtime LLM, model training, browser execution, Mamba runtime, or Gated DeltaNet runtime.

## Sprint 83: Acceptance Recovery and Fixture Hardening

Sprint 83 follows Sprint 82 by hardening evidence-depth fixtures, diagnosing the long-running full-workspace gate, and making the focused-vs-full distinction explicit.

- `sprint83-acceptance-recovery`
- `full-workspace-acceptance-recovery`
- `long-compilation-diagnosis`
- `evidence-depth-fixture-audit`
- `evidence-depth-fixture-normalize`
- `evidence-depth-fixture-completeness`
- `evidence-depth-determinism-regression`
- `sprint82-smoke-compress`
- `fixture-boundary-audit-v2`
- `test-runtime-recovery-plan`
- `workspace-acceptance-recovery-gate`
- `control-tower-sprint83-recovery`

```bash
cargo run --quiet --bin soma_experiment -- sprint83-acceptance-recovery --config examples/soma_sprint83_acceptance_recovery.toml
cargo run --quiet --bin soma_experiment -- long-compilation-diagnosis --config examples/soma_long_compilation_diagnosis.toml
cargo run --quiet --bin soma_experiment -- workspace-acceptance-recovery-gate --config examples/soma_workspace_acceptance_recovery_gate.toml
```

Related docs:

- `docs/SPRINT83_ACCEPTANCE_RECOVERY.md`
- `docs/EVIDENCE_DEPTH_FIXTURE_HARDENING.md`
- `docs/WORKSPACE_TEST_RUNTIME_RECOVERY.md`
- `docs/SPRINT82_SMOKE_COMPRESSION.md`
- `docs/SPRINT83_REPORT.md`

These commands remain **local-only, deterministic, research-only, paper-only, and read-only**. They do **not** add live trading, broker/order/account controls, runtime inference, Mamba runtime, Gated DeltaNet runtime, or model training.
# Sprint 96 BaselineSignal recovery

- `cargo run --quiet --bin soma_experiment -- sprint96-baseline-signal-recover --config examples/soma_sprint96_baseline_signal_recover.toml`
- `cargo run --quiet --bin soma_experiment -- baseline-signal-real-reduction-plan --config examples/soma_baseline_signal_real_reduction_plan.toml`
- `cargo run --quiet --bin soma_experiment -- baseline-signal-feature-regime-preservation --config examples/soma_baseline_signal_feature_regime_preservation.toml`
- `cargo run --quiet --bin soma_experiment -- baseline-signal-notrade-default-preservation --config examples/soma_baseline_signal_notrade_default_preservation.toml`
- `cargo run --quiet --bin soma_experiment -- baseline-signal-no-run-rerun --config examples/soma_baseline_signal_no_run_rerun.toml`
- `cargo run --quiet --bin soma_experiment -- baseline-signal-full-gate-rerun --config examples/soma_baseline_signal_full_gate_rerun.toml`
- `cargo run --quiet --bin soma_experiment -- counterfactual-backfill-entry-gate --config examples/soma_counterfactual_backfill_entry_gate.toml`

Sprint 96 keeps BaselineSignal research-only and local-only, preserves NoTrade/Risk Governor/data-quality/source-boundary/no-lookahead semantics, and only opens CounterfactualBackfill entry/precheck without starting CounterfactualBackfill reduction.
