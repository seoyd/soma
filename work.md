You are operating as a gstack-style engineering team for Soma Zero.

Current state:
Sprint 178 is complete.

Important owner instruction:
Do not include commit / push / GitHub steps.
The owner handles commits manually.
Focus only on implementation.

Current verified state:
- Rust-native SmartCoreMicroKernelV0 exists.
- Mamba3-style temporal cell v0 exists.
- Gated DeltaNet-style memory cell v0 exists.
- SmartCore toy head projection v0 exists.
- SmartCore debug outputs exist for 3 pilot members:
  - TrendEntryAI
  - RiskGuardAI
  - EvidenceRegimeAI.
- Shadow alignment exists.
- SmartCore debug output is compared against:
  - MemberOpinion
  - replay target labels
  - RiskGovernorStatus.
- Mismatch records exist.
- Mismatch records generate research tasks.
- Research tasks execute through safe local source registry.
- Calibration target candidates exist.
- CoreCalibrationDataset exists.
- CoreCalibrationQualitySummary exists.
- NoDecisionBridgeGuard exists.
- SmartCore output is not MemberOpinion.
- SmartCore output is not CommitteeDecision.
- SmartCore output is not RiskGovernor input.
- SmartCore output is not trade signal.
- SmartCore output is not order.
- No training exists.
- No optimizer/backprop/gradient exists.
- No weight update exists.
- No checkpoint exists.
- No live inference exists.
- No broker/order/account/live trading path exists.
- cargo test --workspace --no-run --quiet passes.
- cargo test --workspace --quiet passes.
- Acceptance is based on explicit manifest target set.

Sprint 178 result:
- mismatch data need records = 15.
- generated research tasks = 12.
- research tasks executed = 12.
- generated evidence = 88.
- calibration target candidates = 88.
- approved targets = 12.
- calibration dataset examples = 36 -> 40.
- target_count = 9 -> 40.
- mismatch_count = 15 -> 15.
- alignment recheck status = NoChange.
- calibration quality status = NeedsMoreTargets.
- no_decision_recheck_status = Preserved.
- no training / no live inference / no broker-order-account path.

Interpretation:
The self-growing evidence loop is working.
The calibration target count increased.
But the toy SmartCore head output did not improve because no calibration adjustment has been applied yet.
The next step is not real training.
The next step is a debug-only calibration overlay derived from the calibration dataset.

Owner direction:
The AI core must improve over time.
But the system must not jump into real training too early.
The core can first learn through a safe calibration overlay:
- no model weight mutation.
- no optimizer.
- no backprop.
- no checkpoint.
- no live inference.
- no decision integration.
- debug-only recalibration.

Sprint 179 objective:
Build a SmartCore calibration overlay v0 that uses CoreCalibrationDataset to adjust debug head buckets in shadow mode, then re-run shadow alignment and measure whether mismatch improves.

This sprint must:
1. Build per-member/per-head calibration statistics from CoreCalibrationDataset.
2. Detect dominant mismatch patterns.
3. Build debug-only calibration rules.
4. Apply calibration overlay to SmartCoreDebugOutputV0.
5. Produce CalibratedSmartCoreDebugOutputV0.
6. Re-run shadow alignment on calibrated outputs.
7. Compare pre/post mismatch count.
8. Ensure calibrated output is still not MemberOpinion.
9. Ensure calibrated output is still not CommitteeDecision.
10. Ensure calibrated output is still not trade signal.
11. Ensure no model weights are mutated.
12. Keep Mamba3/Gated runtime and real training deferred.

This sprint is not real model training.
This sprint is not production inference.
This sprint is debug-only calibration overlay.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 179:
SmartCore Calibration Overlay v0 + Shadow Recalibration Pass + No-Decision Calibration Guard

────────────────────────────────────────
1. HARD RULES

Do not add:
- real model training
- optimizer
- gradients
- backpropagation
- weight mutation
- checkpoint writing
- persistent learned weights
- real Mamba3 runtime parity claim
- real Gated DeltaNet runtime parity claim
- real Sparse Attention runtime
- live inference
- production inference endpoint
- broker/order/account
- live trading
- order execution
- real PnL claim
- central MoE committee brain
- Python
- PyTorch
- TensorFlow
- CUDA
- Candle
- Burn
- ONNX runtime
- web UI
- Tauri/Svelte/React/JS
- browser scraping
- broad test suite
- report bloat
- commit / push / GitHub steps

Do not:
- mutate SmartCore weights.
- create persistent model parameters.
- call calibration overlay “training.”
- claim model is trained.
- claim profitability.
- claim live readiness.
- use calibrated debug output as MemberOpinion.
- use calibrated debug output as CommitteeSession input.
- use calibrated debug output as ChairmanDecision input.
- use calibrated debug output as RiskGovernor input.
- use calibrated debug output as trade signal.
- use calibrated debug output as order.
- mutate member score from calibration.
- mutate replay input features.
- leak labels into microkernel input.
- store broker/order/account fields.

Allowed:
- calibration statistics.
- calibration rule table.
- debug-only overlay.
- calibrated debug output.
- shadow recalibration comparison.
- pre/post mismatch delta.
- owner debug summary.
- local JSON output.
- deterministic focused tests.
- paper-only output.

Main rule:
Calibrate debug output.
Do not train.
Do not decide.
Do not trade.

────────────────────────────────────────
2. FEATURE A — CORE CALIBRATION STATISTICS

Add:

SmartCoreHeadCalibrationStatsV0

Fields:
- stats_id
- member_id
- head:
  - Stance
  - Risk
  - EvidenceNeed
  - ConfidenceCalibration
  - Uncertainty
  - ExpectedReturnHint
- example_count
- match_count
- mismatch_count
- unknown_count
- deferred_count
- mismatch_rate
- dominant_debug_bucket optional
- dominant_target_bucket optional
- dominant_mismatch_type optional
- paper_only: true

SmartCoreCalibrationStatsSummaryV0

Fields:
- summary_id
- dataset_id
- member_count
- head_count
- total_examples
- per_head_stats
- per_member_mismatch_rate
- overall_mismatch_rate
- stats_status:
  - Sufficient
  - ThinData
  - NeedsMoreTargets
  - Unsafe
- paper_only: true

Function:
- compute_smartcore_calibration_stats_v0(calibration_dataset)

Rules:
- statistics only.
- no weight update.
- no training.
- deterministic ordering.
- weak/review-required calibration examples should not create strong rules.
- if data count too small, stats_status=NeedsMoreTargets or ThinData.

────────────────────────────────────────
3. FEATURE B — CALIBRATION RULE TABLE

Add:

SmartCoreCalibrationRuleActionV0

Enum:
- Keep
- MapBucket
- LowerConfidence
- RaiseEvidenceNeed
- RaiseRiskBucket
- LowerRiskBucket
- MarkUnknown
- KeepObserving

SmartCoreCalibrationRuleV0

Fields:
- rule_id
- member_id
- head
- from_bucket
- to_bucket optional
- action
- support_count
- mismatch_reduction_estimate
- confidence:
  - High
  - Medium
  - Low
  - ReviewRequired
- rule_status:
  - Active
  - ObserveOnly
  - Disabled
- reason
- paper_only: true

SmartCoreCalibrationRuleTableV0

Fields:
- table_id
- source_dataset_id
- rules
- active_rule_count
- observe_only_rule_count
- disabled_rule_count
- paper_only: true

Function:
- build_smartcore_calibration_rule_table_v0(stats_summary, policy)

Rules:
- only create Active rule if enough support.
- low support => ObserveOnly.
- contradictory evidence => Disabled or ObserveOnly.
- no rule can create order/trade signal.
- no rule can claim real prediction.
- rule table is not model weights.
- rule table is not checkpoint.
- rule table is debug overlay only.

────────────────────────────────────────
4. FEATURE C — CALIBRATION OVERLAY POLICY

Add:

SmartCoreCalibrationOverlayPolicyV0

Fields:
- min_support_for_active_rule
- max_rules_per_member_head
- allow_stance_bucket_mapping: true
- allow_risk_bucket_mapping: true
- allow_evidence_bucket_mapping: true
- allow_confidence_adjustment: true
- allow_expected_return_mapping: false
- allow_trade_signal_output: false
- allow_member_opinion_output: false
- allow_committee_decision_output: false
- debug_only: true
- paper_only: true

Default:
- expected return mapping disabled.
- trade/member/committee output disabled.
- debug_only=true.

Rules:
- overlay may adjust debug buckets only.
- overlay may not create MemberOpinion.
- overlay may not alter committee decision.
- overlay may not alter RiskGovernor result.
- overlay may not mutate weights.

────────────────────────────────────────
5. FEATURE D — CALIBRATED DEBUG OUTPUT

Add:

CalibratedSmartCoreHeadOutputV0

Fields:
- member_id
- head
- original_bucket
- calibrated_bucket
- calibration_action
- applied_rule_id optional
- rule_confidence
- changed: bool
- debug_only: true
- not_investment_signal: true
- not_committee_opinion: true
- paper_only: true

CalibratedSmartCoreDebugOutputV0

Fields:
- calibrated_output_id
- source_debug_output_id
- member_id
- calibrated_heads
- applied_rule_count
- changed_head_count
- calibration_summary
- debug_only: true
- not_investment_signal: true
- not_committee_opinion: true
- not_order: true
- no_training: true
- no_weight_update: true
- no_checkpoint: true
- paper_only: true

CalibratedSmartCoreDebugOutputBatchV0

Fields:
- batch_id
- source_debug_batch_id optional
- member_outputs
- output_count
- changed_output_count
- debug_only: true
- paper_only: true

Function:
- apply_smartcore_calibration_overlay_v0(debug_output_batch, rule_table, policy)

Rules:
- apply only Active rules.
- ObserveOnly rules do not change output.
- preserve original output.
- no weights mutated.
- no checkpoint.
- no training.
- no decision use.

────────────────────────────────────────
6. FEATURE E — CALIBRATION OVERLAY SAFETY GUARD

Add:

SmartCoreCalibrationOverlaySafetyGuardV0

Fields:
- debug_only: bool
- no_training: bool
- no_weight_update: bool
- no_checkpoint: bool
- no_live_inference: bool
- not_member_opinion: bool
- not_committee_decision: bool
- not_trade_signal: bool
- not_order: bool
- no_broker_order_account: bool
- labels_not_in_input: bool
- safety_status:
  - Preserved
  - Violated
- violations
- paper_only: true

Function:
- evaluate_smartcore_calibration_overlay_safety_v0(calibrated_batch, rule_table)

Rules:
- if calibrated output claims MemberOpinion => violation.
- if calibrated output claims trade signal => violation.
- if calibrated output changes committee decision => violation.
- if rule table looks like persistent learned weight checkpoint => violation.
- if labels injected into input features => violation.

────────────────────────────────────────
7. FEATURE F — SHADOW RECALIBRATION PASS

Add:

SmartCoreShadowRecalibrationRunConfig

Fields:
- run_id
- enabled: bool
- calibration_dataset_path optional
- rule_table_output_path optional
- calibrated_debug_output_path optional
- recalibration_result_output_path optional
- dry_run: bool
- paper_only: true

SmartCoreShadowRecalibrationRunResult

Fields:
- run_id
- stats_summary
- rule_table
- calibrated_debug_output_batch
- recalibrated_alignment_result
- pre_mismatch_count
- post_mismatch_count
- mismatch_delta
- pre_alignment_status
- post_alignment_status
- overlay_safety_guard
- no_decision_recheck
- run_status:
  - Passed
  - PassedWithWarnings
  - Failed
- warnings
- paper_only: true

Function:
- run_smartcore_shadow_recalibration_pass(debug_output_batch, calibration_dataset, previous_alignment_result, batch_result, config)

Flow:
1. Compute calibration stats.
2. Build calibration rule table.
3. Apply calibration overlay to debug output.
4. Re-run shadow alignment using calibrated output.
5. Compare pre/post mismatch counts.
6. Evaluate overlay safety.
7. Recheck no-decision boundary.
8. If dry_run=true, write no output except CLI result.
9. If dry_run=false, write local JSON outputs.
10. Do not train.
11. Do not mutate weights.
12. Do not change committee decision.

────────────────────────────────────────
8. FEATURE G — RECALIBRATION INTERPRETATION

Add:

SmartCoreRecalibrationInterpretationV0

Fields:
- interpretation_id
- mismatch_delta
- improved: bool
- worsened: bool
- no_change: bool
- data_sufficiency:
  - SufficientForOverlay
  - NeedsMoreTargets
  - TooSparse
- next_recommended_step:
  - KeepCollectingCalibrationTargets
  - TuneCalibrationPolicy
  - ProceedToShadowOpinionCandidate
  - KeepDebugOnly
- human_readable_summary
- debug_only: true
- paper_only: true

Function:
- interpret_smartcore_recalibration_result_v0(result)

Rules:
- If mismatch improves and safety preserved:
  - next may be ShadowOpinionCandidate, still not decision.
- If no change:
  - continue collecting calibration targets or tune policy.
- If worsens:
  - disable active rules or keep observe-only.
- No live inference.
- No trading.

────────────────────────────────────────
9. FEATURE H — OWNER CONSOLE RECALIBRATION SUMMARY

Add:

OwnerCoreRecalibrationDebugSummary

Fields:
- summary_id
- pre_mismatch_count
- post_mismatch_count
- mismatch_delta
- active_rule_count
- changed_output_count
- interpretation
- message
- debug_only: true
- not_investment_signal: true
- not_committee_opinion: true
- paper_only: true

Function:
- build_owner_core_recalibration_debug_summary(result)

Message should say:
- “Calibration overlay is debug-only.”
- “It does not train or update weights.”
- “It is not used for committee decisions.”
- “It is not a trading signal.”

────────────────────────────────────────
10. CLI CONFIG

Reuse existing command:

soma-experiment minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

Add optional config:
- smartcore_recalibration_enabled: bool
- smartcore_recalibration_dry_run: bool
- smartcore_recalibration_rule_table_output_path optional
- smartcore_calibrated_debug_output_path optional
- smartcore_recalibration_result_output_path optional
- smartcore_recalibration_min_support
- smartcore_recalibration_max_rules_per_member_head
- smartcore_recalibration_emit_owner_summary: bool

CLI output should include:
- calibration_stats_status.
- active_rule_count.
- observe_only_rule_count.
- changed_output_count.
- pre_mismatch_count.
- post_mismatch_count.
- mismatch_delta.
- overlay_safety_status.
- no_decision_recheck_status.
- interpretation next step.
- debug-only/no-training/no-decision warning.

No new CLI family.

────────────────────────────────────────
11. EXAMPLES

Update:
examples/soma_minimal_ai_committee_core.toml

Add:
- smartcore_recalibration_enabled = true or false depending safety.
- smartcore_recalibration_dry_run = true by default.
- smartcore_recalibration_min_support = 2.
- smartcore_recalibration_max_rules_per_member_head = 2.
- smartcore_recalibration_emit_owner_summary = true.
- smartcore_recalibration_rule_table_output_path = "target/minimal_smartcore_calibration_rule_table.json"
- smartcore_calibrated_debug_output_path = "target/minimal_smartcore_calibrated_debug_output.json"
- smartcore_recalibration_result_output_path = "target/minimal_smartcore_recalibration_result.json"

Do not add:
- training config.
- optimizer.
- checkpoint.
- model weight path.
- inference endpoint.
- order path.

────────────────────────────────────────
12. FILE SCOPE

Prefer changing only:
- src/league/minimal_ai_committee_core.rs
- src/bin/soma_experiment.rs
- tests/minimal_ai_committee_core.rs
- examples/soma_minimal_ai_committee_core.toml
- optional docs/SPRINT179_SMARTCORE_CALIBRATION_OVERLAY.md

Do not create many files.
Do not add JS/TS/Tauri/Svelte.
Do not add web assets.
Do not add Python.

────────────────────────────────────────
13. TESTS

Add focused tests inside tests/minimal_ai_committee_core.rs.

Required tests:
1. calibration stats counts mismatches.
2. calibration stats groups by member/head.
3. low-support mismatch creates ObserveOnly rule.
4. sufficient support mismatch creates Active rule.
5. expected return mapping disabled by default.
6. overlay policy forbids trade signal output.
7. calibration overlay changes only debug buckets.
8. calibration overlay does not mutate original output.
9. calibrated output remains not_investment_signal.
10. calibrated output remains not_committee_opinion.
11. calibrated output remains not_order.
12. safety guard detects MemberOpinion misuse.
13. safety guard detects trade signal misuse.
14. recalibration pass recomputes alignment.
15. recalibration pass reports mismatch_delta.
16. no-decision recheck remains Preserved.
17. owner recalibration summary is debug-only.
18. recalibration dry-run writes no files.
19. no training is executed.
20. no weight update occurs.
21. no checkpoint is written.
22. no live inference path exists.
23. no broker/order/account path exists.
24. deterministic repeated recalibration pass.

Do not add broad test files.

────────────────────────────────────────
14. ACCEPTANCE CRITERIA

Sprint 179 succeeds if:

- SmartCoreHeadCalibrationStatsV0 exists.
- SmartCoreCalibrationStatsSummaryV0 exists.
- SmartCoreCalibrationRuleTableV0 exists.
- SmartCoreCalibrationOverlayPolicyV0 exists.
- CalibratedSmartCoreDebugOutputV0 exists.
- SmartCoreCalibrationOverlaySafetyGuardV0 exists.
- SmartCoreShadowRecalibrationRun exists.
- SmartCoreRecalibrationInterpretationV0 exists.
- OwnerCoreRecalibrationDebugSummary exists.
- calibration rules are built from calibration dataset.
- overlay applies only debug bucket changes.
- original debug output is preserved.
- shadow alignment can be re-run after overlay.
- mismatch delta is reported.
- no decision bridge remains preserved.
- calibrated output is not MemberOpinion.
- calibrated output is not CommitteeDecision.
- calibrated output is not TradeSignal.
- no training is executed.
- no weight mutation occurs.
- no checkpoint is written.
- no live inference is added.
- no broker/order/account path exists.
- focused tests pass.
- explicit manifest workspace tests pass.

────────────────────────────────────────
15. RUN COMMANDS

Run:
cargo fmt --all
cargo check --workspace
cargo build --bin soma_experiment
cargo test --test minimal_ai_committee_core --quiet
cargo test --test workspace_timeout_reduction_queue --quiet

Cycle smoke:
cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

Workspace:
cargo test --workspace --no-run --quiet
cargo test --workspace --quiet

────────────────────────────────────────
16. FINAL RESPONSE FORMAT

Keep short:

## 1. What changed

## 2. Calibration stats

## 3. Calibration rule table

## 4. Calibrated debug output

## 5. Shadow recalibration pass

## 6. Recalibration interpretation

## 7. Safety preserved

## 8. Files changed

## 9. Tests run

## 10. Workspace status

## 11. Still deferred

## 12. Next step

No giant report.
No 60-section output.