# SOMA Sprint 105-C2 M3-Micro Positive Readout-Gain Calibration Report

## 1. Mode and Git State

Mode: `TARGETED_CAPABILITY_FIX`.

The worktree contains test-only C2 calibration-boundary and scalar-identifiability instrumentation in `src/model/m3_micro.rs`, plus this single C2 report. No production readout-gain field, state, recurrence, raw readout weight, role, delivery, Metal, receipt, or checkpoint change was made. There are no staged changes, commits, pushes, or network operations.

## 2. C1 Diagnostic Authority

The approved C1 starting diagnosis is `CalibrationDominant`, with length-dependent readout-margin compression retained as a secondary observation. C2 uses that diagnosis without turning its temperature sweep values into a production constant.

## 3. Starting Capability Status

`BlockedByLength32ConfidenceCapability`.

The current canonical Base path retains length-32 ranking accuracy but has categorical NLL 0.803510, higher than the unchanged No-State comparator's 0.766560.

## 4. Progress Ledger

| Stage | Status | Evidence | Blocker |
| --- | --- | --- | --- |
| Starting state | VERIFIED/BLOCKED | source and C1 regression | length-32 confidence capability |
| C1 review authority | VERIFIED | approved targeted-fix verdict | none |
| Calibration-data boundary | PASS | test-side fixture audit | none |
| Eval leakage | ABSENT | 12 calibration record identities vs 12 evaluation identities | none |
| Finite scalar identifiability | FAIL | derivative-limit audit | `BlockedByUnidentifiableCalibrationScale` |
| Positive gain | NOT_APPLIED | finite solution absent | same |
| Gain identity init | NOT_APPLICABLE | no gain adopted | same |
| Frozen core | PASS | source/diff audit | none |
| Raw readout unchanged | PASS | source/diff audit | none |
| Gain determinism | NOT_APPLICABLE | no finite fit exists | same |
| Length-specific branch | ABSENT | source audit | none |
| Evaluation update | ABSENT | source audit | none |
| Argmax invariance | NOT_APPLICABLE | no gain adopted | same |
| Canonical length curve | MEASURED | C1 focused regression | existing blocker remains |
| L32 confidence gate | `BlockedByLength32ConfidenceCapability` | existing canonical protected interaction | existing blocker remains |
| Capability Model | `BlockedByLength32ConfidenceCapability` | canonical gate | existing blocker remains |
| State regression | ABSENT | no calibration application | none |
| G4 | PRESERVED | diff audit | none |
| Roles | PRESERVED | diff and role guard | none |
| Delivery | FROZEN | diff and delivery guard | none |
| Metal | FROZEN | diff and feature check | none |
| fmt/check | PASS | sequential offline commands | none |
| New warning | 0 | compiler audit | none |
| Sprint105-C2 | `BlockedByUnidentifiableCalibrationScale` | finite-scale evidence | finite scalar solution absent |

## 5. Production Readout Call Graph

The existing core computes final head logits in `M3MicroModel::forward_internal`; the role prediction boundary consumes the categorical logits through the existing stable softmax and categorical NLL semantics. C2 did not alter this production call graph because scalar fitting did not yield an identifiable finite value.

## 6. Calibration Data Boundary

| Data/Fixture | Used For Base Training | Available For Calibration | Used For Capability Evaluation |
| --- | --- | --- | --- |
| Existing delayed-recall development examples (per canonical length) | yes | yes: post-training frozen-core raw logits and labels | no |
| Existing delayed-recall frozen examples (per canonical length) | no | no | yes |

Calibration source: raw categorical logits and labels from the existing development examples after the fixed Base training budget. Capability-evaluation source: existing frozen examples. The audit collected 12 calibration record identities and 12 capability-evaluation identities across lengths 8, 16, and 32; their intersection was 0.

## 7. Evaluation Leakage Audit

The fitting probe accepts only records derived from development examples. Frozen capability events are separately identified and asserted disjoint before objective analysis. No capability label, logit, NLL, temperature result, No-State result, or evaluation event is used to select a gain.

## 8. Calibration Objective

The proposed objective is the existing stable categorical negative log-likelihood on frozen-core development-side logits:

`mean(-ln(softmax(gain * logits)[true_class]))`.

No new loss, regularizer, label smoothing, or No-State target objective was introduced.

## 9. Scalar Identifiability

The scalar is not identifiable at a finite positive value on the independent calibration records.

* records: 12
* pre-fit NLL at identity gain: 0.645531
* derivative as gain approaches zero: -0.608784
* derivative at identity gain: -0.313671
* derivative as gain approaches infinity: 0.000000

Every calibration record has its true class at the maximum raw categorical logit, so categorical NLL continues to improve toward the infinite-scale boundary instead of crossing a finite stationary point. Selecting a finite gain would therefore require an arbitrary clamp, a regularizer, or evaluation-derived choice, all of which are outside C2.

## 10. Gain Representation

No `log_gain` or runtime gain was stored. Identity initialization and positive exponential conversion are deliberately not implemented because a production parameter may be adopted only after a finite identifiable fit.

## 11. Identity Initialization

Not applicable: no production gain was created. Existing model output is unchanged, so the effective behavior remains the pre-C2 identity path.

## 12. Frozen-Core Contract

The C2 probe trains the existing Base core exactly as the established capability harness does, then observes its development-side logits without performing any calibration update. During C2 there are zero changes to existing core parameters, raw readout parameters, optimizer state, or recurrent state.

## 13. Fitting Method

No fitter is run after the identifiability audit. The test computes the analytical first derivative of the existing categorical NLL with respect to positive gain and its zero/infinite limits. Because the infinite limit is the non-negative boundary value while the finite derivative remains negative, there is no finite stationary solution to adopt.

## 14. Fitting Result

* records: 12
* pre NLL: 0.645531
* post NLL: not applicable; no finite fit
* gain: not adopted
* finite: no
* identifiable: no

## 15. Parameter Ownership

No mutable global state was added. No model-instance gain was added because no valid scalar value exists to own. The existing model-instance ownership boundaries are unchanged.

## 16. Persistence Boundary

The existing checkpoint payload already persists a model instance through the native serialization path. It was not changed because no calibration parameter was adopted; no new storage format, schema, or checkpoint mechanism was created.

## 17. Length Independence

The source has no sequence-length condition or lookup for calibration. The diagnostic pools only the existing canonical development records to test whether one global scalar contract can be identified; it does not produce or apply per-length gains.

## 18. Raw / Calibrated Logit Separation

Only raw categorical logits are observed by the C2 probe. There are no calibrated production logits because no gain was accepted. Consequently, raw logits and raw readout weights remain the unchanged source of capability evaluation.

## 19. Argmax Invariance

Not applicable: no production gain was applied. No tie policy or argmax implementation changed.

## 20. Canonical Length Curve

The current raw curve was rechecked by the C1 regression diagnostic. `Calibrated Base` is intentionally not evaluated because no finite independently fitted gain exists.

| Length | Path | Accuracy | NLL | Mean P(True) | Raw Margin | Calibrated Margin |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 8 | Raw Base | 1.000000 | 0.459188 | 0.632291 | 1.129179 | not applied |
| 8 | Calibrated Base | not applicable | not applicable | not applicable | 1.129179 | not applied |
| 8 | No-State | 0.500000 | 0.766560 | 0.464742 | 0.000000 | not applied |
| 16 | Raw Base | 1.000000 | 0.678343 | 0.507527 | 0.347706 | not applied |
| 16 | Calibrated Base | not applicable | not applicable | not applicable | 0.347706 | not applied |
| 16 | No-State | 0.500000 | 0.766560 | 0.464742 | 0.000000 | not applied |
| 32 | Raw Base | 1.000000 | 0.803510 | 0.447796 | 0.067146 | not applied |
| 32 | Calibrated Base | not applicable | not applicable | not applicable | 0.067146 | not applied |
| 32 | No-State | 0.500000 | 0.766560 | 0.464742 | 0.000000 | not applied |

## 21. Length-32 Capability Result

The existing canonical protected-interaction gate remains false because the Base path at length 32 does not satisfy the established Base-versus-No-State categorical-NLL condition. The resulting capability-model status remains `BlockedByLength32ConfidenceCapability`.

## 22. State Regression Audit

Absent. C2 does not apply a projection or mutate the state path. The C1 regression retained finite final-state summaries, including Base mean state norms 95.569050, 226.075958, and 449.269653 for lengths 8, 16, and 32.

## 23. Raw Readout Integrity

Raw readout parameter changes: 0. The only C2 source addition is test-side data-boundary and identifiability measurement. There is no gain update and no raw-logit mutation.

## 24. No-State Baseline

No-State parameters, ablation, training, evaluation, and metrics are unchanged. C2 does not use No-State outcomes as fitting targets.

## 25. G4 Preservation

G4 is unchanged. No G1/G2/G3 candidate is reintroduced.

## 26. Role-Agent Preservation

Trend, Volatility, and Reversal role policies/formulas are unchanged. The existing composed-role boundary guard passed.

## 27. Delivery Freeze

Delivery code, receipts, manifests, generators, and D2 were not run or changed. The existing inexpensive delivery-status independence guard passed.

## 28. Metal Freeze

No Metal topology or hardware behavior changed. The offline library check with `backend-metal` passed; no Metal hardware operation was run.

## 29. Hardcoding Audit

No fitted gain, temperature, length branch, gain lookup table, per-example value, state-dependent calibration, clamp, regularizer, or No-State objective was added. The C2 probe records mathematical derivative evidence only and leaves the production model at its existing behavior.

## 30. Focused Verification

Sequentially completed:

* `cargo fmt --all -- --check`
* `cargo check --offline --lib`
* `cargo check --offline --lib --features backend-metal`
* C2 independent-boundary and scalar-identifiability probe, including the existing canonical protected-interaction gate
* C1 representative length-32 confidence diagnostic regression
* existing production-prefix guard
* existing composed role-boundary guard
* existing delivery-status independence guard

## 31. Explicitly Not Run

* Full default library suite
* Full Metal library suite
* D2
* Metal hardware
* generators
* Selective Forgetting V3
* Formula Lab
* live trading
* internet learning

## 32. Warning Audit

New warnings: 0. Library checks retain four pre-existing dead-code warnings; test compilation retains nine pre-existing dead-code warnings. The C2 instrumentation introduces none.

## 33. Status Separation

* Sprint105-C2: `BlockedByUnidentifiableCalibrationScale`
* Calibration Contract: blocked; no finite identifiable scalar solution
* Capability Model: `BlockedByLength32ConfidenceCapability`
* Delivery: `FROZEN`
* Metal: `FROZEN`
* Overall: `BlockedByUnidentifiableCalibrationScale`

## 34. What This Fixes

It prevents an invalid targeted fix: C2 proves that the available independent calibration source cannot justify a finite positive scalar gain under the unchanged categorical-NLL objective. Evaluation leakage, arbitrary gain selection, and production confidence changes were avoided.

## 35. What This Does Not Prove

It does not prove a production gain, a capability improvement, a general calibration solution, or a remedy for length-dependent raw-margin compression. It does not authorize a clamp, regularizer, different objective, state change, readout-weight change, role change, delivery work, or Metal change.

## 36. Final Status

`BlockedByUnidentifiableCalibrationScale`

## 37. Exactly One Next Step

* independent targeted capability review
