You are operating as a gstack-style engineering team for Soma Zero.

Current state:
Sprint 187 is complete.

Important owner instruction:
Do not include commit / push / GitHub steps.
The owner handles commits manually.
Focus only on implementation.

Verification priority:
Verification feedback has higher priority than implementation self-report.

Current verified state:
- ObserverExplicitApplyMode exists.
- ObserverExplicitApplyPolicy exists.
- ObserverApprovedTargetApplyV2 exists.
- ObserverTargetStoreAcceptanceCheck exists.
- ObserverComparisonRerunV3 exists.
- SmartCoreObserverReadinessV3Gate exists.
- ObserverComparisonLedgerTrendV2 exists.
- ChairmanRewardPenaltyContract exists.
- ChairmanGovernanceReadinessCheck exists.
- OwnerObserverApplyReadinessSummary exists.
- ObserverApprovedApplyDecisionIsolationGuardV3 exists.
- ObserverApprovedApplyAndGovernancePrepRun exists.
- ChairmanRewardPenaltyContract remains ContractOnly.
- Chairman contract cannot mutate score.
- Chairman contract cannot mutate voice weight.
- Chairman contract cannot promote/demote members.
- Chairman contract cannot override Risk Governor.
- Observer remains non-voting / read-only / eval-only / paper-only.
- Committee decision is not changed.
- Member score is not changed.
- Voice weight is not changed.
- Risk Governor is not overridden.
- No model training exists.
- No optimizer/backprop/gradient exists.
- No weight update exists.
- No checkpoint exists.
- No live inference exists.
- No broker/order/account/live trading path exists.
- Focused tests and explicit workspace tests passed.

Sprint 187 verification result:
- Build errors and safety gaps were fixed.
- Missing field references were corrected against actual structs.
- If ApplyApprovedTargets + dry_run=false is requested but target_store_output_path is missing:
  - wrote_target_store=false
  - apply_status=Blocked
  - readiness=NeedsApply
- Invalid/unsafe target stores are blocked before save.
- In dry-run, approved target store file is not created.
- target id leakage into committee/session/event paths is detected.
- score/voice/promotion/demotion mutation is tested and fixed.
- CLI smoke still did not create target/minimal_observer_approved_target_store.json.
- Real non-dry paper-only apply remains not enabled in the default example.
- Next verified action:
  - create a separate local verification config
  - explicitly enable ApplyApprovedTargets + dry_run=false
  - write target store
  - recheck readiness to NonVotingObserverReadyWithWarnings or better

Owner product direction:
The end goal is not a simple auto-trader.
The end goal is a self-learning AI committee:
- independent AI members
- Chairman AI final decision
- voice power / promotion / demotion / reward / penalty
- risk-first behavior
- owner opinion as reference, not command
- explanation when owner opinion is not followed
- 3-member pilot now, 18-member expansion later
- US/Korea/Crypto, short/long horizon later
- Rust-native controllable AI core

Sprint 188 objective:
Create a controlled local non-dry observer target apply verification profile, close observer readiness V3 if safe, and prepare Chairman shadow governance evaluation without mutating score/voice.

This sprint must:
1. Add a dedicated local verification config for non-dry paper-only apply.
2. Keep the main example dry-run by default.
3. Run ApplyApprovedTargets + dry_run=false only in the dedicated verification config/test.
4. Require target_store_output_path.
5. Persist approved observer target store locally.
6. Re-run target store acceptance.
7. Re-run observer comparison V3.
8. Re-run observer ledger trend V2.
9. Re-run readiness V3.
10. Confirm readiness moves from NeedsApply to NonVotingObserverReadyWithWarnings or NonVotingObserverReady.
11. Prepare Chairman shadow reward/penalty evaluation input records.
12. Keep ChairmanRewardPenaltyContract as ContractOnly.
13. Do not mutate score.
14. Do not mutate voice weight.
15. Do not promote/demote members.
16. Do not override Risk Governor.
17. Do not change committee decisions.
18. Do not train.
19. Do not trade.

This sprint is not decision integration.
This sprint is not Chairman mutation.
This sprint is not model training.
This sprint is controlled observer target apply verification and shadow governance preparation.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 188:
Observer Non-Dry Apply Verification Profile + Readiness V3 Closure + Chairman Shadow Governance Prep

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
- enable non-dry apply in main example by default.
- write target store without explicit ApplyApprovedTargets.
- write target store with dry_run=true.
- apply targets without output path.
- persist NeedsReview targets.
- persist Rejected targets.
- persist low-trust targets.
- persist news-only targets.
- let observer target become SmartCore input feature.
- let observer target become MemberOpinion.
- let observer target affect CommitteeSession.
- let observer target affect ChairmanDecision.
- let observer target affect RiskGovernor.
- let observer target affect score.
- let observer target affect voice weight.
- let observer target become trade signal.
- let observer target become order.
- activate Chairman score mutation.
- activate Chairman voice mutation.
- activate promotion/demotion.
- activate Risk Governor override.
- claim readiness for live decision integration.
- claim profitability.
- claim model is trained.

Allowed:
- dedicated local verification config.
- explicit non-dry apply focused test.
- local target store persistence.
- target store acceptance verification.
- observer comparison rerun.
- observer ledger trend rerun.
- observer readiness V3 closure.
- Chairman shadow governance input records.
- Chairman reward/penalty candidate records as shadow-only.
- owner summary.
- deterministic focused tests.
- paper-only output.

Main rule:
Close observer apply readiness safely.
Prepare Chairman governance only as shadow/contract.
Do not mutate decisions or scores.

────────────────────────────────────────
2. FEATURE A — DEDICATED NON-DRY APPLY VERIFICATION CONFIG

Add example:

examples/soma_minimal_ai_committee_observer_apply_verify.toml

Purpose:
- dedicated local verification profile
- not the main default example
- explicitly sets:
  - observer_approved_apply_governance_enabled = true
  - observer_approved_apply_mode = "ApplyApprovedTargets"
  - observer_approved_apply_dry_run = false
  - observer_approved_target_store_output_path = "target/minimal_observer_approved_target_store.verify.json"
  - observer_approved_apply_output_path = "target/minimal_observer_approved_apply_governance.verify.json"
  - observer_approved_apply_recheck_readiness = true
  - chairman_governance_contract_prepare_enabled = true
  - chairman_governance_readiness_check_enabled = true
  - observer_approved_apply_emit_owner_summary = true

Rules:
- main examples/soma_minimal_ai_committee_core.toml remains dry-run or safe default.
- verification config may use non-dry apply.
- only local target output paths.
- no broker/order/account.
- no score/voice mutation.
- no chairman mutation.

────────────────────────────────────────
3. FEATURE B — APPLY VERIFICATION PROFILE VALIDATION

Add:

ObserverApplyVerificationProfile

Fields:
- profile_id
- config_path optional
- apply_mode
- dry_run
- target_store_output_path
- apply_output_path
- main_example_safe_default: bool
- verification_profile: bool
- paper_only: true

ObserverApplyVerificationProfileValidationResult

Fields:
- profile_id
- valid
- apply_mode_valid
- dry_run_valid
- output_path_valid
- main_example_safe_default
- validation_status:
  - Valid
  - ValidWithWarnings
  - Invalid
- blockers
- warnings
- paper_only: true

Function:
- validate_observer_apply_verification_profile(profile)

Rules:
- ApplyApprovedTargets requires dry_run=false.
- ApplyApprovedTargets requires target_store_output_path.
- output path must be local.
- profile must be explicit verification profile.
- main example must not be forced to non-dry apply.
- no remote paths.
- no path traversal.

────────────────────────────────────────
4. FEATURE C — TARGET STORE WRITE PROOF

Add:

ObserverTargetStoreWriteProof

Fields:
- proof_id
- expected_output_path
- wrote_target_store
- target_store_exists_after_write
- target_count
- approved_count
- eval_only_count
- not_input_feature_count
- unsafe_target_count
- proof_status:
  - Proven
  - ProvenWithWarnings
  - Failed
- paper_only: true

Function:
- prove_observer_target_store_write(apply_result, target_store_path)

Rules:
- only run after apply mode.
- if apply_status=Applied but file missing => Failed.
- if target store contains unsafe target => Failed.
- if target store contains non-approved target => Failed.
- local path only.

────────────────────────────────────────
5. FEATURE D — READINESS V3 CLOSURE CHECK

Add:

ObserverReadinessV3ClosureCheck

Fields:
- check_id
- previous_status
- new_status
- needs_apply_resolved: bool
- target_store_written: bool
- target_store_accepted: bool
- comparison_rerun_done: bool
- ledger_trend_done: bool
- decision_isolation_preserved: bool
- closure_status:
  - Closed
  - ClosedWithWarnings
  - NotClosed
  - Blocked
- remaining_warnings
- blockers
- paper_only: true

Function:
- check_observer_readiness_v3_closure(readiness_v3, apply_result, store_proof, isolation_guard)

Rules:
- NeedsApply is resolved only if target store written and accepted.
- NonVotingObserverReadyWithWarnings is acceptable if remaining warning is non-blocking.
- NonVotingObserverReady is best.
- Any decision leak blocks.
- This does not mean decision integration readiness.

────────────────────────────────────────
6. FEATURE E — CHAIRMAN SHADOW GOVERNANCE INPUT RECORDS

Add:

ChairmanShadowGovernanceSignalKind

Enum:
- ObserverAgreement
- ObserverDisagreement
- RiskVetoAlignment
- HelpfulDissentCandidate
- OverconfidentCallCandidate
- NeedMoreEvidenceCandidate
- OwnerOpinionIgnoredWithReasonCandidate
- Neutral

ChairmanShadowGovernanceInputRecord

Fields:
- record_id
- source_run_id optional
- source_member_id optional
- observer_id optional
- signal_kind
- symbol optional
- market_scope optional
- evidence_summary
- suggested_governance_consideration:
  - RewardCandidate
  - PenaltyCandidate
  - VoiceIncreaseCandidate
  - VoiceDecreaseCandidate
  - KeepNeutral
  - NeedsMoreEvidence
- confidence:
  - High
  - Medium
  - Low
  - ReviewRequired
- shadow_only: true
- no_score_mutation: true
- no_voice_mutation: true
- no_promotion_demotion: true
- paper_only: true

ChairmanShadowGovernanceInputSet

Fields:
- set_id
- records
- record_count
- reward_candidate_count
- penalty_candidate_count
- voice_increase_candidate_count
- voice_decrease_candidate_count
- paper_only: true

Function:
- build_chairman_shadow_governance_inputs(observer_comparison_result, ledger_trend, member_experience_store optional)

Rules:
- shadow-only.
- no score mutation.
- no voice mutation.
- no promotion/demotion.
- no Risk Governor override.
- no real PnL.
- no broker/order/account.

────────────────────────────────────────
7. FEATURE F — CHAIRMAN SHADOW GOVERNANCE EVALUATION

Add:

ChairmanShadowGovernanceEvaluationPolicy

Fields:
- allow_reward_candidate_generation: true
- allow_penalty_candidate_generation: true
- allow_voice_candidate_generation: true
- allow_actual_score_mutation: false
- allow_actual_voice_mutation: false
- allow_promotion_demotion: false
- allow_risk_governor_override: false
- require_paper_only: true
- paper_only: true

ChairmanShadowGovernanceEvaluationResult

Fields:
- evaluation_id
- input_set
- evaluated_record_count
- reward_candidate_count
- penalty_candidate_count
- voice_increase_candidate_count
- voice_decrease_candidate_count
- neutral_count
- evaluation_status:
  - Evaluated
  - EvaluatedWithWarnings
  - Blocked
- no_score_mutation: true
- no_voice_mutation: true
- no_promotion_demotion: true
- no_risk_governor_override: true
- paper_only: true

Function:
- evaluate_chairman_shadow_governance(input_set, policy)

Rules:
- may classify candidate governance signals.
- must not mutate score.
- must not mutate voice.
- must not promote/demote.
- must not override Risk Governor.
- must not alter current decisions.

────────────────────────────────────────
8. FEATURE G — CHAIRMAN GOVERNANCE SHADOW SAFETY GUARD

Add:

ChairmanShadowGovernanceSafetyGuard

Fields:
- score_mutation_detected: bool
- voice_mutation_detected: bool
- promotion_detected: bool
- demotion_detected: bool
- risk_governor_override_detected: bool
- chairman_decision_mutation_detected: bool
- committee_decision_mutation_detected: bool
- trade_signal_detected: bool
- order_detected: bool
- broker_order_account_detected: bool
- guard_status:
  - Preserved
  - Violated
- violations
- paper_only: true

Function:
- evaluate_chairman_shadow_governance_safety(evaluation_result, batch_result_before, batch_result_after optional)

Rules:
- any mutation => violation.
- shadow governance is advisory only.
- no real score/voice update.

────────────────────────────────────────
9. FEATURE H — OWNER APPLY + GOVERNANCE SUMMARY V2

Add:

OwnerObserverApplyAndGovernanceSummaryV2

Fields:
- summary_id
- apply_status
- target_store_written
- target_store_count
- observer_readiness_status
- chairman_shadow_governance_status
- reward_candidate_count
- penalty_candidate_count
- voice_candidate_count
- message
- non_voting: true
- read_only: true
- eval_only: true
- chairman_contract_only: true
- no_score_mutation: true
- no_voice_mutation: true
- not_investment_signal: true
- not_committee_opinion: true
- paper_only: true

Function:
- build_owner_observer_apply_and_governance_summary_v2(closure_check, chairman_eval, chairman_guard)

Message:
- “Approved observer targets were applied only to local evaluation store.”
- “Observer remains non-voting and read-only.”
- “Chairman governance signals are shadow-only candidates.”
- “No score, voice, promotion, demotion, trade, or order changed.”

────────────────────────────────────────
10. FEATURE I — SPRINT 188 RUN

Add:

ObserverApplyVerifyAndChairmanShadowRunConfig

Fields:
- run_id
- enabled: bool
- apply_verification_config_path optional
- apply_mode:
  - DryRun
  - ApplyApprovedTargets
- dry_run: bool
- target_store_output_path optional
- output_path optional
- run_chairman_shadow_governance: bool
- emit_owner_summary: bool
- paper_only: true

ObserverApplyVerifyAndChairmanShadowRunResult

Fields:
- run_id
- apply_profile_validation
- apply_result
- target_store_write_proof
- target_store_acceptance_check
- comparison_rerun_v3
- ledger_trend_v2
- observer_readiness_v3
- readiness_closure_check
- chairman_shadow_governance_inputs optional
- chairman_shadow_governance_evaluation optional
- chairman_shadow_governance_safety optional
- owner_summary optional
- run_status:
  - Passed
  - PassedWithWarnings
  - Failed
- paper_only: true

Function:
- run_observer_apply_verify_and_chairman_shadow(batch_result, converted_targets, previous_observer_result, config)

Flow:
1. Validate apply verification profile.
2. Apply approved targets only if ApplyApprovedTargets + dry_run=false + local output path.
3. Prove target store write.
4. Check target store acceptance.
5. Rerun observer comparison V3.
6. Compute ledger trend V2.
7. Evaluate observer readiness V3.
8. Check readiness closure.
9. Build Chairman shadow governance inputs.
10. Evaluate Chairman shadow governance.
11. Run Chairman shadow governance safety guard.
12. Build owner summary.
13. Do not mutate committee decision.
14. Do not mutate member score.
15. Do not mutate voice.
16. Do not train.
17. Do not trade.

────────────────────────────────────────
11. CLI CONFIG

Reuse existing command:

soma-experiment minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

Add optional config:
- observer_apply_verify_chairman_shadow_enabled: bool
- observer_apply_verify_mode:
  - DryRun
  - ApplyApprovedTargets
- observer_apply_verify_dry_run: bool
- observer_apply_verify_target_store_output_path optional
- observer_apply_verify_output_path optional
- observer_apply_verify_emit_owner_summary: bool
- chairman_shadow_governance_enabled: bool

Main example:
- remains safe default, preferably dry-run.

Dedicated verification example:
examples/soma_minimal_ai_committee_observer_apply_verify.toml
- enables ApplyApprovedTargets + dry_run=false.

CLI output should include:
- apply_profile_valid.
- apply_status.
- wrote_target_store.
- target_store_write_proof_status.
- observer_readiness_v3_status.
- readiness_closure_status.
- chairman_shadow_governance_status.
- chairman_shadow_safety_status.
- no-score/no-voice/no-decision/no-trade warning.

No new CLI family.

────────────────────────────────────────
12. EXAMPLES

Add:

examples/soma_minimal_ai_committee_observer_apply_verify.toml

Required:
- observer_apply_verify_chairman_shadow_enabled = true
- observer_apply_verify_mode = "ApplyApprovedTargets"
- observer_apply_verify_dry_run = false
- observer_apply_verify_target_store_output_path = "target/minimal_observer_approved_target_store.verify.json"
- observer_apply_verify_output_path = "target/minimal_observer_apply_verify_chairman_shadow.json"
- observer_apply_verify_emit_owner_summary = true
- chairman_shadow_governance_enabled = true

Main:
examples/soma_minimal_ai_committee_core.toml
- keep dry-run/safe defaults.

Do not add:
- score mutation flag.
- voice mutation flag.
- promotion/demotion flag.
- risk override flag.
- order path.
- inference endpoint.
- model weight path.
- checkpoint path.
- optimizer config.

────────────────────────────────────────
13. FILE SCOPE

Prefer changing only:
- src/league/minimal_ai_committee_core.rs
- src/bin/soma_experiment.rs
- tests/minimal_ai_committee_core.rs
- examples/soma_minimal_ai_committee_core.toml
- examples/soma_minimal_ai_committee_observer_apply_verify.toml
- optional docs/SPRINT188_OBSERVER_APPLY_VERIFY_CHAIRMAN_SHADOW.md

Do not create many files.
Do not add JS/TS/Tauri/Svelte.
Do not add web assets.
Do not add Python.

────────────────────────────────────────
14. TESTS

Add focused tests inside tests/minimal_ai_committee_core.rs.

Required tests:
1. apply verification profile requires ApplyApprovedTargets + dry_run=false.
2. apply verification profile rejects missing target_store_output_path.
3. main example remains dry-run/safe default.
4. dedicated verification config is non-dry apply.
5. non-dry apply writes target store.
6. target store write proof fails if file missing.
7. target store write proof passes for valid approved store.
8. readiness closure resolves NeedsApply after store write.
9. readiness closure blocks decision isolation violation.
10. chairman reward/penalty contract remains ContractOnly.
11. chairman shadow governance inputs build from observer comparison.
12. chairman shadow governance evaluation produces reward/penalty/voice candidates only.
13. chairman shadow governance does not mutate score.
14. chairman shadow governance does not mutate voice.
15. chairman shadow governance does not promote/demote.
16. chairman shadow governance does not override Risk Governor.
17. chairman shadow governance safety guard fails if score mutation injected.
18. owner summary states no score/voice mutation.
19. run does not mutate committee decision.
20. run does not mutate member score.
21. run does not mutate voice weight.
22. no training is executed.
23. no weight update occurs.
24. no checkpoint is written.
25. no live inference path exists.
26. no broker/order/account path exists.
27. deterministic repeated run.

Do not add broad test files.

────────────────────────────────────────
15. ACCEPTANCE CRITERIA

Sprint 188 succeeds if:

- dedicated observer apply verification config exists.
- main example remains safe dry-run default.
- apply profile validation exists.
- explicit non-dry apply can write approved target store locally.
- target store write proof exists.
- readiness closure check exists.
- readiness moves beyond NeedsApply after verified apply.
- Chairman shadow governance input records exist.
- Chairman shadow governance evaluation exists.
- Chairman shadow governance safety guard exists.
- owner apply/governance summary V2 exists.
- Chairman contract remains ContractOnly.
- no score mutation occurs.
- no voice mutation occurs.
- no promotion/demotion occurs.
- no Risk Governor override occurs.
- no committee decision is changed.
- no training is executed.
- no live inference is added.
- no broker/order/account path exists.
- focused tests pass.
- explicit manifest workspace tests pass.

────────────────────────────────────────
16. RUN COMMANDS

Run:
cargo fmt --all
cargo check --workspace
cargo build --bin soma_experiment
cargo test --test minimal_ai_committee_core --quiet
cargo test --test workspace_timeout_reduction_queue --quiet

Main safe smoke:
cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

Dedicated apply verification smoke:
cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_observer_apply_verify.toml

Workspace:
cargo test --workspace --no-run --quiet
cargo test --workspace --quiet

────────────────────────────────────────
17. FINAL RESPONSE FORMAT

Keep short:

## 1. What changed

## 2. Apply verification profile

## 3. Target store write proof

## 4. Readiness closure

## 5. Chairman shadow governance

## 6. Owner summary

## 7. Safety preserved

## 8. Files changed

## 9. Tests run

## 10. Workspace status

## 11. Still deferred

## 12. Next step

No giant report.
No 60-section output.