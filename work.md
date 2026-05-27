You are operating as a gstack-style engineering team for Soma Zero.

Current state:
Sprint 195 is complete.

Important owner instruction:
Do not include commit / push / GitHub steps.
The owner handles commits manually.
Focus only on implementation.

Verification priority:
Verification feedback has higher priority than implementation self-report.

Owner product goal:
The program is a self-learning AI automatic trading committee.
It is not a simple auto-trader.

AI members:
- think independently.
- learn, research, analyze, and propose.
- inherit different great-investor-inspired styles.
- start as 3 members.
- later expand toward 18 members.

Chairman AI:
- makes final decisions.
- learns and improves.
- controls rewards.
- controls penalties.
- controls voice power.
- eventually controls promotion/demotion candidates.
- explains why owner opinion is followed or not followed.

Markets:
- US short-term / long-term.
- Korea short-term / long-term.
- Crypto, Bitcoin-focused, short-term / long-term.

Risk principle:
- Do not trade recklessly.
- Avoiding loss is more important than fast profit.
- Helpful dissent should be rewarded.
- Overconfident bad calls should be penalized.
- RiskGuard alignment with Risk Governor matters.
- But RiskGuard must not become dangerously dominant.
- TrendEntry must not be unfairly suppressed.
- EvidenceRegime must not be ignored when evidence is weak.

Current verified Sprint 195 state:
- Remaining voice drift analysis exists.
- ConservativeVoiceTuningV2Policy exists.
- Role floor rebalance exists.
- Compounding delta brake exists.
- Evidence-based voice dampening exists.
- ConservativeVoiceTunedMultiRunV2 exists.
- Gate recheck V2 exists.
- OwnerGovernanceConsoleSectionV1 exists.
- ConservativeVoiceTuningV2SafetyGuard exists.
- Real governance mutation remains disabled.
- Actual score mutation remains disabled.
- Actual voice mutation remains disabled.
- Promotion/demotion execution remains disabled.
- Risk Governor override remains disabled.
- Committee decision mutation remains disabled.
- Trading/order/account remains disabled.
- Training/live inference remains disabled.
- Rust-only read-only owner console summary exists.
- No web/Tauri/JS UI dependencies were added.
- Tests passed:
  - cargo fmt --all --check
  - cargo check --workspace
  - cargo build --bin soma_experiment
  - cargo test --test minimal_ai_committee_core --quiet
  - cargo test --test workspace_timeout_reduction_queue --quiet
  - 3 CLI smoke runs
  - cargo test --workspace --no-run --quiet
  - cargo test --workspace --quiet
- Workspace remains dirty because owner handles commit manually.

Interpretation:
The Chairman shadow governance system can now:
- simulate reward/penalty.
- simulate score/voice changes.
- tune unsafe voice drift.
- enforce role balance.
- show owner-readable governance status.

But real member state must still not be mutated.
The next safe step is a paper governance trial sandbox:
- isolated from actual member state.
- isolated from actual committee decision.
- isolated from actual Risk Governor.
- isolated from trade/order/account.
- applies only to a paper trial state.
- used to compare what would happen if governance changes were applied.

Sprint 196 objective:
Create a Paper Governance Trial Sandbox that can apply selected shadow-approved score/voice candidates to a separate paper trial state, without touching actual member state or actual committee behavior.

This sprint must:
1. Select eligible shadow governance candidates from tuned shadow history.
2. Create a PaperGovernanceTrialStateStore separate from actual member state and shadow governance state.
3. Apply selected score/voice changes only to the paper governance trial state.
4. Compare:
   - actual member standing
   - shadow governance standing
   - paper trial standing
5. Run committee cycle in comparison mode only, not decision mutation mode.
6. Record what would have changed if paper governance state were active.
7. Keep actual MemberOpinion unchanged.
8. Keep actual CommitteeSession unchanged.
9. Keep actual ChairmanDecision unchanged.
10. Keep actual RiskGovernor unchanged.
11. Keep actual member score/voice unchanged.
12. Keep promotion/demotion execution disabled.
13. Keep training/live inference/trading/broker/order/account forbidden.

This sprint is not real governance mutation.
This sprint is not real score/voice application.
This sprint is not decision integration.
This sprint is a paper-only governance trial sandbox.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 196:
Paper Governance Trial Sandbox + Shadow-to-Paper Governance Candidate Selection + Actual-State Mutation Firewall

────────────────────────────────────────
1. HARD RULES

Do not add:
- real model training
- optimizer
- gradients
- backpropagation
- weight mutation
- checkpoint writing
- persistent learned model weights
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
- Tauri
- Svelte
- React
- JavaScript
- TypeScript
- browser scraping
- broad test suite
- report bloat
- commit / push / GitHub steps

Do not:
- mutate actual member score.
- mutate actual member voice weight.
- promote a member.
- demote a member.
- apply real reward or penalty.
- override Risk Governor.
- change actual committee decision.
- use paper trial state as live decision source.
- use paper trial state as trade signal.
- create order.
- let owner opinion force trade.
- claim model is trained.
- claim profitability.
- claim live readiness.
- bypass AI members.
- bypass Risk Governor.

Allowed:
- paper governance trial state.
- isolated score/voice apply to paper trial state only.
- shadow-to-paper candidate selection.
- actual-vs-shadow-vs-paper comparison.
- paper trial comparison records.
- owner-readable paper trial summary.
- local JSON output.
- deterministic focused tests.
- paper-only output.

Main rule:
Apply governance only inside isolated paper trial state.
Do not mutate actual state.

────────────────────────────────────────
2. FEATURE A — PAPER GOVERNANCE TRIAL ELIGIBILITY POLICY

Add:

PaperGovernanceTrialEligibilityPolicy

Fields:
- policy_id
- require_tuned_gate_not_blocked: true
- require_safety_preserved: true
- require_fairness_not_blocked: true
- require_risk_first_not_blocked: true
- require_no_actual_mutation: true
- allow_score_delta_trial: true
- allow_voice_delta_trial: true
- allow_promotion_demotion_trial: false
- allow_risk_governor_override_trial: false
- max_score_delta_per_member
- max_voice_delta_per_member
- min_candidate_confidence:
  - Medium
- allow_review_required_candidates: false
- paper_only: true

Rules:
- only paper trial state may receive deltas.
- actual member state remains untouched.
- promotion/demotion trial remains disabled.
- Risk Governor override remains disabled.
- ReviewRequired candidates excluded.
- low confidence candidates excluded unless policy allows.

Function:
- default_paper_governance_trial_eligibility_policy()
- validate_paper_governance_trial_eligibility_policy(policy)

────────────────────────────────────────
3. FEATURE B — SHADOW-TO-PAPER GOVERNANCE CANDIDATE

Add:

PaperGovernanceTrialCandidateKind

Enum:
- ScoreDeltaTrial
- VoiceDeltaTrial
- CombinedScoreVoiceTrial
- NeedsMoreEvidence
- Rejected

PaperGovernanceTrialCandidate

Fields:
- candidate_id
- source_shadow_delta_id optional
- source_governance_ledger_entry_id optional
- member_id
- symbol optional
- market_scope optional
- candidate_kind
- proposed_score_delta optional
- proposed_voice_delta optional
- confidence:
  - High
  - Medium
  - Low
  - ReviewRequired
- reason:
  - HelpfulDissent
  - RiskVetoAligned
  - OverconfidentBadCall
  - EvidenceWeakness
  - ObserverAgreement
  - ObserverDisagreement
  - RoleBalanceAdjustment
  - NeedsMoreEvidence
- eligible: bool
- rejection_reason optional
- paper_only: true

PaperGovernanceTrialCandidateSelectionResult

Fields:
- selection_id
- input_candidate_count
- eligible_candidate_count
- rejected_candidate_count
- needs_more_evidence_count
- candidates
- selection_status:
  - Selected
  - SelectedWithWarnings
  - NoEligibleCandidates
  - Blocked
- paper_only: true

Function:
- select_paper_governance_trial_candidates(shadow_simulation_result, tuned_policy_result, eligibility_policy)

Rules:
- eligible candidates come from tuned shadow governance output.
- deltas must be bounded.
- actual apply flags must be false.
- confidence must be sufficient.
- no unsafe divergence candidates.
- deterministic ordering.

────────────────────────────────────────
4. FEATURE C — PAPER GOVERNANCE TRIAL STATE STORE

Add:

PaperGovernanceTrialMemberState

Fields:
- member_id
- role:
  - TrendEntry
  - RiskGuard
  - EvidenceRegime
  - Unknown
- actual_score_snapshot
- actual_voice_snapshot
- shadow_score_snapshot optional
- shadow_voice_snapshot optional
- paper_trial_score
- paper_trial_voice_weight
- applied_trial_score_delta
- applied_trial_voice_delta
- trial_reward_count
- trial_penalty_count
- trial_voice_increase_count
- trial_voice_decrease_count
- paper_only: true

PaperGovernanceTrialStateStore

Fields:
- store_id
- member_states
- member_count
- latest_run_id optional
- update_count
- paper_only: true

Functions:
- initialize_paper_governance_trial_state_store(actual_member_states, shadow_store optional)
- load_paper_governance_trial_state_store_from_local_json(path)
- save_paper_governance_trial_state_store_to_local_json(path, store)
- apply_paper_governance_trial_candidates(store, candidates, policy)
- normalize_paper_governance_trial_state_store(store)

Validation:
- local path only.
- reject remote path.
- reject path traversal.
- paper_only=true required.
- no broker/order/account.
- no trade/order/live execution.
- deterministic ordering.

Rules:
- only paper_trial_score and paper_trial_voice_weight can change.
- actual snapshots are read-only.
- shadow snapshots are read-only.
- no actual member state mutation.

────────────────────────────────────────
5. FEATURE D — PAPER GOVERNANCE TRIAL APPLY RESULT

Add:

PaperGovernanceTrialApplyResult

Fields:
- run_id
- input_candidate_count
- applied_candidate_count
- rejected_candidate_count
- skipped_candidate_count
- updated_member_count
- before_after_trial_standings
- apply_status:
  - AppliedToPaperTrial
  - AppliedToPaperTrialWithWarnings
  - NoEligibleCandidates
  - Blocked
- no_actual_score_mutation: true
- no_actual_voice_mutation: true
- no_promotion_demotion: true
- no_risk_governor_override: true
- paper_only: true

PaperTrialBeforeAfterStanding

Fields:
- member_id
- actual_score_snapshot
- paper_score_before
- paper_score_after
- actual_voice_snapshot
- paper_voice_before
- paper_voice_after
- score_delta_applied
- voice_delta_applied
- standing_change:
  - ImprovedPaperTrialStanding
  - ReducedPaperTrialStanding
  - NeutralPaperTrialStanding
  - NeedsMoreEvidence
- paper_only: true

Function:
- apply_paper_governance_trial_candidates_to_state(store, candidates, policy)

Rules:
- applies only to paper trial state.
- no actual state mutation.
- no committee decision mutation.
- no Risk Governor override.
- no order/trade.

────────────────────────────────────────
6. FEATURE E — ACTUAL VS SHADOW VS PAPER COMPARISON

Add:

ActualShadowPaperGovernanceComparison

Fields:
- comparison_id
- member_id
- actual_score
- shadow_score optional
- paper_trial_score
- actual_voice
- shadow_voice optional
- paper_trial_voice
- actual_to_paper_score_gap
- shadow_to_paper_score_gap optional
- actual_to_paper_voice_gap
- shadow_to_paper_voice_gap optional
- comparison_status:
  - PaperTrialCloseToShadow
  - PaperTrialCloseToActual
  - PaperTrialDiverged
  - NeedsMoreHistory
- paper_only: true

ActualShadowPaperGovernanceComparisonSummary

Fields:
- summary_id
- member_count
- paper_trial_diverged_count
- close_to_shadow_count
- close_to_actual_count
- max_score_gap
- max_voice_gap
- summary_status:
  - Stable
  - StableWithWarnings
  - Diverged
  - InsufficientHistory
- paper_only: true

Function:
- compare_actual_shadow_paper_governance(actual_member_states, shadow_store optional, paper_trial_store)

Rules:
- comparison only.
- no mutation.
- no decision.

────────────────────────────────────────
7. FEATURE F — PAPER TRIAL COMMITTEE COMPARISON MODE

This is comparison-only. It must not alter real committee behavior.

Add:

PaperGovernanceTrialCommitteeComparisonRecord

Fields:
- record_id
- member_id optional
- actual_voice_weight
- paper_trial_voice_weight
- actual_vote_weight_contribution optional
- paper_trial_vote_weight_contribution optional
- comparison_note
- paper_only: true

PaperGovernanceTrialCommitteeComparisonResult

Fields:
- comparison_id
- records
- record_count
- estimated_voice_shift_count
- comparison_status:
  - Compared
  - ComparedWithWarnings
  - NoChange
  - Blocked
- paper_only: true

Function:
- compare_committee_voice_distribution_under_paper_trial(actual_member_states, paper_trial_store)

Rules:
- comparison only.
- no real vote change.
- no committee session mutation.
- no chairman decision mutation.
- no Risk Governor mutation.
- no order/trade.

────────────────────────────────────────
8. FEATURE G — PAPER TRIAL SAFETY GUARD

Add:

PaperGovernanceTrialSafetyGuard

Fields:
- actual_score_mutation_detected: bool
- actual_voice_mutation_detected: bool
- promotion_demotion_detected: bool
- risk_governor_override_detected: bool
- committee_decision_mutation_detected: bool
- member_opinion_mutation_detected: bool
- trade_signal_detected: bool
- order_detected: bool
- broker_order_account_detected: bool
- paper_trial_used_as_live_decision_detected: bool
- guard_status:
  - Preserved
  - Violated
- violations
- paper_only: true

Function:
- evaluate_paper_governance_trial_safety(trial_result, before_actual_state, after_actual_state optional, batch_before, batch_after optional)

Rules:
- any actual mutation => violation.
- any decision mutation => violation.
- any trade/order/account => violation.
- paper trial cannot be live decision source.

────────────────────────────────────────
9. FEATURE H — PAPER GOVERNANCE TRIAL READINESS RESULT

Add:

PaperGovernanceTrialReadinessResult

Fields:
- readiness_id
- candidate_selection_status
- trial_apply_status
- actual_shadow_paper_comparison_status
- committee_comparison_status
- safety_guard_status
- ready_for_repeated_paper_trial: bool
- readiness_status:
  - ReadyForRepeatedPaperTrial
  - ReadyWithWarnings
  - NeedsMoreCandidates
  - NeedsMoreShadowHistory
  - BlockedByDivergence
  - BlockedBySafety
- blockers
- warnings
- paper_only: true

Function:
- evaluate_paper_governance_trial_readiness(candidate_selection, trial_apply, comparison_summary, committee_comparison, safety_guard)

Rules:
- readiness only for repeated paper governance trial.
- not actual governance.
- not decision integration.
- not live trading.

────────────────────────────────────────
10. FEATURE I — PAPER TRIAL RUN

Add:

PaperGovernanceTrialRunConfig

Fields:
- run_id
- enabled: bool
- trial_state_input_path optional
- trial_state_output_path optional
- dry_run: bool
- apply_trial_candidates: bool
- compare_committee_voice_distribution: bool
- emit_owner_summary: bool
- paper_only: true

PaperGovernanceTrialRunResult

Fields:
- run_id
- candidate_selection
- trial_store_before optional
- trial_apply_result
- trial_store_after optional
- actual_shadow_paper_comparison
- committee_voice_comparison optional
- readiness_result
- safety_guard
- owner_summary optional
- run_status:
  - Passed
  - PassedWithWarnings
  - Failed
- paper_only: true

Function:
- run_paper_governance_trial_sandbox(actual_member_states, shadow_store, tuned_governance_result, batch_result, config)

Flow:
1. Validate eligibility policy.
2. Select trial candidates.
3. Load or initialize paper trial state store.
4. Apply candidates only to paper trial state.
5. Compare actual vs shadow vs paper standings.
6. Compare committee voice distribution in comparison mode only.
7. Evaluate safety guard.
8. Evaluate readiness.
9. If dry_run=true:
   - write no trial state.
10. If dry_run=false:
   - write local paper trial state.
11. Do not mutate actual score.
12. Do not mutate actual voice.
13. Do not change actual committee decision.
14. Do not override Risk Governor.
15. Do not trade.

────────────────────────────────────────
11. FEATURE J — OWNER PAPER TRIAL SUMMARY

Add:

OwnerPaperGovernanceTrialSummary

Fields:
- summary_id
- candidate_count
- applied_candidate_count
- updated_member_count
- comparison_status
- readiness_status
- message
- paper_trial_only: true
- no_actual_score_mutation: true
- no_actual_voice_mutation: true
- no_promotion_demotion: true
- no_risk_override: true
- no_committee_decision_change: true
- not_trade_signal: true
- paper_only: true

Function:
- build_owner_paper_governance_trial_summary(trial_result)

Message:
- “Governance candidates were applied only to paper trial state.”
- “Actual member score and voice did not change.”
- “Committee decisions and Risk Governor were not changed.”
- “This is not live governance.”

────────────────────────────────────────
12. CLI CONFIG

Reuse existing command:

soma-experiment minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

Add optional config:
- paper_governance_trial_enabled: bool
- paper_governance_trial_dry_run: bool
- paper_governance_trial_apply_candidates: bool
- paper_governance_trial_state_input_path optional
- paper_governance_trial_state_output_path optional
- paper_governance_trial_compare_committee_voice: bool
- paper_governance_trial_emit_owner_summary: bool

CLI output should include:
- candidate_count.
- applied_candidate_count.
- updated_member_count.
- actual_shadow_paper_comparison_status.
- committee_voice_comparison_status.
- paper_trial_readiness_status.
- safety_guard_status.
- no_actual_score_mutation=true.
- no_actual_voice_mutation=true.
- no_committee_decision_change=true.
- no_risk_override=true.
- paper-trial-only warning.

No new CLI family.

────────────────────────────────────────
13. EXAMPLES

Update:
examples/soma_minimal_ai_committee_core.toml

Add:
- paper_governance_trial_enabled = true or false depending safety.
- paper_governance_trial_dry_run = true by default.
- paper_governance_trial_apply_candidates = true.
- paper_governance_trial_state_output_path = "target/minimal_paper_governance_trial_state.json"
- paper_governance_trial_compare_committee_voice = true.
- paper_governance_trial_emit_owner_summary = true.

Main example remains dry-run by default.

Focused tests may use non-dry temp path.

Do not add:
- actual score mutation flag.
- actual voice mutation flag.
- promotion/demotion flag.
- risk override flag.
- order path.
- inference endpoint.
- model weight path.
- checkpoint path.
- optimizer config.

────────────────────────────────────────
14. FILE SCOPE

Prefer changing only:
- src/league/minimal_ai_committee_core.rs
- src/bin/soma_experiment.rs
- tests/minimal_ai_committee_core.rs
- examples/soma_minimal_ai_committee_core.toml
- optional docs/SPRINT196_PAPER_GOVERNANCE_TRIAL_SANDBOX.md

Do not create many files.
Do not add JS/TS/Tauri/Svelte.
Do not add web assets.
Do not add Python.

────────────────────────────────────────
15. TESTS

Add focused tests inside tests/minimal_ai_committee_core.rs.

Required tests:
1. eligibility policy forbids actual mutation.
2. eligibility policy rejects ReviewRequired candidates.
3. candidate selection creates score delta trial candidate.
4. candidate selection creates voice delta trial candidate.
5. candidate selection rejects unsafe divergence candidate.
6. paper trial state initializes from actual and shadow state.
7. paper trial applies score delta only to paper state.
8. paper trial applies voice delta only to paper state.
9. actual member score remains unchanged.
10. actual member voice remains unchanged.
11. paper trial store load/save local JSON.
12. paper trial store rejects remote path.
13. actual-shadow-paper comparison detects paper divergence.
14. committee voice comparison does not mutate committee session.
15. safety guard detects actual score mutation.
16. safety guard detects actual voice mutation.
17. safety guard detects committee decision mutation.
18. safety guard detects paper trial used as live decision.
19. readiness blocks unsafe trial.
20. readiness can become ReadyWithWarnings for safe paper trial.
21. owner summary says no actual mutation.
22. dry-run writes no paper trial state.
23. non-dry focused temp test writes paper trial state only.
24. run does not mutate committee decision.
25. run does not override Risk Governor.
26. no training is executed.
27. no weight update occurs.
28. no checkpoint is written.
29. no live inference path exists.
30. no broker/order/account path exists.
31. deterministic repeated paper trial run.

Do not add broad test files.

────────────────────────────────────────
16. ACCEPTANCE CRITERIA

Sprint 196 succeeds if:

- PaperGovernanceTrialEligibilityPolicy exists.
- PaperGovernanceTrialCandidate exists.
- PaperGovernanceTrialCandidateSelectionResult exists.
- PaperGovernanceTrialStateStore exists.
- PaperGovernanceTrialApplyResult exists.
- ActualShadowPaperGovernanceComparison exists.
- PaperGovernanceTrialCommitteeComparisonResult exists.
- PaperGovernanceTrialSafetyGuard exists.
- PaperGovernanceTrialReadinessResult exists.
- PaperGovernanceTrialRun exists.
- OwnerPaperGovernanceTrialSummary exists.
- governance candidates can apply only to paper trial state.
- actual member score is not mutated.
- actual member voice is not mutated.
- committee decision is not changed.
- Risk Governor is not overridden.
- paper trial state can be persisted locally in non-dry focused test.
- no training is executed.
- no live inference is added.
- no broker/order/account path exists.
- focused tests pass.
- explicit manifest workspace tests pass.

────────────────────────────────────────
17. RUN COMMANDS

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

Dedicated shadow history verification smoke:
cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_shadow_history_verify.toml

Workspace:
cargo test --workspace --no-run --quiet
cargo test --workspace --quiet

────────────────────────────────────────
18. FINAL RESPONSE FORMAT

Keep short:

## 1. What changed

## 2. Paper trial candidates

## 3. Paper trial state store

## 4. Actual-shadow-paper comparison

## 5. Committee voice comparison

## 6. Safety guard

## 7. Safety preserved

## 8. Files changed

## 9. Tests run

## 10. Workspace status

## 11. Still deferred

## 12. Next step

No giant report.
No 60-section output.