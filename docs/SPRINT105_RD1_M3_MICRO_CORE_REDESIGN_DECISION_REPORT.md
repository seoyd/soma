# SOMA Sprint 105-RD1 M3-Micro Core Redesign Decision

## 1. Mode and Git State

`CORE_REDESIGN_DECISION_ONLY`로 수행했다. 스테이지된 변경은 없고, production prefix에는 변경이 없다. source change는 기존 `#[cfg(test)]` module의 RD1 private causal-audit probe뿐이며, 이 문서가 RD1의 유일한 report artifact다.

## 2. Strategic Decision Goal

Q1의 `CORE_NOT_VIABLE`를 authority로 삼아 현재 M3-Micro에서 보존할 외곽 contract와 교체할 internal core 책임을 결정했다. production V2, equation, state update, calibration, loss, delivery, Metal 변경은 수행하지 않았다.

## 3. Q1 CORE_NOT_VIABLE Authority

Q1은 fixed footprint, hidden-history absence, determinism, CPU full/streaming/chunked equivalence, numerical stability, trainability, Delayed Cue utility를 PASS로 기록했다. 반면 Order, max-length Interference, overall reset causality, Local Control은 FAIL이었다. RD1은 이 판정을 재해석하거나 완화하지 않는다.

## 4. C1 / C2 Separation

C1의 `CalibrationDominant` diagnostic은 유지한다. C2는 `CHANGES_REQUESTED`, production gain/calibration은 absent이며 deferred/non-canonical이다. RD1 decision과 probe에 C2 gain이나 calibration을 사용하지 않았다.

## 5. Sprint 104 Freeze

Delivery verification infrastructure, manifests, checkpoint/provenance authority, direct binding, Metal topology는 재개방하지 않았다. D2와 Metal hardware replay도 실행하지 않았다.

## 6. Current Core Equation

각 block에서 current hidden `h`와 input `x_t`에 대해 실제 구현은 다음 의미를 갖는다.

```text
e_t = tanh(W_embed x_t + b_embed)
u_t = tanh(W_in h_t + b_in)
p_t = sigmoid(W_prev u_t + b_prev)
c_t = sigmoid(W_curr u_t + b_curr)
d_t = bounded_decay(u_t, c_t, input_activity)
s_t[i] = d_t[i] s_(t-1)[i]
       + p_t[channel(i)] previous_u_(t-1)[channel(i)] prev_scale[i]
       + c_t[channel(i)] u_t[channel(i)] curr_scale[i]
r_t[channel] = sum_i s_t[i] readout_scale[i] / d_state
z_t = softsign(r_t + sigmoid(skip) * u_t)
h_(t+1) = softsign(W_out z_t + b_out)
raw_output = W_head h_final + b_head
```

`previous_u`와 `s` 모두 token마다 갱신된다. 따라서 `F(F(s, A), B)`와 `F(F(s, B), A)`는 구조적으로 동일한 식이 아니며, commutativity를 가정하지 않았다.

## 7. Current Core Component Map

```text
input projection -> per-block u/gates/decay -> fixed M3MicroState
                 -> state readout + local skip-u additive fusion -> block output
                 -> final linear head
```

현재 local `u`는 존재하지만 state readout과 같은 `z`에서 softsign 이전에 합쳐진다. final head로 가는 별도 post-fusion local bypass는 없다.

## 8. Q1 Passed Properties

`FIXED_STATE_FOOTPRINT`, `NO_FULL_HISTORY_BUFFER`, deterministic execution, exact CPU mode equivalence, finite state/output, trainability sanity, Delayed Cue utility를 V2 preservation requirements로 채택했다.

## 9. Q1 Failed Properties

Order Sensitive는 Base/No-State가 length 8/16/32에서 모두 0.5였다. Interference는 length 8에서만 utility가 있었고 length 16/32는 0.5/0.5였다. Local Control은 Base 0.5, separately-trained No-State 1.0이었다.

## 10. Order-Sensitivity Algebra Audit

Order dependence는 current state equation에 존재한다. `u_t`, gate, decay가 current hidden에 의존하고 `previous_u`와 `s_(t-1)`가 다음 update에 직접 들어가므로 A/B 순서를 교환하면 state transition input과 cross-token term이 바뀐다. 따라서 Q1 Order failure는 equation의 exact commutativity로 설명되지 않는다.

## 11. Order Causal Probe

공통 query 뒤 A→B와 B→A를 비교했다.

| Path | State difference L2 | Final hidden difference L2 | Raw-logit difference L2 |
| --- | ---: | ---: | ---: |
| Untrained | 8.2541 | 6.2904 | 0.0779 |
| Q1-trained | 9.6709 | 0.3156 | 0.0150 |

모든 값은 finite다. Trained state는 명확히 분리되지만 final hidden/logit 전달 차이는 크게 압축되며, Q1 task accuracy는 여전히 0.5다.

## 12. Order Failure Location

`READOUT_ORDER_BLINDNESS`.

이 명칭은 exact zero difference가 아니라 operational classification blindness를 뜻한다. State order separation은 있으므로 `STATE_ORDER_COLLAPSE`가 primary evidence와 맞지 않는다. 반면 trained state difference 9.6709가 final hidden 0.3156과 logit 0.0150으로 축소되어 common query classification에 이용되지 못했다.

## 13. Interference State Trace

Length-32 positive cue trace에서 actual distractor path의 state L2는 cue 직후 6.2057에서 query 직전 431.2779로 증가했고 cue-state cosine은 1.0000에서 0.4574로 감소했다. 같은 cue/query를 유지한 neutral zero-input control은 final L2 471.7456, cosine 0.4957이었다.

## 14. Interference Causal Probe

Actual distractor와 neutral control의 final state L2 distance는 116.5910이었다. Actual final positive raw margin은 0.2786, neutral은 0.2838이며 actual cue cosine은 neutral보다 낮다. 중립 input도 retention loss를 보이지만, actual distractor는 state trajectory를 추가로 크게 바꾼다.

## 15. Interference Failure Location

`MIXED` — `STATE_OVERWRITE + RETENTION_DECAY`.

모든 token이 same update path를 통과하고, input-conditioned strong/weak update와 explicit retain decision이 분리되어 있지 않다. Trace는 cue alignment의 지속적 감쇠와 distractor-specific final-state displacement를 모두 보인다. 이 probe는 특정 state coordinate가 cue를 저장한다는 주장은 하지 않는다.

## 16. Local-Control Representation Decomposition

Final local input은 input projection과 first-block `u`까지는 state와 독립적으로 도달한다. 그러나 block output은 `state readout + skip(u)`의 additive fusion, softsign, output projection을 거친다. 그러므로 local current signal은 존재하되 final decision으로 가는 독립 bypass가 없다.

## 17. Local-Control Counterfactual

동일 trained Base weights와 동일 frozen Local-Control evaluation에서 비교했다.

| Path | Accuracy | NLL | Mean P(true) | Mean correct-class margin |
| --- | ---: | ---: | ---: | ---: |
| Normal | 0.5 | 0.7904 | 0.4563 | -0.0001 |
| Reset immediately before final local input | 1.0 | 0.8458 | 0.4293 | 0.2928 |
| Same weights, test-only No-Recurrent-State | 1.0 | 0.8803 | 0.4149 | 0.2037 |

Representative final local input의 embedded difference와 first `u` difference는 각각 0이다. 그러나 final hidden difference는 8.3979, raw-logit difference는 1.2392다.

## 18. Local-Control Failure Location

`MIXED` — `STATE_DOMINATES_LOCAL_SIGNAL + LOCAL_BYPASS_ABSENT`.

Reset과 same-weight state removal이 1.0 accuracy를 회복하므로 local signal 자체가 사라진 것이 아니다. Source audit은 state/local fusion 뒤 별도 local path가 없음을 보이며, state contribution이 local classification을 덮는 causal evidence와 일치한다.

## 19. Shared Failure Mechanism

세 failure는 서로 독립된 task literal 문제가 아니다. Current core는 (1) 모든 input을 동일 recurrence update pipeline으로 넣고, (2) state readout과 local signal을 one-way additive fusion으로 섞고, (3) fused representation만 head에 넘긴다. 이 구조는 distractor displacement, order signal attenuation, local-signal domination을 각각 허용한다.

## 20. Redesign Requirements

V2는 fixed state, structural order sensitivity, input/state-conditioned selective update/retention, state causal carrier, local information preservation, no length special case를 만족해야 한다. 하나의 Common Brain equation, independent model/state ownership, no router/MoE/expert, no external architecture mashup도 유지한다.

## 21. Candidate A — Patch Current Core

`REJECTED`.

Local bypass patch만으로는 Order의 trained fusion/readout attenuation과 Interference의 update/retention failure를 해결하지 못한다. selective update patch만으로는 local dominance도 해결하지 못한다. 세 failure가 하나의 작은 isolated defect라는 evidence가 없다.

## 22. Candidate B — M3-Micro V2 Internal Rewrite

`SUPPORTED`.

M3-Micro shell, role ownership, fixed-state principle, execution mode contract, compact resource target은 Q1에서 좋은 특성으로 확인됐다. 반면 recurrent update, local/state fusion, internal readout handoff은 shared failure mechanism과 직접 연결되어 있어 함께 교체할 최소 internal scope다.

## 23. Candidate C — Retire M3-Micro

`WEAK`.

현재 public shell/API가 Order, Interference, Local failure의 원인이라는 evidence는 없다. B는 public outer contract를 갈아엎지 않고 failure-coupled internals만 교체할 수 있으므로, lineage retirement는 현재 근거보다 범위가 크다.

## 24. Candidate Comparison Matrix

| Criterion | A | B | C |
| --- | --- | --- | --- |
| Order failure coverage | WEAK | SUPPORTED | HIGH_RISK |
| Interference coverage | WEAK | SUPPORTED | HIGH_RISK |
| Local Control coverage | WEAK | SUPPORTED | HIGH_RISK |
| Fixed-state preservation | SUPPORTED | SUPPORTED | WEAK |
| Determinism/mode preservation | SUPPORTED | SUPPORTED | WEAK |
| Simplicity after decision | WEAK | SUPPORTED | WEAK |
| Implementation scope | SMALL but insufficient | BOUNDED internal | BROAD |
| Testability with Q1 reuse | WEAK | SUPPORTED | WEAK |
| Patch accumulation risk | HIGH_RISK | LOWER | HIGH_RISK |
| Architecture mashup risk | LOW | LOW | MEDIUM |
| Independent-AI ownership | SUPPORTED | SUPPORTED | REQUIRES new contract |
| Future self-learning suitability | WEAK | SUPPORTED after qualification | NOT_ESTABLISHED |

## 25. Current Component Salvage Map

| Component | KEEP/REWRITE/REMOVE | Reason |
| --- | --- | --- |
| Input projection | KEEP | current local representation is available deterministically |
| Fixed-size typed state ownership | KEEP | fixed footprint and no history buffer passed |
| Current decay/injection update | REWRITE | no selective retain/update separation; interference evidence |
| `previous_u` cross-token use | REWRITE | must participate in V2 order semantics under a coherent update contract |
| State-readout plus skip-`u` additive fusion | REMOVE | causally dominates local signal and attenuates order signal |
| Local/current representation handoff | REWRITE | local signal needs an independently preservable path |
| Internal readout handoff | REWRITE | trained order separation is compressed before class logits |
| Final role output boundary/head contract | KEEP | outer role contract is not failure-coupled evidence |
| Initialization determinism | KEEP | deterministic contract passed |
| State reset semantics | KEEP | reset remains the causal boundary |
| Full/stream/chunk wrappers | KEEP | exact mode equivalence passed |

## 26. Complexity Budget

V2 is limited to three internal responsibilities: one fixed persistent state, one selective update/retain decision, and one local/memory contribution decision. These are not routers, experts, separate models, length switches, or an imported architecture block. Exact equations are deliberately deferred to the implementation phase.

## 27. Fixed-State Contract

Persistent state shape is determined solely by model shape. Sequence length must not add state elements, buffers, or history storage. State remains native typed in-memory data; no serialization requirement is introduced.

## 28. Order-Sensitivity Contract

For suitable input pairs, A→B and B→A must be representable as different state semantics and remain observable at the common output boundary. This is a general transition property, not an Order-task branch.

## 29. Selective-Update / Retention Contract

Each input may cause strong incorporation, weak incorporation, or retention according to current input/state. Length is not an input to this decision. Label-independent distractors must not force unconditional overwrite.

## 30. Local-Information Preservation Contract

Current/local information must reach the final representation through a state-independent preservable contribution. Memory may influence, but cannot unconditionally erase, that contribution. No task-specific bypass is allowed.

## 31. Common-Brain Contract

Trend, Volatility, and Reversal use one V2 core equation. They retain independent parameters, model instances, and recurrent state; no role-specific brain equation, router, or expert selector is introduced.

## 32. Independent-AI Ownership Contract

Each agent continues to own its model parameters and fixed state. Streaming/chunking state is instance-local, deterministic, finite, and cannot carry data between independently owned models.

## 33. Checkpoint Compatibility Decision

`INTENTIONALLY_INCOMPATIBLE`.

A rewritten internal update/fusion equation makes current core weights semantically unsafe to reuse as V2 parameters. RD1 creates neither migration code nor a conversion promise.

## 34. Q1 Qualification Reuse Contract

V2 must reuse unchanged: Delayed Cue, Order Sensitive, Interference Retention, State-Irrelevant Control, State Reset Causality, state footprint, determinism, CPU mode equivalence, and numerical-stability qualification. V2 may not alter these tasks to fit its result.

## 35. Confidence / C2 Boundary

V2 records confidence/NLL separately after structural qualification. C1/C2 calibration is not carried into V2 automatically; production gain remains absent until a future independent decision.

## 36. Production Immutability

No model equation, state update, readout, calibration, loss, G4/G1/G2/G3, role behavior, learning behavior, dependency, delivery, or Metal source was changed. RD1 probes use only private test access.

## 37. Delivery / Metal Freeze

Production-prefix, role-boundary, and delivery-fingerprint guards passed. Metal was compile-checked only; no hardware execution, topology replay, receipt generation, or delivery change occurred.

## 38. Hardcoding Audit

No Q1/RD1 metric, task name, sequence length, seed, margin, or output literal affects production behavior. Synthetic A/B, neutral control, and test-only state ablation are confined to the existing test module.

## 39. Focused Causal Probes

| Stage | Status | Evidence | Finding |
| --- | --- | --- | --- |
| Starting Q1 verdict | VERIFIED | Q1 evidence | `CORE_NOT_VIABLE` |
| Sprint 104 freeze | PASS | diff/guard | no frozen-system change |
| Production model immutability | PASS | test-module-only diff | production prefix unchanged |
| Current equation audit | COMPLETE | source | update and fusion mapped |
| Order algebra audit | COMPLETE | source | non-commutative structure |
| Order state/hidden/logit separation | MEASURED | RD1 probe | state survives; output compresses |
| Order failure location | READOUT_ORDER_BLINDNESS | RD1 synthesis | trained task use absent |
| Interference trace | MEASURED | RD1 probe | cue cosine decays |
| Interference overwrite | MEASURED | RD1 neutral control | final state L2 delta 116.5910 |
| Interference failure location | MIXED | RD1 synthesis | overwrite plus retention decay |
| Local decomposition | COMPLETE | source/tape | no final independent bypass |
| Local reset counterfactual | MEASURED | RD1 probe | 0.5 -> 1.0 accuracy |
| Local failure location | MIXED | RD1 synthesis | state dominance plus bypass absence |
| Passed-property inventory | COMPLETE | Q1/RD1 | retained in V2 contract |
| Candidate A | REJECTED | matrix | insufficient common repair |
| Candidate B | SUPPORTED | matrix | bounded internal rewrite |
| Candidate C | WEAK | matrix | shell not failure-coupled |
| Salvage map/complexity | COMPLETE | design | three responsibilities only |
| Q1 qualification reuse | PASS | contract | unchanged tasks required |
| C2 deferred | PASS | boundary | no calibration carryover |
| Production V2 implementation | NOT_RUN | mode | forbidden in RD1 |
| Final redesign decision | ACTUAL | synthesis | `M3_MICRO_V2_INTERNAL_REWRITE` |

## 40. Compiler Verification

- `cargo fmt --all -- --check`: PASS
- `git diff --check`: PASS
- `cargo test --offline --lib --no-run` with fresh RD1 target: PASS
- `cargo check --offline --lib`: PASS
- `cargo check --offline --lib --features backend-metal`: PASS

All Cargo/Rust work was executed one SOMA process at a time.

## 41. Explicitly Not Run

Full suite, D2, Metal hardware, production calibration, production V2 implementation, self-learning, Formula Lab, Investor Constitution, live trading, internet learning, and generators were not run.

## 42. Warning Audit

RD1 introduced zero warnings. Test compilation reproduced nine existing dead-code warnings in pre-existing code. Both library checks reproduced four existing dead-code warnings in persona-card/learning-campaign code; none are attributable to RD1.

## 43. Redesign Decision

`M3_MICRO_V2_INTERNAL_REWRITE`.

## 44. Decision Rationale

The outer M3-Micro contract demonstrably preserves useful properties, so retirement is unsupported. The causal evidence connects multiple failures to the current internal update/fusion/readout path, so a minimal patch is insufficient. A bounded V2 internal rewrite is the smallest decision that addresses all observed structural failure locations without expanding the product contract.

## 45. What This Proves

It proves, under the current deterministic Q1/RD1 protocol, that order state is represented but poorly delivered to trained logits, distractors materially displace/decay state, and the current state fusion can suppress an otherwise available local signal. It establishes the preservation and acceptance contracts for a future V2 implementation.

## 46. What This Does Not Prove

It does not implement or validate V2, prove a market outcome, select final V2 equations, establish checkpoint migration, solve confidence calibration, or authorize learning/formula/delivery work.

## 47. Final Implementation Status

`READY_FOR_INDEPENDENT_REVIEW`.

RD1 test-only causal audit and this decision report are complete. Production implementation remains zero; no stage, commit, push, network, delivery, or Metal runtime work occurred.

## 48. Exactly One Next Step

- independent core redesign decision review
