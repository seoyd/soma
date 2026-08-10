# SOMA Sprint 105-RD1-R1-R1 Work Report

## 1. Mode and Git State

Mode: REPORT_INTEGRITY_REPAIR_ONLY.

This phase changes the existing RD1-R1 report only. Its delta contains no production source change, no test change, no causal-probe change, no V2 implementation, no dependency change, and no delivery or Metal change. No staging, commit, push, or network action was performed. The working tree already contained unrelated and earlier-phase changes; they were not modified by this report-only repair.

## 2. Previous RD1-R1 Review Finding

Independent review found exactly one reason for CHANGES_REQUESTED: the RD1-R1 report described inactive softsign control flow too broadly. It incorrectly grouped every relevant softsign use under softsign_activations and described input_activity too narrowly. The active production path remains tanh / affine_tanh; the Q1 verdict, reproduced causal measurements, candidate direction, and V2 property contracts are not reinterpreted here.

## 3. Progress Ledger

| Stage | Status | Evidence | Finding |
| --- | --- | --- | --- |
| Starting RD1-R1 review | VERIFIED | independent review | Documentation control-flow finding isolated |
| Production profile | VERIFIED | source | reduced profile is false / 1.0 / false |
| Active tanh path | VERIFIED | source | fusion is tanh; output is affine_tanh |
| reinforced_retention softsign branch | VERIFIED | source | separate retention-response family |
| softsign_activations branch | VERIFIED | source | separate fusion/output activation family |
| input_activity calculation | VERIFIED | source | calculated for every forward input step |
| input_activity consumers | VERIFIED | source | reinforced forward and backward retention paths consume it |
| Current equation report | CORRECT | source/report | retained equation matches active reduced branch |
| Softsign control-flow wording | CORRECT | source/report | two control families are now separated |
| input_activity wording | CORRECT | source/report | no longer described as tape-only |
| Activation salvage | EVIDENCE_ALIGNED | report | active activation components are UNRESOLVED |
| Order rationale | PRESERVED | report | state distinction and downstream compression retained |
| Interference rationale | PRESERVED | report | active update/retention path remains the basis |
| Local rationale | PRESERVED | report | active state/local fusion path remains the basis |
| Candidate A | REJECTED | report | multiple structural boundaries, not activation naming |
| Candidate B | SUPPORTED | report | structural evidence and preserved shell only |
| Candidate C | WEAK | report | no direct outer-shell causal failure evidence |
| V2 property contract | PRESERVED | report | property-level, activation-neutral requirements retained |
| Q1 freeze | PRESERVED | diff | no fixture, seed, metric, threshold, or semantic change |
| C2 boundary | PRESERVED | report | deferred and non-canonical; production gain absent |
| Production source change | NO | diff | report-only delta |
| Test change | NO | diff | report-only delta |
| Delivery | FROZEN | diff | no delivery change |
| Metal | FROZEN | diff | no Metal change |
| fmt/check | PASS | compiler | sequential checks passed |
| New warnings | 0 | compiler | only existing unrelated warnings reproduced |
| RD1-R1-R1 | READY_FOR_INDEPENDENT_REVIEW | source/report/compiler | documentation repair is complete |

## 4. Active Production Profile

The source-defined production profile is REDUCED_CORE_MATH_PROFILE_V1:

| Property | Current production value | Resolution evidence |
| --- | --- | --- |
| reinforced_retention | false | normal forward and training/loss paths pass the reduced profile |
| readout_gain | 1.0 | reduced profile value used in state readout |
| softsign_activations | false | selects the active else activation branches |

The normal call graph is forward → forward_with_work_counters → forward_internal → forward_internal_with_profile(..., REDUCED_CORE_MATH_PROFILE_V1). Training forward and loss-gradient entry paths resolve the same reduced profile. Private attribution profiles can choose alternatives for tests, but callers and checkpoints cannot select them as production behavior.

## 5. Active tanh / affine_tanh Path

With softsign_activations=false, the active fusion is tanh(readout + sigmoid(skip) × u). The active block output helper is affine_tanh, implemented as affine projection followed elementwise by tanh. Input embedding and u use the same affine-then-tanh helper. The raw head is affine only.

This is the current production path. It does not imply that tanh or affine_tanh alone is the root cause of an observed causal failure.

## 6. Softsign Source Inventory

| Softsign use | Controlling condition | Active in production reduced profile? | Purpose |
| --- | --- | --- | --- |
| softsign(decay_channel × RETENTION_RESPONSE_GAIN_V2) | core_profile.reinforced_retention | No | Builds reinforced retention base response |
| derivative of that retention softsign | core_profile.reinforced_retention during backward evaluation | No | Propagates the reinforced retention response gradient |
| softsign(readout + skip × u) | core_profile.softsign_activations | No | Alternative fusion activation |
| affine_softsign(...) | core_profile.softsign_activations | No | Alternative block-output activation |
| softsign activation derivative at fusion/output | core_profile.softsign_activations during backward evaluation | No | Backpropagates the alternative activation branch |

The helpers are defined in source, but their behavior is selected only by the conditions above. The retention-response family and activation-alternative family are distinct and must not be represented as a single flag-controlled path.

## 7. reinforced_retention Softsign Boundary

reinforced_retention controls the retention-response softsign family. When true, the forward path applies softsign to the decay channel, multiplies the resulting response by input_activity and the squared current gate, and incorporates the response into the bounded decay calculation. Its backward branch uses the corresponding softsign derivative and propagates an input_activity gradient.

The current reduced production profile sets reinforced_retention=false. Therefore this reinforced retention response is not active production behavior. The reduced branch instead uses the decay channel directly in sigmoid(state_bias + decay_channel) and sets the retention response vector to zero.

## 8. softsign_activations Boundary

softsign_activations controls a different family: the activation applied after state/local fusion and the activation applied after the block output affine projection. When true, it selects softsign(value) for fusion and affine_softsign for block output, with matching activation derivatives in backward evaluation.

The current reduced production profile sets softsign_activations=false, selecting fusion tanh(value), affine_tanh output, and tanh derivatives. This flag does not govern the reinforced-retention softsign family in Section 7.

## 9. Active vs Inactive Activation Paths

| Path | Status in current reduced production profile |
| --- | --- |
| Input embedding: affine then tanh | Active |
| Recurrent u: affine then tanh | Active |
| State/local fusion: tanh(readout + skip × u) | Active |
| Block output: affine_tanh | Active |
| Reinforced retention response softsign | Inactive because reinforced_retention=false |
| Alternative fusion softsign | Inactive because softsign_activations=false |
| Alternative block affine_softsign | Inactive because softsign_activations=false |

The two inactive softsign families have different control conditions. Their existence does not make them part of the current equation or a production remedy.

## 10. input_activity Source Audit

For every input step, before embedding, source calculates:

\[
input_activity_t = mean_i(tanh(x_t,i)^2).
\]

The calculation occurs regardless of profile and regardless of whether a forward tape is requested. When a tape is requested, the value is stored in that step's tape alongside the raw input and embedding. This storage does not define its sole purpose.

## 11. input_activity Consumer Audit

input_activity has two source-level consumer contexts:

1. In the forward reinforced-retention branch, it multiplies the softsign-derived retention base response and squared current gate to form the retention response.
2. In the matching backward reinforced-retention branch, the taped value contributes to decay-channel and current-gate gradients; the accumulated input_activity_gradient is then propagated to the input through the derivative of mean(tanh(input)^2).

With reinforced_retention=false, the forward retention response is a zero vector and the backward reduced branch does not add a reinforced-retention input_activity contribution. Thus the value is calculated but does not drive current reduced production retention behavior. It is not a tape-only quantity: the alternative reinforced profile and its gradient path consume the real value.

## 12. Current Equation Documentation

The active reduced production equation remains source-aligned, in source evaluation order:

1. embedded = tanh(affine(input)); each block starts with hidden = embedded or the prior block output.
2. u = tanh(affine(hidden)); decay_channel, previous gate preactivation, and current gate preactivation are affine functions of u.
3. prev_gate and curr_gate are sigmoid of their respective preactivations. decay = decay_min + (decay_max - decay_min) × sigmoid(state_bias + decay_channel).
4. next_state = decay × previous_state + prev_gate × previous_u × tanh(prev_scale) + curr_gate × u × tanh(curr_scale).
5. readout is the per-channel average of next_state × tanh(readout_scale), multiplied by readout_gain.
6. z = tanh(readout + sigmoid(skip) × u).
7. h_output = affine_tanh(z) = tanh(affine(z)); the next block consumes h_output.
8. raw_output = affine(final_hidden), with no final output activation.

The equation intentionally excludes reinforced response terms and softsign activation alternatives because neither is active in the reduced production profile. input_activity is calculated alongside this path but is not an active reduced-retention operand.

## 13. Activation Salvage Correction

The prior salvage wording is corrected without changing the causal measurements:

| Activation component | Corrected salvage status | Evidence boundary |
| --- | --- | --- |
| Fusion tanh boundary | UNRESOLVED | A/B difference compresses across the active boundary, but tanh alone is not proven as the root cause or a required replacement |
| Block affine_tanh boundary | UNRESOLVED | A/B difference compresses across the active boundary, but no activation counterfactual proves causal sufficiency |

Observed fusion and output compression remain evidence. The component status is not forced to REWRITE, and no alternative activation is claimed to repair the failure or define the V2 solution.

## 14. Order Rationale Preservation

The frozen RD1 order evidence remains: A/B prefixes create distinct recurrent state, and the observed difference becomes smaller through fusion, output handoff, and raw readout. The active path responsible for this audit is state readout plus local skip, fusion tanh, output affine_tanh, and affine raw head. This is an observed information-survival problem, not proof that a softsign/tanh choice alone is causal.

## 15. Interference Rationale Preservation

The active reduced recurrence is decay times previous state plus gated, scaled previous u and gated, scaled current u. Its decay uses the direct decay channel in a sigmoid-bounded rule, not the inactive reinforced retention response. The frozen interference evidence therefore supports concern about state update/retention behavior and missing demonstrated relevance-conditioned retention capacity in the current active path; it does not treat the inactive reinforced profile as a production solution.

## 16. Local-Control Rationale Preservation

The active local relation is current u scaled by sigmoid skip, added to state readout, transformed by fusion tanh, passed through output affine_tanh, then read by the affine head. The frozen normal/reset/no-state counterfactual remains the basis for the local-control conclusion. Alternative softsign paths are not mixed into this rationale.

## 17. Shared Failure Mechanism

The evidence supports two coupled internal areas, no more specific causal claim:

1. State update / retention behavior.
2. State-local fusion / observable output handoff.

This formulation preserves the structural rationale without asserting a specific activation, gate, retention, residual, or fusion equation for V2.

## 18. Candidate A Recheck

Candidate A remains REJECTED. The reason is multiple structural boundaries spanning recurrence/update-retention and state/local observable handoff, with no direct evidence that a single small patch resolves all frozen causal failures. It is not rejected because tanh or affine_tanh has been declared the cause.

## 19. Candidate B Recheck

Candidate B remains SUPPORTED, pending this documentation re-review, only because:

1. Q1 structural failures span multiple internal boundaries.
2. Current recurrence/update plus retention behavior has interference evidence.
3. State/local fusion-output handoff has local-control and order-information transmission evidence.
4. The outer fixed-state shell, determinism, and mode equivalence retain value.
5. No direct evidence supports a small single patch resolving all three structural failures.

Candidate B is an activation-neutral internal rewrite direction. It specifies no V2 equation or activation choice.

## 20. Candidate C Recheck

Candidate C remains WEAK. There is insufficient evidence that the outer shell, public API, or fixed-state principle is itself the direct structural cause. Retirement/new-core remains a fallback only if a bounded internal rewrite cannot satisfy the preserved property contracts.

## 21. V2 Property Contract Preservation

The following requirements remain property-level and activation-neutral: fixed auditable state; order information surviving to an output boundary; input/state-conditioned selective update and retention; local-information preservation; state causality; no length special case; one common brain; independent AI ownership; no router, MoE, or expert dispatch; no architecture mashup; and a minimal mechanism budget. This report creates no V2 equation, gate, retention, residual, fusion, or activation specification.

## 22. Q1 Benchmark Freeze

Q1 remains CORE_NOT_VIABLE. Its fixtures, seeds, metrics, thresholds, task semantics, generator, and causal measurements are frozen. This documentation repair neither reruns nor alters the qualification.

## 23. C2 Boundary

C2 remains CHANGES_REQUESTED, with production gain absent and its calibration DEFERRED / NON-CANONICAL. Calibration is not used as a V2 rationale or as a remedy for the current causal failures.

## 24. Production Immutability

The RD1-R1-R1 delta has zero production source changes. No model equation, activation, state, readout, calibration, loss, training, role, checkpoint, delivery, or backend behavior changed. Existing earlier test-only instrumentation is outside this report-only delta and was not modified.

## 25. Test Immutability

The RD1-R1-R1 delta has zero test changes. Existing causal probes, Q1 fixtures, thresholds, seeds, classifications, and report-consistency contracts were not edited. No consistency fixture blocked the documentation correction.

## 26. Delivery / Metal Freeze

Delivery remains frozen. Metal remains frozen. No delivery behavior, Metal source, or Metal hardware execution occurred. The feature-gated compiler check is compilation-only and is not Metal hardware work.

## 27. Hardcoding Audit

| Check | Result |
| --- | --- |
| Q1 numeric result used as a production rule | 0 |
| softsign/tanh production task shortcut | 0 |
| length-specific production rule | 0 |
| Candidate B forced by preference | 0 |
| current digest literal | 0 |
| current line number as a design contract | 0 |

The report derives statements from source control flow and frozen evidence only.

## 28. Focused Verification

The following report/source checks were completed: active production profile; active tanh/affine_tanh path; reinforced-retention softsign branch; softsign_activations branch; input_activity calculation and consumers; equation comparison; activation salvage wording; Candidate B rationale; production/test/delivery/Metal diff boundaries; and report heading/status integrity.

No test was run because this delta implements documentation only and changes no executable code. Compiler commands were run one at a time with one Cargo build job and a fresh target directory:

| Verification | Result |
| --- | --- |
| cargo fmt --all -- --check | Pass |
| cargo check --offline --lib | Pass |
| cargo check --offline --lib --features backend-metal | Pass |
| git diff --check | Pass |

## 29. Explicitly Not Run

- Full
- Q1 Full Qualification
- D2
- Metal Hardware
- V2 Production Implementation
- Calibration
- Self-Learning
- Formula Lab
- Investor Constitution
- Chair AI
- Generators
- Live Trading
- Internet Learning

## 30. Warning Audit

The report-only delta introduces zero compiler warnings. Both compiler checks reproduce only four existing unrelated dead-code warnings. No warning was suppressed, reclassified, or hidden.

## 31. Status Separation

- Q1: CORE_NOT_VIABLE
- RD1: CHANGES_REQUESTED
- RD1-R1: CHANGES_REQUESTED
- RD1-R1-R1: READY_FOR_INDEPENDENT_REVIEW
- Selected Redesign: Candidate B, SUPPORTED pending independent documentation-integrity re-review
- C2: CHANGES_REQUESTED; DEFERRED / NON-CANONICAL
- Delivery: FROZEN
- Metal: FROZEN
- Overall: READY_FOR_INDEPENDENT_REVIEW

## 32. What This Fixes

This fixes the RD1-R1 report's softsign control-flow documentation: reinforced-retention softsign and activation-alternative softsign are now separately described; input_activity is accurately described as always calculated and as an alternative reinforced-path input rather than tape-only; and activation salvage is limited to the actual causal evidence.

## 33. What This Does Not Prove

It does not establish isolated activation causality, a mandatory output-nonlinearity change, sufficiency of any alternative activation, the validity of any V2 equation, or the validity of calibration, delivery, Metal hardware, trading, or external learning.

## 34. Final Status

READY_FOR_INDEPENDENT_REVIEW

## 35. Exactly One Next Step

- independent documentation-integrity re-review
