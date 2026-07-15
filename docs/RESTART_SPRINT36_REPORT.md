# Restart Sprint36 Report

## 1. Sprint summary

The campaign’s semantic-digest verifier was corrected and an ordered sanitized safety trace plus layered eligibility were added. Offline Shadow evaluation now completes on the accepted frozen Protobuf evidence; all higher authorities remain blocked.

## 2. Full baseline verification

Before the change, default formatting and workspace checks passed; default tests passed 602 tests. Metal checks passed and Metal tests passed 603 tests.

## 3. Source audit result

The campaign rehashed a JSON serialization while acquisition, inventory, pack verification, and Protobuf persistence used the canonical semantic dataset digest. That inconsistent verifier was the rejection source.

## 4. Accepted Protobuf evidence revalidation

One locally stored Protobuf snapshot reloaded with a matching canonical digest; inventory accepted one real historical series.

## 5. Frozen evidence-pack verification

The deterministic pack verified as frozen before campaign evaluation. Training received the pack’s immutable snapshots only.

## 6. Offline rejection reproduction

The pre-fix offline campaign reproduction returned `RejectedForSafety` with zero windows and zero versions. No network-enabled invocation was used.

## 7. Safety gate evaluation order

Immutable/sanitized evidence, real historical classification, canonical semantic digest, chronology, finite OHLCV, history length, purge-separated windows, CPU readiness, frozen encoder capture, offline Shadow eligibility, promotion, voting, execution, and frozen encoder unchanged are ordered in the result trace.

## 8. First rejecting gate

The reproduced first rejection was `CanonicalSemanticDigest`.

## 9. Exact reason code

`canonical_semantic_digest_mismatch`.

## 10. Sanitized evaluated facts

The rejected trace reports the snapshot count and `semantic_digest_match=false`; it contains no identifier, path, credential, raw row, or price.

## 11. Root-cause classification

Implementation bug: an obsolete JSON-byte hash verifier remained after canonical semantic identity was introduced. It was not a bad snapshot, mutable evidence, provider problem, or policy denial.

## 12. Intended safety policy

Immutable, sanitized, credential-free, accepted real daily OHLCV with matching canonical identity may enter an offline-only evaluation. Evidence integrity, chronology, finite values, leakage protection, backend readiness, and encoder immutability remain mandatory.

## 13. Non-negotiable gate result

All non-negotiable gates passed in the corrected local rerun: canonical digest, frozen evidence pack, chronological/purged windows, finite feature path, CPU full inference path, frozen encoder check, and closed campaign transport boundary.

## 14. Configuration correction

No local provider, credential, market, network, backend, or trading configuration was relaxed. The existing campaign configuration was reused.

## 15. Implementation bug fix

`validate_historical_evidence` now invokes `historical_replay_dataset_digest_v0` directly instead of hashing JSON serialization output.

## 16. Layered eligibility architecture

The result records four distinct booleans: offline Shadow learning, promotion, voting, and execution. They cannot inherit permission from one another.

## 17. Offline Shadow learning eligibility

Eligible after the evidence and runtime gates passed. It produces only offline assessments and `ShadowOnly` versions.

## 18. Promotion eligibility

Blocked: `experimental_internal_reference`.

## 19. Voting eligibility

Blocked: `shadow_only`.

## 20. Execution eligibility

Blocked: `official_oracle_execution_blocked`.

## 21. Files added

`docs/CAMPAIGN_LAYERED_SAFETY_ELIGIBILITY.md` and this report.

## 22. Files changed

`src/model/learning_campaign.rs`, `src/model/mod.rs`, `src/cli.rs`, and `docs/MOMENTUM_REAL_HISTORICAL_EVIDENCE.md`.

## 23. Campaign rerun result

The corrected offline rerun produced `DriftDetected`, four evaluated windows, and seven generated Shadow versions. No safety rejection was recorded.

## 24. Chronological window result

Four expanding walk-forward windows were built with the existing two purge boundaries. Train-only normalization and existing sequence-boundary leakage checks remained active.

## 25. Constant baseline result

The seven evaluated paths did not show a uniform Frozen-Mamba improvement over the constant baseline; four paths improved and three did not.

## 26. Linear baseline result

The seven evaluated paths did not show a uniform Frozen-Mamba improvement over the linear baseline; six paths had lower Brier loss and one had higher Brier loss.

## 27. Frozen-Mamba result

The frozen encoder plus trainable logistic head produced seven offline path results. This is a small single-series observation, not a model-value or profitability conclusion.

## 28. Mamba-versus-linear result

The observed per-path Brier deltas versus linear ranged from -0.035015 to +0.002000. The aggregate campaign status is drift-detected, so the result cannot satisfy a promotion or value gate.

## 29. Cold-versus-warm result

Three windows had both paths: warm had lower Brier loss once and cold had lower Brier loss twice. The first window has no warm parent by design.

## 30. Drift and calibration result

The campaign reported `ProbabilityCollapse`, yielding `DriftDetected`. This is a protective negative result; no calibration or deployment claim follows.

## 31. Real single-series verdict

One accepted series proves only that the offline pipeline can evaluate this frozen evidence. It does not prove a generalizable edge, profitability, or market readiness.

## 32. Model-version result

Seven version records were generated with `ShadowOnly` deployment status. They are isolated campaign artifacts with no voting or execution authority.

## 33. Shadow-assessment result

Offline assessments were generated only for the seven evaluated paths. Their authority flags remain false for voting and execution.

## 34. Backend result

The existing CPU full-inference route was used. Metal remains a separately verified partial-backend build boundary, not a campaign authority change.

## 35. Official Mamba conformance status

The model remains `ExperimentalInternalReference`; official-oracle execution is blocked. No official Mamba conformance claim is made.

## 36. Network-to-training firewall

The campaign API receives snapshots and a frozen encoder, not a provider, broker, or transport. The rerun did not use a network-enable flag or invoke a provider.

## 37. Hardcoding audit

The fix calls the shared canonical digest function. No digest, snapshot identifier, market value, window result, or authority decision was hardcoded.

## 38. Model/trading isolation

The change adds no committee membership, order capability, account access, promotion action, vote action, or execution path.

## 39. Tests added or reused

The learning-campaign test set now verifies the first canonical-digest rejection gate/reason and successful canonical-digest Shadow eligibility. The existing campaign test continues to assert Shadow-only versions and no vote/execution flags.

## 40. Full final verification

Before cache cleanup, the final source tree passed workspace checks for the default and Metal feature sets and the default workspace test suite. The full Metal workspace suite was stopped only to prevent disk exhaustion. After cleaning 185.4GiB of project build cache, the focused learning-campaign tests passed for both default and Metal feature sets, and the offline frozen-evidence rerun reproduced the recorded four-window result. Formatting and diff checks pass.

## 41. Request-source boundary result

The request document was read as implementation input only. It was not copied, embedded, or included by source, tests, configuration, fixtures, or documentation.

## 42. Unrelated-file boundary result

The pre-existing unrelated untracked file was left untouched and excluded from the change set.

## 43. Risk/security review

The change narrows a false rejection without weakening canonical identity or evidence policy. Trace facts are sanitized, upper authority remains blocked, and no new network or trading capability exists.

## 44. What was proven

Canonical Protobuf evidence can pass the corrected campaign verifier, enter a frozen offline pack, produce walk-forward Shadow evaluations, and retain strict higher-authority blocks.

## 45. What remains unproven

Generalization across markets, repeatable model value, calibration stability, official Mamba conformance, live readiness, profitability, and any trading suitability remain unproven.

## 46. Deferred items

Additional providers, new evidence acquisition, compression/storage changes, Toss integration, GPU training, Mamba gradients, promotion, voting, execution, and new agents are deferred.

## 47. Commit/push result

This report and the implementation are committed and pushed together to `origin/main`.

## 48. Next gstack Sprint recommendation

Keep the current offline-only scope. Add more independently accepted immutable historical series only through the existing approved evidence path, then reassess cross-series evidence and drift without granting higher authority.
