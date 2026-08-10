# SOMA Sprint 105-LD1 M3-Micro Lineage Retirement Decision

## 1. Mode and Git State

Mode: `LINEAGE_RETIREMENT_DECISION_ONLY`.

This phase adds this decision record only. It makes no production-source or test-source change, runs no Rust/Cargo command, and does not rerun qualification. The working tree contains the already-qualified V2-P1 implementation from its preceding phase; no staged change is present. `git diff --check` passes.

## 2. Decision Goal

Determine whether the M3-Micro core lineage should continue patching, retire while preserving useful system contracts, or retire all system work. The target is accepted only if the recorded V1/V2 evidence and present source boundaries agree.

## 3. Q1 V1 Authority

The frozen V1 Q1 record is authoritative: `CORE_NOT_VIABLE`. Delayed Cue utility passed, while Order Sensitive, maximum-length Interference Retention, Local Control, and overall reset causality failed. Fixed footprint, determinism, CPU mode equivalence, and numerical stability passed.

## 4. V2-P1 Authority

The frozen V2-P1 record is authoritative: `V2_CORE_NOT_VIABLE`. Its distinct implementation passed revision isolation, fixed state, bounded interpolation, local-preserving additive output, manual backward, representative gradient integrity, training-loss decrease, determinism, CPU mode equivalence, and non-CPU fail-close.

## 5. V1 Qualification Summary

V1 exhibits partial memory utility: Delayed Cue Base exceeds No-State and reset. It does not establish general order sensitivity, maximum-length interference retention, local preservation, or all-history reset causality. It is therefore frozen non-viable experimental evidence, not a fallback core.

## 6. V2 Qualification Summary

V2 repairs the V1 Local Control comparison but has no frozen history-family separation: Delayed Cue, Order Sensitive, and Interference Retention each record Base / No-State / Reset of `0.5 / 0.5 / 0.5` at lengths 8, 16, and 32. Required state utility and reset causality fail.

## 7. Complementary Failure Pattern

| Property | V1 | V2-P1 |
| --- | --- | --- |
| Memory utility | PARTIAL | FAIL |
| Local preservation | FAIL | PASS |
| General Common-Brain qualification | FAIL | FAIL |

This is complementary failure evidence, not a license to average, merge, or layer the two equations. Combining the V1 memory path with the V2 local path would be unqualified architecture mashup and patch accumulation.

## 8. Implementation-Defect Separation

V2's non-viability is not explained by compilation failure, NaN/Inf, missing backward, representative gradient mismatch, state growth, mode mismatch, benchmark drift, optimizer unfairness, or Local Control failure. The qualified implementation is mechanically sound under the protocol; the result is an architecture qualification failure.

## 9. Calibration Separation

C1 remains diagnostic only. C2 remains `CHANGES_REQUESTED`, deferred, non-canonical, and absent from production. Confidence calibration cannot create structural memory utility or reset causality, so it is not a lineage-rescue argument.

## 10. Training-Budget Separation

The frozen training budget is part of the Common-Brain trainability qualification. V2's history tasks at `0.5` under that budget cannot be dismissed as a request for more training. Any different successor budget requires separately authorized protocol revision; none is made here.

## 11. Candidate A — Continue M3-Micro Patching

**REJECT.** A third M3-Micro patch has no single verified common defect to repair. The proposed reasons of adding a gate, changing activation, changing mean readout, or increasing budget are hypotheses rather than evidence-backed common defects. V2-P2 and M3-Micro V3 continuation are therefore out of scope.

## 12. Candidate B — Retire Core Lineage, Preserve Contracts

**SELECT.** This exactly matches the evidence: retire the non-viable V1/V2 recurrent-core lineage while retaining the system contracts that made the failure detectable, reproducible, and honestly recorded.

## 13. Candidate C — Retire Entire System Work

**REJECT.** There is no direct evidence against the outer shell. Frozen Q1 detected real failures, and the verification infrastructure preserved them without converting them into a false pass. Those are reasons to preserve the system contracts, not discard them.

## 14. Candidate Comparison

| Candidate | Decision | Reason |
| --- | --- | --- |
| A. Continue M3-Micro patch lineage | REJECT | no verified common defect; would be speculative patch accumulation |
| B. Retire core lineage, preserve contracts | SELECT | aligns with both frozen core failures and validated outer contracts |
| C. Retire all system work | REJECT | no evidence that Q1 or verification infrastructure is defective |

## 15. Lineage Decision

`RETIRE_M3_MICRO_CORE_LINEAGE_PRESERVE_SYSTEM_CONTRACTS`

This is a PM decision derived from the audited evidence, not a claim of new implementation authority. It terminates M3-Micro as an active Common-Brain core candidate.

## 16. Retirement Scope

The following are retired from active Common-Brain continuation:

- M3-Micro V1 internal recurrent core;
- M3-Micro V2-P1 internal core;
- V1/V2 recurrence, fusion, readout, state, and checkpoint semantics;
- M3-Micro V2-P2 continuation; and
- treating M3-Micro as the next active Common-Brain candidate.

Retirement does not delete code or evidence.

## 17. Preservation Scope

Preserve V1/V2 source as historical evidence; C1/C2/Q1/RD1/V2 reports; frozen Q1; state/no-state/reset comparison; fixed-state, determinism, mode-equivalence, numerical-stability, instance-ownership, and role-boundary requirements; Delivery and Metal verification infrastructure; Candidate Manifest; Clean/Actual authority; and Direct Binding.

## 18. V1 Evidence State

`FROZEN_NON_VIABLE_EXPERIMENTAL_EVIDENCE`

V1 is neither a deprecated production model nor a temporary viable baseline. Its source remains retained for reproducibility and future failure-regression comparison.

## 19. V2 Evidence State

`FROZEN_NON_VIABLE_EXPERIMENTAL_EVIDENCE`

V2-P1 remains a distinct, mechanically qualified experiment whose frozen structural result is non-viable. It is not a production candidate, calibration-pending baseline, or silent CPU/Metal fallback.

## 20. No-Deletion Evidence Policy

No V1/V2 source, state, checkpoint, report, or qualification record is deleted or moved. Retention supports reproducibility, successor comparison, and protection against recreating known failures.

## 21. Successor Naming Boundary

Do not name a successor M3-Micro V3, Mamba4 Micro, M3 Enhanced, or M3 Hybrid. Until a separately authorized naming decision, use only `SOMA_SUCCESSOR_COMMON_CORE` as a temporary document label.

## 22. Successor Fixed-State Contract

The successor must use fixed-size persistent state with no sequence-length-growing hidden history, fixed resource cost, explicit state auditing, finite-state validation, deterministic execution, and exact Full/Streaming/Chunked equivalence.

## 23. Successor Order-Sensitivity Contract

The successor must express general order sensitivity and carry order information to raw output under the frozen Q1 protocol. It must have enough auditable structured interaction for order and interference without automatically choosing diagonal-only recurrence or a dense state-by-state matrix.

## 24. Successor Structured-Memory-Readout Contract

The successor must not assume early collapse of all state into one mean/scalar. Multiple state channels must remain distinguishable to output through a fixed-cost-with-sequence-length readout. No projection equation is selected in this phase.

## 25. Successor Selective-Retention Contract

The successor must provide selective write/retain capacity and bounded resource cost, with explicit auditability. The exact gate or retention equation is deliberately undecided.

## 26. Successor Local/Memory Observability Contract

At the successor output boundary, local and memory signals must both remain observable; neither may silently erase the other. Composition must be task-independent. No implementation or fusion equation is chosen here.

## 27. Successor Learning-Credit Contract

Before Q1, successor prequalification must show that history-dependent loss reaches state-transition parameters, learned state contribution affects raw outputs, and Base separates from No-State during training. Thresholds and tests require a later authorized design phase; none are invented here.

## 28. Common-Brain / Independent-Ownership Contract

The successor must use one task-independent Common-Brain math family, preserve role boundaries, have instance-local parameters and state, and introduce no task/family/length special case, router, MoE, or expert system.

## 29. Architecture-Mashup Boundary

V1 and V2 failures must not be resolved by composing their recurrence, fusion, readout, activation, or checkpoint behavior. A successor is a neutral lineage with a separately designed and separately qualified core, not an accumulated M3-Micro patch.

## 30. Q1 Reuse Contract

The successor must reuse the same frozen Q1 task semantics, seed, sample count, development/evaluation split, reset intervention, metrics, and structural gates. Benchmark changes may not be used to fit a successor design.

## 31. Checkpoint Compatibility

`INTENTIONALLY_INCOMPATIBLE`

No V1/V2 weight, state, or checkpoint is semantically reinterpreted for a successor. No migration implementation is created in this phase.

## 32. Storage / Serialization Boundary

No JSON, Protobuf, or artifact schema is introduced. Future runtime storage decisions must be compact, fast, low-overhead, typed, and versioned only where necessary; this decision does not design them.

## 33. Delivery / Metal Preservation

Delivery and Metal remain frozen. Preserve their verification infrastructure and evidence boundaries, but make no delivery change, D2 work, Metal source change, shader/kernel work, hardware execution, receipt generation, or fallback claim.

## 34. Hardcoding Audit

| Audit item | Result |
| --- | --- |
| V1/V2 Q1 result used as successor behavior | absent |
| length-specific shortcut | absent |
| task-family shortcut | absent |
| PM decision stored as production constant | absent |
| current revision metadata used as decision condition | absent |
| retirement production enum | absent |
| production source change in this phase | 0 |
| test source change in this phase | 0 |

## 35. Explicitly Not Run

- Rust/Cargo commands and Q1 rerun
- V1/V2 causal probes or diagnostics
- V2-P2, V3, successor implementation, equations, training, or checkpoint work
- calibration, self-learning, Formula Lab, Investor Constitution, Chair, market, trading, or internet learning
- delivery, D2, Metal changes, or Metal hardware

## 36. Status Separation

- V1: `CORE_NOT_VIABLE`; frozen experimental evidence.
- V2-P1: `V2_CORE_NOT_VIABLE`; frozen experimental evidence.
- M3-Micro Lineage: `RETIRED` from active Common-Brain candidacy.
- Frozen Q1: preserved, authoritative, and unchanged.
- C2: `CHANGES_REQUESTED`; deferred and non-canonical.
- Delivery: `FROZEN`.
- Metal: `FROZEN`.
- Successor Core: no equation, implementation, training, or name authorized.
- Overall: `READY_FOR_INDEPENDENT_REVIEW`.

## 37. What This Proves

It proves that the M3-Micro V1/V2 core lineage should stop active continuation while the contracts and evidence that exposed the failures remain valuable. It preserves a clear neutral boundary for a future Common-Brain design without claiming a successor has been designed or qualified.

## 38. What This Does Not Prove

It does not prove a successor mechanism, readout, gate, activation, budget, checkpoint, storage format, production readiness, calibration remedy, Delivery result, Metal runtime result, market behavior, or trading validity. It does not authorize code deletion or a new core implementation.

## 39. Final Status

`READY_FOR_INDEPENDENT_REVIEW`

The decision is internally consistent with the recorded V1/V2 qualification evidence and the audited source boundaries.

## 40. Exactly One Next Step

- independent M3-Micro lineage retirement review
