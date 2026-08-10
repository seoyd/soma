# SOMA Sprint 105-C1 M3-Micro Length-32 Confidence Diagnostic Report

## 1. Mode and Git State

Mode: `CAPABILITY_DIAGNOSTIC_ONLY`.

The working branch was `agent/sprint104-r1-m3-micro-functional-conformance-v1`. The implementation changed only test-side instrumentation in `src/model/m3_micro.rs`; this report is the sole diagnostic artifact. There were no staged changes, commits, pushes, or network operations. Pre-existing untracked workspace files were left untouched.

## 2. Starting Capability Blocker

The starting blocker was a genuine length-32 confidence deficit: the Base stateful path retained correct delayed-recall ranking, but its categorical NLL degraded enough at length 32 to be worse than the No-State comparator.

## 3. Progress Ledger

| Stage | Status | Evidence | Finding |
| --- | --- | --- | --- |
| Starting state | VERIFIED/BLOCKED | source and focused run | Length-32 confidence blocker present |
| Current blocker reproduced | PASS | focused diagnostic | Base accuracy 1.000000, Base NLL 0.803510; No-State accuracy 0.500000, NLL 0.766560 |
| Metric semantics | PASS | independent NLL recomputation | all 24 frozen events and 6 populations match |
| Base/No-State fairness | PASS | fixture and parameter audit | identical fixture/training/optimizer; only 6,144 state-capability paths differ |
| Length curve | MEASURED | focused diagnostic | Base NLL rises with length while accuracy remains 1.0 |
| Length-32 example decomposition | MEASURED | focused diagnostic | all four Base examples rank correctly with low confidence |
| Logit scale curve | MEASURED | focused diagnostic | Base relative margins compress with length |
| Correct-class margin curve | MEASURED | focused diagnostic | mean Base margin: 1.129179 → 0.347706 → 0.067146 |
| Temperature diagnostic | MEASURED | frozen-logit counterfactual | lower diagnostic temperature materially improves Base NLL without ranking changes |
| Argmax invariance under T | PASS | focused diagnostic | 0 changes for every path, length, and sampled temperature |
| State diagnostic | MEASURED | test-side private-state summary | state values are finite; magnitude rises with length |
| Numerical finite audit | PASS | focused diagnostic | logits, probabilities, NLL, and observed states are finite |
| Production model changed | NO | source boundary/diff audit | instrumentation is below the test-only boundary |
| G4 changed | NO | diff audit | preserved |
| Role models changed | NO | diff and existing guard | preserved |
| Delivery infrastructure | FROZEN | diff and existing guard | preserved |
| Metal topology | FROZEN | diff and feature check | preserved |
| Root-cause classification | CalibrationDominant | evidence synthesis | confidence scale is the primary diagnostic explanation |
| fmt | PASS | sequential command | passed |
| Default check | PASS | sequential offline command | passed |
| Metal check | PASS | sequential offline feature command | passed |
| New warnings | 0 | compiler audit | only pre-existing warnings remained |
| Sprint105-C1 | READY_FOR_INDEPENDENT_REVIEW | all focused evidence | diagnostic complete |

## 4. Current Capability Call Graph

`delayed_recall_evidence_policy_v2` supplies the canonical lengths and frozen-fixture contract. `delayed_recall_evidence_v2` deterministically invokes `run_delayed_recall_v2` for each length. That routine trains and evaluates the Base and `NoRecurrentState` paths through `evaluate_classification_v2`. The diagnostic consumes those same recorded events, while `structural_loss_evidence_v1` independently recomputes categorical NLL from logits.

## 5. Canonical Benchmark Contract

The canonical delayed-recall policy supplies lengths 8, 16, and 32; balanced binary targets; a fixed training budget of 6; and two frozen evaluation examples per class. Each Base/No-State population therefore contains four frozen events, for six populations and 24 evaluated events in total.

## 6. Metric Semantics Audit

The classifier uses the first three raw logits, stable softmax normalization, the target-class probability, and categorical NLL `-ln(P(true))`. The independent event-level stable-log-sum-exp recomputation matched every recorded event NLL and every population aggregate. The audit does not alias a composite role loss.

## 7. Base / No-State Fairness

Both paths use the same canonical examples, seed family, role, model configuration outside the ablation, training budget, and optimizer contract. The comparator differs only by the documented recurrent-state capability paths: 6,144 expected state paths differ, with no missing or unexpected differences.

## 8. Current Blocker Reproduction

At length 32, Base has accuracy 1.000000 versus No-State 0.500000, demonstrating preserved delayed information. However, Base categorical NLL is 0.803510 versus No-State 0.766560. Thus the starting confidence blocker reproduces under the current canonical fixture, rather than being inferred from a prior record.

## 9. Length Curve

| Length | Path | Accuracy | NLL | Mean P(True) | Median P(True) | Mean Margin | Entropy |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | Base | 1.000000 | 0.459188 | 0.632291 | 0.630471 | 1.129179 | 0.907928 |
| 8 | No-State | 0.500000 | 0.766560 | 0.464742 | 0.464742 | 0.047896 | 0.898975 |
| 16 | Base | 1.000000 | 0.678343 | 0.507527 | 0.509155 | 0.347706 | 0.980585 |
| 16 | No-State | 0.500000 | 0.766560 | 0.464742 | 0.464742 | 0.047896 | 0.898975 |
| 32 | Base | 1.000000 | 0.803510 | 0.447796 | 0.446575 | 0.067146 | 0.992979 |
| 32 | No-State | 0.500000 | 0.766560 | 0.464742 | 0.464742 | 0.047896 | 0.898975 |

## 10. Length-32 Example Decomposition

The four Base examples are ordered by NLL contribution; all are correctly ranked, but their confidence is narrow because the neutral class retains substantial probability mass.

| Example | Correct | P(True) | Top1 | Margin | NLL Contribution |
| --- | --- | ---: | ---: | ---: | ---: |
| 3 | yes | 0.441661 | 0.441661 | 0.043251 | 0.817212 |
| 0 | yes | 0.442424 | 0.442424 | 0.036225 | 0.815486 |
| 1 | yes | 0.450725 | 0.450725 | 0.087175 | 0.796898 |
| 2 | yes | 0.456374 | 0.456374 | 0.101932 | 0.784442 |

## 11. NLL Contribution Distribution

For Base length 32, NLL minimum/Q1/median/Q3/maximum is 0.784442 / 0.793784 / 0.806192 / 0.815918 / 0.817212. The range is narrow: the confidence issue is shared across all four canonical examples rather than caused by a single outlier.

## 12. Logit Scale Curve

| Length | Path | Logit Mean | Logit Std | Abs Max | Logit L2 | Mean Top1-Top2 Margin |
| --- | --- | ---: | ---: | ---: | ---: |
| 8 | Base | 0.045040 | 0.599013 | 0.961999 | 2.080900 | 1.129179 |
| 8 | No-State | 0.264740 | 0.888980 | 0.992164 | 3.213172 | 0.047896 |
| 16 | Base | 0.093724 | 0.567017 | 0.731717 | 1.990856 | 0.347706 |
| 16 | No-State | 0.264740 | 0.888980 | 0.992164 | 3.213172 | 0.047896 |
| 32 | Base | 0.117342 | 0.555752 | 0.675940 | 1.967627 | 0.067146 |
| 32 | No-State | 0.264740 | 0.888980 | 0.992164 | 3.213172 | 0.047896 |

The common logit mean shift is not treated as causal. The relevant finding is the Base relative-margin compression as length grows.

## 13. Correct-Class Margin Curve

Base mean correct-class margin decreases from 1.129179 at length 8 to 0.347706 at length 16 and 0.067146 at length 32. All length-32 examples retain a positive correct-class margin (0.036225–0.101932), so ordering is intact but weak. No-State’s balanced aggregate correct-class margin is 0.000000 at every length because its two correct and two incorrect cases offset.

## 14. Temperature Counterfactual

The test-only frozen-logit sweep used the deterministic geometric temperatures 0.25, 0.5, 1.0, 2.0, and 4.0. It did not alter weights, state, or production configuration. `Argmax Changed` was zero for every row.

| T | Accuracy | NLL | Argmax Changed |
| --- | ---: | ---: | ---: |
| 0.25 | 1.000000 | 0.573856 | 0 |
| 0.50 | 1.000000 | 0.674972 | 0 |
| 1.00 | 1.000000 | 0.803510 | 0 |
| 2.00 | 1.000000 | 0.921480 | 0 |
| 4.00 | 1.000000 | 1.001462 | 0 |

The table is Base length 32, the blocked population. For context, Base NLL at T=0.25 / 0.5 / 1.0 / 2.0 / 4.0 was respectively 0.017487 / 0.161516 / 0.459188 / 0.730455 / 0.902560 at length 8 and 0.229175 / 0.451302 / 0.678343 / 0.856063 / 0.968178 at length 16. No-State remained 0.500000 accurate at all lengths and temperatures, with NLL 0.697992 / 0.705733 / 0.766560 / 0.871179 / 0.964777.

## 15. State Summary

The existing private test context exposes final state without expanding the production API. Values below summarize the final state of the frozen evaluation paths; step-to-step delta is not collected by the existing evaluation contract.

| Length | State Norm | Delta Norm | Finite | Notes |
| --- | ---: | --- | --- | --- |
| 8 Base | 95.569050 | not observed | yes | mean abs 1.235923; max abs 5.456274 |
| 8 No-State | 4.719987 | not observed | yes | mean abs 0.025648; max abs 0.452235 |
| 16 Base | 226.075958 | not observed | yes | mean abs 3.060501; max abs 11.265863 |
| 16 No-State | 4.719987 | not observed | yes | mean abs 0.025648; max abs 0.452235 |
| 32 Base | 449.269653 | not observed | yes | mean abs 6.186203; max abs 22.890062 |
| 32 No-State | 4.719987 | not observed | yes | mean abs 0.025648; max abs 0.452235 |

## 16. Numerical Stability

All evaluated logits, softmax probabilities, NLL values, and observed final-state summaries were finite. The growth in Base state magnitude is measurable but does not produce NaN or infinity in this canonical run.

## 17. Calibration Hypothesis

Supported. At length 32, all Base rankings are correct, while diagnostic scaling from T=1.0 to T=0.25 reduces NLL from 0.803510 to 0.573856 without changing any argmax. This more than closes the current Base-versus-No-State NLL gap (0.036950) on the same frozen examples. This is diagnostic evidence only, not an adopted calibration value or a capability pass claim.

## 18. Readout-Margin Hypothesis

Supported as a secondary mechanism. The Base correct-class margin clearly contracts with length and is small for every length-32 example. It is not selected as the primary classification because a scalar frozen-logit counterfactual materially moves NLL while preserving every ranking; that evidence more directly isolates confidence scale for the present metric blocker.

## 19. State-Representation Hypothesis

Not selected. Base state magnitude grows substantially with sequence length, so it is relevant follow-up context. However, observed values remain finite, Base preserves 100% canonical ranking accuracy, and post-hoc confidence scaling changes the blocked NLL without changing state or ranking. These observations do not establish representation loss as the primary cause.

## 20. Metric / Objective Hypothesis

Not selected. Categorical NLL was independently recomputed from raw logits for every frozen event and matched the recorded values and aggregates. The issue is therefore not an observed metric implementation or aggregation defect.

## 21. Root-Cause Decision

| Hypothesis | Supporting Evidence | Contradicting Evidence | Status |
| --- | --- | --- | --- |
| Calibration | perfect Base ranking; T=0.25 changes length-32 NLL by 0.229654 with 0 argmax changes | raw margins also contract with length | Primary: CalibrationDominant |
| Readout margin | Base correct-class margin contracts to 0.067146 at length 32 | temperature-only scaling materially closes the current NLL gap | Secondary mechanism |
| State representation | final state magnitude rises with length | finite outputs, retained ranking, and scale-only NLL movement | Not primary |
| Metric/objective | none | event and aggregate NLL recomputation matched exactly | Rejected |
| Numerical | none | finite audit passed for all observables | Rejected |

* Primary: `CalibrationDominant`
* Secondary: length-dependent readout-margin compression
* Confidence of classification: moderate-to-high, because the same frozen logits preserve every ranking while scalar temperature alone materially improves the blocked NLL; this does not establish a production temperature or an implementation fix.

## 22. Production Immutability

No production equation, recurrence, state update, readout, confidence head, loss, initialization, or public production API changed. All code instrumentation is test-only and located after the existing test boundary.

## 23. G4 Preservation

G4 was not changed and no G4 counterfactual was run. G1, G2, and G3 were not reintroduced.

## 24. Role-Agent Immutability

Trend, Volatility, and Reversal production policies/formulas were unchanged. The existing composed-role boundary guard passed.

## 25. Delivery Freeze

Delivery code and receipt/manifest behavior were unchanged. The existing cheap delivery-status independence guard passed.

## 26. Metal Freeze

No Metal topology or hardware behavior changed. The offline library check with `backend-metal` passed; no Metal hardware test was run.

## 27. Storage / Diagnostic Artifact Audit

There is one capability diagnostic report and test-only source instrumentation. The diagnostic writes no receipt, manifest, training artifact, fixture, dataset, or runtime measurement file.

## 28. Hardcoding Audit

Lengths, fixture size, training budget, and supported population contract are read from the existing capability policy. The temperature set is a deterministic geometric diagnostic sweep containing 1.0, not a selected production setting. Length-32 confidence buckets use observed distribution ordering rather than fixed probability cutoffs.

## 29. Focused Verification

Sequentially completed:

* `cargo fmt --all -- --check`
* `cargo check --offline --lib`
* `cargo check --offline --lib --features backend-metal`
* `m3_micro_sprint105_c1_length32_confidence_diagnostic`
* existing production-prefix protection guard
* existing composed role-boundary guard
* existing delivery-status independence guard

The C1 diagnostic test covers current reproduction, Base/No-State fairness, independently recomputed NLL semantics, canonical length curve, per-example confidence/NLL decomposition, logit and correct-class margins, temperature counterfactual, argmax invariance, finite audit, and private-state summary.

## 30. Explicitly Not Run

* Full default library suite
* Full Metal library suite
* D2
* Detached closure
* Metal hardware
* Actual Metal mutation replay
* generators
* broker
* live API
* internet learning
* live trading

## 31. Warning Audit

New warnings: 0. The test compilation reported nine pre-existing dead-code warnings; the sequential library checks reported only their applicable pre-existing warnings. No warning was introduced by the C1 diagnostic instrumentation.

## 32. What This Proves

The current canonical benchmark genuinely reproduces the length-32 confidence blocker under a fair Base/No-State comparison. The recorded categorical NLL is exact for the evaluated events. Base retains delayed-recall ranking at length 32, but its confidence scale and correct-class margins deteriorate with length. Frozen-logit temperature scaling can materially improve the blocked NLL without changing ranking in this fixture.

## 33. What This Does Not Prove

This does not prove a production calibration setting, a generalization result beyond the canonical fixture, a causal state-mechanism change, or a production capability improvement. It does not authorize changes to production math, model initialization, role formulas, delivery, or Metal code.

## 34. Recommended Exactly One Next Model Action

Start a **confidence/readout calibration contract phase**.

## 35. Final Status

`READY_FOR_INDEPENDENT_REVIEW`

## 36. Exactly One Next Step

* independent capability diagnostic review
