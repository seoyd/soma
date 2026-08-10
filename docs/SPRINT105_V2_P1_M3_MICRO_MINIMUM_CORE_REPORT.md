# SOMA Sprint 105-V2-P1 M3-Micro V2 Minimum Core Report

## 1. Mode and Git State

Mode: V2_MINIMUM_CORE_IMPLEMENTATION_AND_REQUALIFICATION.

This phase adds an explicit V2 candidate in the existing M3-Micro production module and test-only V2 qualification coverage. The V1 default remains intact. No staging, commit, push, network, delivery, D2, calibration, or Metal topology change was performed.

## 2. RD1-R1-R1 Implementation Authority

The independent RD1-R1-R1 implementation approval authorized an internal V2 candidate, not replacement of the V1 default. This implementation follows that boundary: V2 is explicit, CPU-qualified, and intentionally incompatible with V1 state and parameters.

## 3. Starting V1 / Q1 State

V1 remains CORE_NOT_VIABLE under frozen Q1. Its active default profile, tanh / affine_tanh path, training behavior, C1 boundary, C2 deferral, delivery, and Metal topology remain unchanged. The selected redesign direction was an internal rewrite, not a V1 profile-flag combination.

## 4. Progress Ledger

| Stage | Status | Evidence | Blocker |
| --- | --- | --- | --- |
| Starting approval | VERIFIED | independent review | none |
| V1 preservation | PASS | representative V1 profile test and diff | none |
| Revision isolation | PASS | explicit V1Reduced / V2Candidate boundary | none |
| V2 state type | COMPLETE | values plus step index only | none |
| V2 parameter set | COMPLETE | V2 layout contains local, gate, candidate, read, head families | none |
| V2 update gate | PASS | focused source and runtime gate check | none |
| V2 candidate state | PASS | focused source and gradient check | none |
| V2 bounded update | PASS | long-sequence runtime check | none |
| Local residual path | PASS | source and V2 local-control execution | none |
| Post-fusion saturation | ABSENT | source audit | none |
| Dense recurrent matrix | ABSENT | V2 layout audit | none |
| Fixed state footprint | PASS | 1,024 elements and 4,104 bytes at all Q1 lengths | none |
| Hidden history | ABSENT | state layout/runtime footprint audit | none |
| Determinism | PASS | deterministic initialization and exact mode results | none |
| Mode equivalence | PASS | Full, Streaming, Chunked focused and Q1 checks | none |
| Gradient integrity | PASS | finite difference for four representative families | none |
| Training sanity | PASS | development loss decreases | none |
| Q1 benchmark freeze | PASS | existing fixture generator and integrity helper reused | none |
| Frozen Q1 run | PASS | one complete frozen V2 execution | none |
| Delayed Cue | FAIL | base equals No-State and reset at all lengths | structural gate |
| Order Sensitive | FAIL | base equals No-State and reset at all lengths | structural gate |
| Interference | FAIL | base equals No-State and reset at all lengths | structural gate |
| Local Control | PASS | base accuracy equals No-State at 1.0 | none |
| State causality | FAIL | history-family Base does not exceed reset | structural gate |
| Structural gates | FAIL | utility and causality fail | V2 core |
| Confidence overlay | MEASURED | fresh raw mean NLL 0.74560696 | not a calibration decision |
| V2 core verdict | V2_CORE_NOT_VIABLE | frozen structural derivation | no P1 retune allowed |
| V1 default | PRESERVED | V1 profile guard | none |
| C2 calibration | DEFERRED | no gain path | none |
| Delivery | FROZEN | guard and diff | none |
| Metal | FROZEN | fail-closed candidate boundary | none |
| fmt/check | PASS | sequential compiler checks | none |
| New warnings | 0 | warning audit | none |
| V2-P1 | READY_FOR_INDEPENDENT_REVIEW | honest implementation and qualification record | core verdict is separate |

## 5. V1 / V2 Revision Boundary

The source exposes M3MicroCoreRevisionV2 with V1Reduced and V2Candidate. Existing M3MicroModel reports V1Reduced and keeps its original constructor and default behavior. M3MicroV2Candidate is a distinct model, parameter, state, layout, tape, and forward/backward boundary; it is not a Boolean combination of V1 math-profile fields.

## 6. V1 Preservation Audit

The V1 active-profile representative test passed after the V2 addition: reinforced retention remains false, readout gain remains 1.0, softsign activations remain false, and the production block output remains affine_tanh. A separate V1 sentinel forward before and after V2 candidate construction produced identical output and state. The V1 default constructor, V1 state type, V1 recurrence, and V1 checkpoint path were not redirected.

## 7. V2 Design Scope

V2 P1 implements only the approved minimum internal mechanism:

- deterministic V2 parameters;
- fixed recurrent state;
- local affine-tanh representation;
- diagonal state/input-conditioned update gate;
- diagonal state/input-conditioned candidate;
- interpolation state update;
- signed state readout;
- additive local plus memory output;
- existing affine categorical raw head;
- manual tape/backward using the existing optimizer rule.

No attention, router, expert, MoE, dense recurrent matrix, history cache, V2 calibration, or new framework is present.

## 8. V2 Persistent State

M3MicroV2State contains a fixed vector of per-block state values and a step index. Each of two blocks holds 64 × 8 = 512 state values; there is no previous_u field. Zero state initializes all values and the step index to zero. Validation requires exact shape, finite values, and bounded values.

## 9. V2 Parameter Set

The V2 layout contains 16,259 parameters for the qualified Trend shape:

| Family | Meaning |
| --- | --- |
| input embedding | input to 64-wide local input |
| block local affine | block input to local representation |
| gate state scale, input scale, bias | diagonal selective-update gate |
| candidate state scale, input scale, bias | diagonal candidate state |
| memory read scale | signed fixed-cost state read |
| existing-shape raw head | 64-wide hidden to categorical raw output |

No state_dim × state_dim matrix exists. All recurrent state parameters are elementwise vectors.

## 10. V2 Local Representation

The V2 input embedding is affine-tanh. Each V2 block then computes a local representation u through affine-tanh of the current block input. This local value remains separately observable through the additive output contract; it is neither task-specific nor a conditional bypass.

## 11. V2 Selective Update Gate

For state component i with the corresponding local channel projection u-hat:

g_i = sigmoid(gate_state_scale_i × previous_state_i + gate_input_scale_i × u-hat_i + gate_bias_i).

The focused long-sequence check observed finite gates strictly between zero and one. The gate depends on both prior state and local input, does not inspect length or task identity, and is the only retain-versus-update gate in P1.

## 12. V2 Candidate State

The candidate is:

candidate_i = tanh(candidate_state_scale_i × previous_state_i + candidate_input_scale_i × u-hat_i + candidate_bias_i).

It depends on prior state and the local input projection. It has no previous_u term, no separate decay term, and no full recurrent state matrix.

## 13. V2 State Update Equation

The V2 update is:

next_state_i = previous_state_i + gate_i × (candidate_i - previous_state_i).

Equivalently, it is elementwise interpolation between previous state and candidate. This is the only state transition in P1.

## 14. Bounded-State Proof / Runtime Check

With zero initial state, candidate values in [-1, 1], and gate values in [0, 1], the update is an elementwise interpolation and therefore remains in the candidate/prior bounded hull. The implementation adds no state clamp. The focused 96-token runtime check passed state validation; all state values remained finite and at most 1.0 plus the numerical tolerance. Q1 state maxima remained below 0.704.

## 15. V2 Memory Readout

For each 64-wide channel, V2 computes memory as the fixed-cost average of state value times a learned signed read scale over its eight state coordinates. The scale is a V2 memory-read parameter, not V1 readout_gain and not a C2 calibration gain.

## 16. Local-Preserving Additive Output

Every V2 block outputs:

hidden = local_u + memory.

The same equation is applied for every sequence and every role-shaped configuration. The No-State ablation removes only memory contribution at this composition boundary while retaining the same V2 outer model, raw head semantics, data, initialization, optimizer, and training budget.

## 17. No Post-Fusion Saturation

There is no tanh, softsign, or sigmoid after local_u + memory in V2 P1. The next block receives the additive result; the existing raw head consumes the final block result. This preserves a direct additive local path without claiming any particular activation is universally correct.

## 18. Complexity / Dense-Recurrent Audit

V2 recurrent parameters are seven state-length vectors per block: three gate vectors, three candidate vectors, and one read vector. Local affine maps are 64 × 64 and are not state × state recurrence. Persistent storage is fixed by model shape. No sequence-length-growing state allocation, token-state list, KV cache, or hidden history buffer is allocated.

## 19. Common-Brain Contract

The V2 equation family is one shared implementation. It introduces no Trend, Volatility, or Reversal-specific core equation, router, expert, or task-family branch. Separate agent instances may have separate parameters, but their V2 math is identical.

## 20. Independent-AI Ownership

M3MicroV2Candidate owns its parameter vector and M3MicroV2State is supplied per candidate instance. The implementation has no process-global mutable V2 state. Optimizer state is provided explicitly to the V2 training step, allowing each future agent instance to own it independently.

## 21. V2 State Reset Semantics

Reset means constructing a new V2 zero state: all state values are zero and step index is zero. The frozen Q1 reset intervention uses the existing family-defined reset point and only swaps in this V2 zero state; it does not move the task reset point.

## 22. Checkpoint Compatibility

- INTENTIONALLY_INCOMPATIBLE

V2 uses a different model, layout, parameter vector, and persistent state type without previous_u. It is not serialized or loaded through the V1 checkpoint contract, and no V1 parameter/state is treated as semantically valid V2 data.

## 23. Initialization

V2 uses the existing deterministic RNG style with small symmetric initialization, zero embedding/local/head biases, identity additions for input and local affines, fixed gate scales of 0.25 with zero bias, candidate state/input scales of 0.5 and 1.0 with zero bias, and read scales of 1.0. These choices were fixed before the frozen Q1 evaluation. No Q1 outcome was used to select them.

## 24. Training Integration

V2 computes the existing role-policy loss and output gradient, aggregates the same development examples, uses the existing M3MicroOptimizerState rule, and uses the frozen Q1 learning rate and fixed training budget. The public V2 training entry applies gradients to the V2 parameter vector. Development loss decreased for every qualified family and length.

## 25. Backward / Tape Integration

V2 has a distinct tape containing input, embedding, per-block local representation, previous state, gate, candidate, next state, and output. Backward covers the raw head, additive local/memory composition, read scale, interpolation update, gate, candidate, local affine-tanh, and input embedding. V1 tape fields are not reused with altered V2 meaning.

## 26. Gradient Integrity

Finite-difference checks passed for representative update-gate input scale, candidate input scale, memory read scale, and raw-head weight parameters. The training-sanity test also completed with finite gradients and reduced development loss. No gradient mismatch was observed before frozen Q1.

## 27. Fixed-State Footprint

At every frozen Q1 length 8, 16, and 32, V2 persistent state is:

| Elements | Bytes | Blocks | Zero step index |
| ---: | ---: | ---: | ---: |
| 1,024 | 4,104 | 2 | 0 |

The footprint is constant across all families and lengths.

## 28. Hidden-History Audit

The V2 state object contains only fixed per-block value vectors and a step index. Forward tape grows only when training/recording and is not persistent recurrent state. Full, streaming, and chunked execution produce exactly equal output and state in the focused mode test.

## 29. Determinism

Deterministic candidate construction with the fixed seed, repeated focused forward behavior, and exact Full/Streaming/Chunked equivalence passed. The frozen Q1 execution is deterministic by construction; it was executed once under the one-shot rule rather than rerun after its result.

## 30. CPU Mode Equivalence

The V2 focused test and every frozen Q1 entry used the same CPU reference semantics. Full sequence, one-token streaming, and three-token chunked execution were exact-equivalent in output and final state.

## 31. Metal Boundary

V2 exposes a CPU-only candidate path. Any non-default candidate backend request returns UnsupportedCandidateBackend; it cannot silently invoke a V1 Metal kernel or claim CPU fallback as Metal success. No V2 Metal shader, kernel, dispatch, topology, or hardware execution was added.

## 32. Frozen Q1 Integrity Audit

The existing Sprint 105 Q1 generator, family identities, development/frozen split, positive/negative balance, lengths, reset positions, optimizer settings, training budget, metrics, and pass/fail derivation were reused. V2 qualification changes only the candidate model construction and V2 No-State state contribution flag. A preflight comparison was corrected before frozen evaluation because equal initial losses are not a fairness condition when one path includes state in its output; the final fairness construction shares the same starting V2 model, data, initialization, optimizer configuration, budget, and raw head.

## 33. Frozen Q1 Execution

One complete frozen V2 Q1 execution ran after all focused checks passed. It covered four families at lengths 8, 16, and 32; each row has four frozen examples. No V2 equation, initialization, gate bias, parameter scale, data, seed, metric, threshold, or training budget was changed after that execution.

## 34. Delayed Cue

| Length | Base accuracy | No-State accuracy | Reset Base accuracy | Base NLL |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 0.5 | 0.5 | 0.5 | 0.7784554 |
| 16 | 0.5 | 0.5 | 0.5 | 0.7695873 |
| 32 | 0.5 | 0.5 | 0.5 | 0.76755786 |

Development loss decreased at all three lengths, but the frozen history utility and reset intervention did not separate.

## 35. Order Sensitive

| Length | Base accuracy | No-State accuracy | Reset Base accuracy | Base NLL |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 0.5 | 0.5 | 0.5 | 0.77895474 |
| 16 | 0.5 | 0.5 | 0.5 | 0.77001905 |
| 32 | 0.5 | 0.5 | 0.5 | 0.7674997 |

State remains finite and bounded, but no frozen order advantage over No-State or reset is established.

## 36. Interference Retention

| Length | Base accuracy | No-State accuracy | Reset Base accuracy | Base NLL |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 0.5 | 0.5 | 0.5 | 0.7939963 |
| 16 | 0.5 | 0.5 | 0.5 | 0.7757095 |
| 32 | 0.5 | 0.5 | 0.5 | 0.7758403 |

The minimum V2 update does not demonstrate frozen interference-retention utility under the unchanged Q1 protocol.

## 37. State-Irrelevant Control

| Length | Base accuracy | No-State accuracy | Base NLL |
| ---: | ---: | ---: | ---: |
| 8 | 1.0 | 1.0 | 0.66931736 |
| 16 | 1.0 | 1.0 | 0.6478514 |
| 32 | 1.0 | 1.0 | 0.65249556 |

The Local Control structural comparison passes: Base is not worse than the fair V2 No-State comparator.

## 38. State-Reset Causality

For Delayed Cue, Order Sensitive, and Interference Retention, Base accuracy is 0.5 and reset-before-query Base accuracy is also 0.5 at every frozen length. The required strict Base advantage therefore fails. This is a frozen structural result, not a calibration finding.

## 39. Q1 Structural Gate Matrix

| Gate | Status | Evidence |
| --- | --- | --- |
| Finite outputs/states | PASS | every Base, No-State, and reset metric is finite |
| Fixed persistent footprint | PASS | 1,024 elements and 4,104 bytes across lengths |
| Full/Streaming/Chunked equivalence | PASS | focused and per-entry checks |
| Deterministic training sanity | PASS | all development losses decrease |
| State utility at maximum length | FAIL | history-family Base accuracy equals No-State at length 32 |
| State-reset causality | FAIL | history-family Base accuracy equals reset |
| Local Control | PASS | Base accuracy is at least No-State |
| Structural gate aggregate | FAIL | utility and causality are required and failed |

## 40. Confidence Overlay

The fresh uncalibrated V2 Base mean categorical NLL across all 12 family/length entries is 0.74560696. This is a raw measurement only. No C1 value, temperature, production gain, C2 scalar, or calibration was applied. The structural failure determines the core verdict before any confidence-only classification could be relevant.

## 41. V2 Core Viability Decision

- V2_VIABLE_BASELINE: not selected.
- V2_CONDITIONALLY_VIABLE_CONFIDENCE_BLOCKED: not selected.
- V2_CORE_NOT_VIABLE: selected.
- V2_QUALIFICATION_INCONCLUSIVE: not selected.

The decision follows the frozen structural aggregate: state utility at maximum length and state-reset causality both failed. No P1 patch, retune, or retry is authorized by this result.

## 42. V1 Default Preservation

V1 remains the canonical default. No roster constructor, role production file, public V1 state, V1 profile selection, or V1 checkpoint behavior defaults to V2. The V2 candidate is explicit and has not been promoted.

## 43. C2 Calibration Boundary

C2 stays deferred and non-canonical. Production calibration gain remains absent. The V2 raw NLL measurement was not calibrated, and C2 is not used to reinterpret the structural failure.

## 44. Delivery Freeze

Delivery artifacts, manifests, receipts, D2, parent evidence, and direct bindings were not modified or rerun. The delivery fingerprint guard passed.

## 45. Metal Freeze

Existing Metal source and topology remain unchanged. V2 has no Metal implementation and fail-closes non-CPU backend requests. The backend-metal feature check was compilation-only; no Metal hardware work ran.

## 46. Hardcoding Audit

| Check | Result |
| --- | --- |
| Q1 task answer lookup | 0 |
| length equals 32 special branch | 0 |
| family-name production branch | 0 |
| Q1 numeric result as production constant | 0 |
| C1 temperature use in production | 0 |
| C2 gain use | 0 |
| V1 checkpoint semantic reuse | 0 |
| dense state × state recurrence | 0 |
| V2 profile-flag pile | 0 |

## 47. Focused Verification

All Rust commands ran one at a time with one Cargo build job and a fresh target directory.

| Verification | Result |
| --- | --- |
| V2 revision, fixed state, zero reset, Metal fail-closed | Pass |
| V1 sentinel and V2 bounded state/gate/mode contracts | Pass |
| V2 representative finite-difference gradients and training sanity | Pass |
| V2 Q1 fixture freeze integrity | Pass |
| V1 active math-profile sentinel | Pass |
| frozen V2 Q1 one-shot execution | Pass execution; V2_CORE_NOT_VIABLE result retained |
| production-prefix guard | Pass |
| role-boundary guard | Pass |
| delivery/fingerprint guard | Pass |
| cargo fmt --all -- --check | Pass |
| cargo check --offline --lib | Pass |
| cargo check --offline --lib --features backend-metal | Pass |
| git diff --check | Pass |

## 48. Explicitly Not Run

- Full global suite
- D2
- Metal hardware
- calibration
- self-learning
- Formula Lab
- Investor Constitution
- Chair
- market/live trading
- internet learning

## 49. Warning Audit

The V2 delta introduces zero warnings. The final library checks reproduce four existing unrelated dead-code warnings. Test compilation reproduces nine existing warnings. No warning was suppressed, reclassified, or hidden.

## 50. Status Separation

- V1: CORE_NOT_VIABLE; default preserved.
- V2-P1 implementation: READY_FOR_INDEPENDENT_REVIEW.
- V2 Core Viability: V2_CORE_NOT_VIABLE.
- V2 Confidence: raw uncalibrated measurement only; not independently classified.
- Q1: frozen protocol preserved and executed once.
- C2: DEFERRED / NON-CANONICAL.
- Delivery: FROZEN.
- Metal: FROZEN.
- Overall: READY_FOR_INDEPENDENT_REVIEW.

## 51. What This Proves

It proves that a minimal, distinct V2 candidate with fixed state, no previous_u, state/input-conditioned interpolation, additive local-plus-memory output, manual backward, deterministic CPU modes, and fail-closed Metal boundary can be implemented without changing V1. It also proves that this exact candidate was qualified once against the unchanged frozen Q1 and did not meet the required structural gates.

## 52. What This Does Not Prove

It does not prove that V1 is viable, that a later V2 revision will work, that a different gate or state equation is correct, that calibration would repair structural failure, that V2 is production-ready, or that delivery, Metal hardware, trading, self-learning, Formula Lab, investor governance, or internet learning is valid.

## 53. Final Implementation Status

READY_FOR_INDEPENDENT_REVIEW

The implementation status records complete, honest implementation and one-shot qualification. The separate V2 core verdict is V2_CORE_NOT_VIABLE.

## 54. Exactly One Next Step

- independent V2 implementation + frozen-Q1 review
