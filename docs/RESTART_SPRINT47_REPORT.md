# Sprint 47 restart report

## 1. Sprint summary

Implemented a semantic support-gate closure for the offline BTC cross-regime
diagnostic. The change separates usable support evidence from a genuinely
unavailable gate without changing models, features, labels, candidates, or
thresholds.

## 2. Honest AI maturity status

This remains a deterministic, CPU/Metal-tested, ShadowOnly research prototype;
it is not a trading-ready AI and gains no vote, promotion, or execution power.

## 3. Complete baseline verification

Before implementation, formatting, workspace checking, default tests, and
Metal-feature tests completed sequentially. The four pre-existing dead-code
warnings remained the only warnings.

## 4. Immutable-input revalidation

The offline diagnostic verified the accepted snapshot digest and consumed only
the frozen local evidence path.

## 5. Regime-pack verification

Two chronological frozen BTC packs were replayed, each with 152 rows; their
pack identities were accepted by the sealed regime path.

## 6. Model/config freeze proof

The diagnostic reported `model_freeze_all_equal=true` with freeze proof digest
`b3f23d561cd229c4`.

## 7. Support-state source audit

The prior mapping classified a validation-coverage threshold breach as
`SupportGateUnavailable`; it also gave a validation-rejected path the same
test status instead of stating that test support had not been evaluated.

## 8. Current support-state reproduction

All four selected paths constructed Ready envelopes and had Applicable gates.
Two validation paths were InSupport and two were OutOfSupport; none had a true
gate failure or insufficient-support result.

## 9. Current aggregate reproduction

The support aggregate is validation InSupport 2, validation OutOfSupport 2,
test InSupport 1, test OutOfSupport 1, true unavailable 0, and accepted
research-only versions 1.

## 10. Support-envelope architecture

The train-only frozen-representation envelope trace records construction
status, sample and dimension counts, finite statistics, constant dimensions,
and a deterministic digest.

## 11. Gate-applicability architecture

Applicability is calculated independently from finite metrics, representation
shape, and minimum samples. Threshold rejection cannot turn an Applicable gate
into an unavailable gate.

## 12. Partition-support decision architecture

Validation and test decisions remain separate. Validation is InSupport,
OutOfSupport, insufficient, unavailable, numerical failure, or explicitly not
evaluated only where appropriate.

## 13. Operational support-outcome architecture

Per-path support traces feed a regime-level dominant support outcome while the
existing operational Shadow result continues to control research-only version
acceptance.

## 14. Required support metrics

Stable-order required metrics are validation sample count, validation coverage,
mean standardized shift, maximum standardized shift, mean log variance ratio,
and out-of-support fraction.

## 15. Optional support metrics

No optional metric is currently implemented or fabricated; the trace records
zero missing optional metrics.

## 16. Older-regime envelope construction

Both older-regime paths constructed Ready envelopes from the selected
train-only representations.

## 17. Older-regime support-metric trace

The first older path had 16 validation samples and all metrics except mean log
variance ratio passed. The second had 16 samples and all six metrics passed.

## 18. Older-regime first breach or missing metric

The first breach was mean log variance ratio 2.3182707 against 2.0. No required
or optional metric was missing.

## 19. Older-regime gate applicability

Both older paths were Applicable; unavailable count was zero.

## 20. Older-regime train-history applicability audit

Both deterministic two-fold label-free chronological audits were
`OverRejectingOnTrainingHistory`, with 0 in-support and 2 out-of-support folds.
This is diagnostic-only and does not change any gate threshold.

## 21. Older-regime validation support decision

One path was OutOfSupport and one was InSupport.

## 22. Older-regime test-seal result

The validation-rejected path recorded `NotEvaluated`; the validation-supported
path was evaluated once and remained InSupport.

## 23. Older-regime operational abstention

The rejected path abstained; the supported path remains only a
`ShadowPredictionResearchOnly` result. Neither can vote, promote, or execute.

## 24. Older-regime support-qualified versions

One actual support-qualified research-only version was accepted.

## 25. Older-regime support verdict

The regime's dominant support outcome is `Mixed`.

## 26. Newer-regime envelope construction

Both newer-regime paths constructed Ready envelopes from selected train-only
representations.

## 27. Newer-regime support-metric trace

The first newer path passed all validation metrics. The second had 15 samples
and breached validation coverage, both standardized-shift metrics, mean log
variance ratio, and out-of-support fraction.

## 28. Newer-regime first breach or missing metric

Its first deterministic breach was validation coverage 0.7 against 0.8; no
required or optional metric was missing.

## 29. Newer-regime gate applicability

Both newer paths were Applicable; unavailable count was zero.

## 30. Newer-regime train-history applicability audit

The first two-fold audit had 0 in-support and 2 out-of-support folds. The
second had 1 and 1 respectively. Both report `OverRejectingOnTrainingHistory`
as a diagnostic warning only.

## 31. Newer-regime validation support decision

One path was InSupport and one was OutOfSupport.

## 32. Newer-regime test-seal result

The validation-supported path was evaluated and rejected at test as
OutOfSupport. The validation-rejected path records `NotEvaluated`.

## 33. Newer-regime operational abstention

The regime remains `ShadowAbstainOutOfSupport` with no accepted predictive
version.

## 34. Newer-regime support-qualified versions

Zero versions qualified through both validation and test support.

## 35. Newer-regime support verdict

The regime's dominant support outcome is `Mixed`.

## 36. Per-regime replay determinism

A repeated offline JSON replay reproduced regime report digests
`9a8fba8550a051e2` and `16b57ec78f8e392e` and their execution-trace digests.

## 37. Actual InSupport count

Validation InSupport count is 2; test InSupport count is 1.

## 38. Actual OutOfSupport count

Validation OutOfSupport count is 2; evaluated-test OutOfSupport count is 1.

## 39. Actual insufficient-support count

The aggregate insufficient-support count is 0.

## 40. Actual gate-unavailable count

The aggregate true gate-unavailable count is 0.

## 41. Existing SparseInSupportEvidence audit

The old sparse result was caused by conflating threshold rejection and
not-evaluated test support with gate availability. The new aggregate forbids a
zero-qualified result from being called sparse.

## 42. Minimum semantic implementation fix

Validation coverage below threshold now produces OutOfSupport, and test support
after any validation non-acceptance produces NotEvaluated. Threshold values and
selection logic are unchanged.

## 43. Cross-regime support aggregate

The aggregate counts are 2 validation accepts, 2 validation rejections, 1 test
accept, 1 test rejection, 0 insufficient, 0 unavailable, and 1 accepted
research-only version.

## 44. Cross-regime support status

`SparseSupportQualifiedEvidence` is accurate because exactly one actual path
qualified through test support; it is not a recurrence or readiness claim.

## 45. Cross-regime representation status

`StableAcrossAvailableRegimes` is reported separately and does not override the
sparse support result.

## 46. Support-versus-representation interpretation

Representation stability and predictive support qualification answer different
questions. The former does not prove the latter.

## 47. Accepted predictive versions

Exactly one version is accepted for research-only Shadow observation; accepted
count does not authorize live use.

## 48. Offline CLI result

Text and JSON `--btc-cross-regime-diagnostics` completed successfully with two
regimes and cross-regime digest `4301a770f34107b2`.

## 49. CLI exit-status audit

The valid negative and sparse-evidence result exits successfully. A nonzero
exit remains reserved for a technical diagnostic failure.

## 50. Report redaction and determinism

Output contains sanitized IDs, metrics, thresholds, states, and digests; it
contains no credentials, provider response, or local evidence rows.

## 51. Network firewall

The replay reported provider calls 0, post-freeze provider calls 0, transport
construction 0, and network-consent reads 0.

## 52. Evidence-ledger result

The immutable evidence ledger completed with digest `a3249852cc155c5e`.

## 53. Prospective holdout result

The status is `PolicySealedNoFutureRows`; it was not opened and no holdout
labels were accessed.

## 54. Layered eligibility

Support traces can only document diagnostic eligibility. Promotion, voting, and
execution eligibility remain false.

## 55. Backend result

Default CPU and Metal-feature workspace checks and tests completed sequentially;
the Metal library test set included its available-device transition test.

## 56. Official Mamba conformance status

The repository retains its bounded reference implementation and explicit
oracle-availability boundary; this change does not claim official external
Mamba conformance.

## 57. Storage/evidence immutability

The replay uses immutable local snapshot and frozen pack evidence, including
the existing Protobuf storage path; it performs no data rewrite.

## 58. Hardcoding audit

No regime-specific outcome, metric value, threshold, or accepted-version count
was hardcoded. Traces derive from the campaign's frozen representations.

## 59. Model/trading isolation

The work is confined to diagnostic semantics and reporting. It does not modify
model training, market data acquisition, Chair, Risk Governor, or trading code.

## 60. Files added

Added `BTC_SUPPORT_GATE_CLOSURE.md` and this restart report.

## 61. Files changed

Changed the existing learning campaign, historical evidence, model export, CLI,
cross-regime closure, and multi-regime evidence files.

## 62. Tests added or reused

Added focused tests for coverage rejection semantics, label-free chronological
train-history audit, and the zero-qualified sparse-status guard; reused the
workspace default and Metal suites.

## 63. Complete final verification

`cargo check --workspace`, default workspace tests, Metal workspace check, and
Metal workspace tests were run with `CARGO_BUILD_JOBS=1` sequentially. A direct
final Metal library run passed 215 tests.

## 64. Build-cache result

No cache was deleted. The existing build cache occupied about 48G after final
verification, with about 172Gi free on the workspace volume.

## 65. Instruction-document boundary result

The instruction document was used only to define implementation scope; its
contents were not copied into runtime source, tests, or report payloads.

## 66. Unrelated untracked-file boundary result

The pre-existing unrelated untracked file was not read, modified, staged, or
committed.

## 67. Risk/security review

The change fails closed on non-finite or undersized support metrics, preserves
the offline firewall, retains test-seal semantics, and creates no live authority.

## 68. What was proven

The old unavailable label was semantically overbroad; the fixed offline replay
has zero true gate failures, two actual validation rejections, one actual
support-qualified test path, and deterministic trace output.

## 69. What remains unproven

The evidence does not prove recurrent predictive value, external replication,
prospective performance, calibration robustness, or live-trading readiness.

## 70. Deferred items

Do not tune thresholds from this diagnostic. Future work requires genuinely new
immutable chronology and prospective holdout policy approval.

## 71. Commit/push result

The completed implementation is committed on `main` and pushed to the existing
`origin` remote.

## 72. Next gstack Sprint recommendation

Keep support and representation aggregates separate; collect only approved new
future evidence before considering a pre-registered prospective evaluation.
