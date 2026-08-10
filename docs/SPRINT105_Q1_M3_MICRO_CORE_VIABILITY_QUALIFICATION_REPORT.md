# SOMA Sprint 105-Q1 M3-Micro Core Viability Qualification Report

## 1. Mode and Git State

`CORE_VIABILITY_QUALIFICATION_ONLY`로 수행했다. 브랜치는 `agent/sprint104-r1-m3-micro-functional-conformance-v1`이며, 스테이지된 변경은 없다. 변경된 소스는 `src/model/m3_micro.rs`의 기존 `#[cfg(test)]` 모듈 내부 검증 코드뿐이다.

## 2. Strategic Goal

현재 M3-Micro가 Common Brain baseline으로 계속 개발할 만한 core state capability를 보이는지, 학습·공식·프로덕션 변경 없이 판정했다.

## 3. C1 / C2 Status Separation

```text
C1: CalibrationDominant, secondary length-dependent readout-margin compression
C2 calibration work: DEFERRED
production gain: ABSENT
production calibration: ABSENT
Length32 canonical confidence blocker: REMAINS
```

C2의 print-only 관찰은 Q1 통과 근거로 사용하지 않았다.

## 4. Progress Ledger

Q1 test-only harness, 네 synthetic family, 공정한 Base/No-State 학습 비교, reset causality, deterministic replay, CPU mode 비교, 상태 footprint audit를 구현했다. 프로덕션 수정은 없다.

## 5. M3-Micro Core Call Graph

실제 경로는 다음과 같다.

```text
input row
  -> affine_tanh(w_embed, b_embed) input projection
  -> two recurrent blocks: u / gates / decay / M3MicroState update
  -> final hidden representation
  -> affine(w_head, b_head) raw readout
  -> TrendContinuation direction distribution / class decision
```

이 경로는 `forward_internal_with_profile`에 있으며, Q1은 public full forward, `stream_step`, chunked forward만 사용했다.

## 6. Persistent State Definition

Trend configuration은 `d_model=64`, `expansion=2`, `inner_dim=128`, `d_state=8`, `block_count=2`다. 각 block은 `values` 1,024개와 `previous_u` 128개를 갖는다. 따라서 전체 persistent float element는 2,304개이며, 런타임 `byte_size()`는 9,224 bytes였다.

## 7. Current Training Policy

기존 deterministic roster seed와 기존 balanced trainer를 재사용했다. 각 family/length는 development 4개, frozen evaluation 4개, balanced class, 고정 training budget 6으로 Base와 `NoRecurrentState`를 별도 학습했다. optimizer, learning rate, loss 의미는 기존 `train_balanced_model_v2`를 변경하지 않았다.

## 8. Trainer / Architecture Separation

모든 12 Base run에서 final development loss가 initial loss보다 낮았다. 따라서 Q1의 trainability-inconclusive 조건은 발생하지 않았다. 아래의 core-gate 실패는 confidence overlay나 학습 손실 미하락으로 치환하지 않았다.

## 9. Qualification Data Design

테스트 메모리 안에서만 deterministic synthetic sequence를 만들었다.

- Delayed Cue: 첫 cue를 마지막 공통 query에서 회상한다.
- Order Sensitive: 첫 두 token의 순서가 class를 결정하고 이후 suffix/query는 공통이다.
- Interference Retention: 첫 cue 뒤의 강한 label-independent interference를 견딘다.
- State-Irrelevant Control: 마지막 local cue만 class를 결정한다.

## 10. Development / Evaluation Separation

Development variant는 11·23, frozen evaluation variant는 101·113이다. family/length마다 identity 집합의 교집합은 0이고 각 split은 positive 2개, negative 2개다.

## 11. Base / No-State Fairness

각 비교는 동일 initial model, dataset identity, training policy identity, optimizer digest를 사용했다. No-State와 Base의 parameter difference는 정확히 state-capability path 6,144개이고 unexpected/missing path는 모두 0이었다.

## 12. Delayed Cue

Base accuracy는 8/16/32에서 모두 1.0, No-State는 모두 0.5였다. reset 후 Base는 모두 0.5로 하락했다. 이 family에서는 state utility와 reset causality가 확인됐다.

## 13. Order Sensitive

Base와 No-State accuracy는 8/16/32 모두 0.5였다. state reset도 0.5였다. 순서 정보에 대해 strict state utility나 causality를 보이지 못했다.

## 14. Interference Retention

길이 8은 Base 1.0, No-State/reset 0.5였지만, 길이 16과 32는 Base/No-State/reset 모두 0.5였다. 최대 길이 retention gate는 실패다.

## 15. State-Irrelevant Control

Base accuracy는 8/16/32에서 모두 0.5였고 No-State는 모두 1.0이었다. local control의 `Base >= No-State` 조건을 충족하지 못했다.

## 16. State-Reset Causality

Delayed Cue는 모든 length에서 Base 1.0 > reset 0.5였다. Interference Retention은 length 8에서만 1.0 > 0.5였고, Order Sensitive와 나머지 history cases는 strict inequality가 성립하지 않았다. 전체 causality gate는 FAIL이다.

## 17. Length-Retention Matrix

Accuracy `Base / No-State`:

| Family | 8 | 16 | 32 | Max-length strict utility |
| --- | ---: | ---: | ---: | --- |
| Delayed Cue | 1.0 / 0.5 | 1.0 / 0.5 | 1.0 / 0.5 | PASS |
| Order Sensitive | 0.5 / 0.5 | 0.5 / 0.5 | 0.5 / 0.5 | FAIL |
| Interference Retention | 1.0 / 0.5 | 0.5 / 0.5 | 0.5 / 0.5 | FAIL |
| State-Irrelevant Control | 0.5 / 1.0 | 0.5 / 1.0 | 0.5 / 1.0 | FAIL (control) |

## 18. Qualification Matrix

`NLL` cell은 `categorical NLL / mean P(true)`이고, `Mean Margin` cell은 `mean / median correct-class logit margin`이다.

| Family | Length | Path | Samples | Accuracy | NLL | Mean Margin | Finite |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- |
| Delayed Cue | 8 | Base | 4 | 1.0 | 0.4245 / 0.6550 | 1.2191 / 1.2187 | true |
| Delayed Cue | 8 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Delayed Cue | 8 | State Reset | 4 | 0.5 | 0.9319 / 0.4013 | 0.0000 / 0.0000 | true |
| Delayed Cue | 16 | Base | 4 | 1.0 | 0.7067 / 0.4938 | 0.1724 / 0.1675 | true |
| Delayed Cue | 16 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Delayed Cue | 16 | State Reset | 4 | 0.5 | 0.7853 / 0.4562 | 0.0000 / 0.0000 | true |
| Delayed Cue | 32 | Base | 4 | 1.0 | 0.7437 / 0.4755 | 0.0900 / 0.0900 | true |
| Delayed Cue | 32 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Delayed Cue | 32 | State Reset | 4 | 0.5 | 0.7917 / 0.4534 | 0.0000 / 0.0000 | true |
| Order Sensitive | 8 | Base | 4 | 0.5 | 0.7660 / 0.4651 | 0.0000 / 0.0000 | true |
| Order Sensitive | 8 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Order Sensitive | 8 | State Reset | 4 | 0.5 | 0.7660 / 0.4651 | 0.0000 / 0.0000 | true |
| Order Sensitive | 16 | Base | 4 | 0.5 | 0.7781 / 0.4602 | 0.0017 / 0.0035 | true |
| Order Sensitive | 16 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Order Sensitive | 16 | State Reset | 4 | 0.5 | 0.7791 / 0.4597 | 0.0000 / 0.0000 | true |
| Order Sensitive | 32 | Base | 4 | 0.5 | 0.7671 / 0.4644 | 0.0000 / 0.0000 | true |
| Order Sensitive | 32 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Order Sensitive | 32 | State Reset | 4 | 0.5 | 0.7671 / 0.4644 | 0.0000 / 0.0000 | true |
| Interference Retention | 8 | Base | 4 | 1.0 | 0.5784 / 0.5617 | 0.7866 / 0.8613 | true |
| Interference Retention | 8 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Interference Retention | 8 | State Reset | 4 | 0.5 | 0.9107 / 0.4057 | 0.0000 / 0.0000 | true |
| Interference Retention | 16 | Base | 4 | 0.5 | 0.7641 / 0.4683 | 0.1257 / 0.1373 | true |
| Interference Retention | 16 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Interference Retention | 16 | State Reset | 4 | 0.5 | 0.8307 / 0.4394 | 0.0000 / 0.0000 | true |
| Interference Retention | 32 | Base | 4 | 0.5 | 0.7762 / 0.4617 | 0.1034 / 0.1112 | true |
| Interference Retention | 32 | No-State | 4 | 0.5 | 0.7788 / 0.4590 | 0.0000 / 0.0000 | true |
| Interference Retention | 32 | State Reset | 4 | 0.5 | 0.8221 / 0.4413 | 0.0000 / 0.0000 | true |
| State-Irrelevant Control | 8 | Base | 4 | 0.5 | 0.7716 / 0.4631 | 0.0159 / 0.0140 | true |
| State-Irrelevant Control | 8 | No-State | 4 | 1.0 | 0.3881 / 0.6784 | 1.2565 / 1.2565 | true |
| State-Irrelevant Control | 16 | Base | 4 | 0.5 | 0.7734 / 0.4621 | 0.0000 / 0.0000 | true |
| State-Irrelevant Control | 16 | No-State | 4 | 1.0 | 0.3881 / 0.6784 | 1.2565 / 1.2565 | true |
| State-Irrelevant Control | 32 | Base | 4 | 0.5 | 0.7904 / 0.4563 | -0.0001 / 0.0000 | true |
| State-Irrelevant Control | 32 | No-State | 4 | 1.0 | 0.3881 / 0.6784 | 1.2565 / 1.2565 | true |

NLL은 별도 confidence/calibration 관찰값이며 core viability gate 자체를 대체하지 않는다.

## 19. Persistent-State Footprint

| Length | Elements | Bytes |
| ---: | ---: | ---: |
| 8 | 2,304 | 9,224 |
| 16 | 2,304 | 9,224 |
| 32 | 2,304 | 9,224 |

## 20. Hidden-History Buffer Audit

`M3MicroState::zero`는 고정 길이 `values`와 `previous_u`만 만든다. token update는 이 배열을 `clone_from`으로 교체하고 `step_index`만 증가시킨다. Q1은 full path에서 allocation count 1, final `step_index == sequence length`, reset path에서 `step_index == suffix length`를 확인했다. 성장하는 history buffer는 발견하지 못했다.

## 21. State Magnitude

모든 run은 finite이고 `state.validate()`를 통과했다. Base final-state L2는 length와 함께 증가했다(예: Delayed Cue 103.58, 241.65, 458.64; Order 100.06, 242.62, 483.01). 이는 관찰값이며 독립 FAIL gate가 아니다.

## 22. Determinism

동일 seed와 fixture로 전체 Q1 evidence를 한 test 안에서 두 번 생성해 완전 동치(`first == second`)를 확인했다. 최종 재실행도 동일한 모든 metric과 gate를 재현했다.

## 23. Execution-Mode Equivalence

각 family/length의 trained Base model에 대해 CPU full-sequence, token streaming, 3-token chunked forward의 raw output과 final state가 완전 동치였다. 12/12 PASS다.

## 24. Numerical Stability

24 Base/No-State rows와 9 reset rows 모두 raw output, class distribution, state value, NLL, probability, margin이 finite였고 state validation을 통과했다.

## 25. Runtime Efficiency

진단 전용이다. 한 Q1 complete deterministic qualification은 115.40초였고, path별 evaluation token 수는 length 8/16/32에서 각각 32/64/128이다. 성능 threshold나 production SLO 판정에는 사용하지 않았다.

## 26. Core Gate Matrix

| Gate | Result | Evidence |
| --- | --- | --- |
| Development/evaluation separation | PASS | identity overlap 0 |
| Base/No-State fairness | PASS | identical data/policy/optimizer; 6,144 expected paths only |
| Trainability sanity | PASS | 12/12 final development loss decreased |
| History utility at length 32 | FAIL | Order and Interference: Base 0.5 = No-State 0.5 |
| State-reset causality | FAIL | no strict Base > reset for all history cases |
| Local control | FAIL | Base 0.5 < No-State 1.0 at all lengths |
| Constant state footprint | PASS | 2,304 elements / 9,224 bytes at every length |
| Hidden-history absence | PASS | fixed vectors, one allocation, checked step index |
| Determinism | PASS | two equal evidence runs |
| CPU execution-mode equivalence | PASS | 12/12 exact output/state matches |
| Numerical stability | PASS | all reported paths finite |

## 27. Confidence Overlay

C1의 canonical Length-32 confidence blocker는 유지된다. Q1의 core-gate FAIL은 calibration change를 제안하거나 C2 observation으로 완화하지 않는다.

## 28. Production Immutability

모델, recurrence, state equation, readout, calibration, loss, roles, learning, formula, G4/G1/G2/G3, delivery logic를 변경하지 않았다. Q1 additions are entirely below the existing test-module boundary.

## 29. Delivery Freeze

Representative delivery-fingerprint guard `m3_micro_d1r1e_test_only_and_delivery_test_mutations_do_not_change_fingerprint` passed. Delivery artifact generation과 publication은 수행하지 않았다.

## 30. Metal Freeze

Metal hardware execution, dispatch, benchmark, receipt generation은 수행하지 않았다. `backend-metal` library type-check만 PASS했다.

## 31. Storage / Artifact Audit

Q1 persistent fixture, cache, checkpoint, receipt, dataset artifact는 만들지 않았다. 이 문서가 Q1의 유일한 report artifact다.

## 32. Hardcoding Audit

길이는 기존 policy의 8/16/32만 사용했다. split variant와 synthetic tokens는 test-only deterministic fixture identity이며 production inference, label, threshold, or length-special-case code에는 추가되지 않았다.

## 33. Focused Verification

- `cargo fmt --all -- --check`: PASS
- `git diff --check`: PASS
- Q1 fixture-integrity test: PASS
- Q1 core-viability qualification test: PASS (115.40s)
- Existing C1 length-32 confidence diagnostic: PASS
- Production-prefix guard: PASS
- Role-boundary guard: PASS
- Delivery-freeze representative guard: PASS
- `cargo test --offline --lib --no-run`: PASS
- `cargo check --offline --lib`: PASS
- `cargo check --offline --lib --features backend-metal`: PASS

## 34. Explicitly Not Run

Full suite, full Metal suite, Metal hardware execution, delivery/receipt generators, D2, calibration fix, Formula Lab, self-learning, internet/broker/live integrations were not run.

## 35. Warning Audit

Q1 code introduced no compiler warning. Library checks reproduced four existing dead-code warnings in persona-card/learning-campaign code. Test compilation reported nine existing test-build dead-code warnings, including optional Metal helper paths; none point to the Q1 additions.

## 36. Core Viability Decision

`NOT_VIABLE`.

The Q1 definition requires an actual architecture-gate FAIL to produce this result. State utility at maximum length, reset causality across history families, and state-irrelevant local control all fail, while trainability and numerical checks are conclusive.

## 37. What This Proves

Under the current deterministic trainer and fixed synthetic protocol, M3-Micro has usable state behavior for Delayed Cue but does not meet the required general core-state qualification across order, interference, and local control.

## 38. What This Does Not Prove

It does not prove a production-market outcome, a calibration remedy, an optimal hyperparameter search, a Metal runtime result, or that a future architecture redesign cannot qualify. It does not alter C1/C2 status.

## 39. Final Implementation Status

Complete: test-only Q1 qualification harness and its single report are present; production behavior is unchanged; no staging, commit, push, delivery, or follow-up fix was performed.

## 40. Exactly One Next Step

- Conduct an independent core-qualification review of the recorded `NOT_VIABLE` result before authorizing any learning, formula, calibration, or architecture change.
