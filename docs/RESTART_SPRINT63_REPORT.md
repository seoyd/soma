# Sprint 63 Restart Report

## Sprint summary

Implemented a sealed prospective-only learned-agent attribution and reward-eligibility bridge that ends at a compute-only candidate.

## Baseline verification

Before implementation, sequential formatting, default check/test, and Metal check/test passed with existing unrelated warnings unchanged.

## Immutable before-state

The Momentum challenge was sealed with no future rows, events, or opened labels; Cycle/Risk remains a sealed offline tournament contract.

## Prospective-contract audit

Exact challenge, model, horizon, agent, objective, post-cutoff timestamp, and sealed opinion identities are required.

## Existing reward-system audit

The bridge targets the existing bounded reward computation and defines no second score or formula.

## Registration architecture

`LearnedRewardEligibilityRegistrationV0` binds both contracts and all required policy digests.

## Registration digest

The registration digest commits to all policy digests and the complete no-retroactive/no-owner/no-application prohibition set.

## Attribution architecture

Each event has one sealed opinion, one agent, one objective, one challenge, one model, one evidence identity, and one horizon; duplicate IDs are rejected.

## Maturity policy

Labels require finalized rows, matching identity, valid challenge, authorization, and maturity; early or repeat opening is invalid.

## Objective-specific outcome policy

Momentum and Cycle/Risk use separate outcome payload types and cannot be swapped.

## Abstention attribution

Justified protection, uncertainty, missed opportunity, failed warning, neutral, and not-yet-evaluable abstentions are explicit and prospective only.

## Sample gates

Mature-event and support gates are derived from the sealed evaluation policies; regime coverage is derived from the offline Cycle/Risk contract.

## Integrity gates

Challenge invalidation, integrity failure, early access, duplicate opening, retroactive evidence, owner influence, and wrong objectives are ineligible.

## Owner-input exclusion proof

Owner influence is an explicit rejection path; the Sprint 62 advisory record remains observation-only and is not a reward input.

## Reward adapter boundary

Synthetic eligible results produce typed signals only; `eligible_for_application` is always false.

## Fixture results

Deterministic coverage includes valid Momentum/Cycle-Risk outcomes, justified abstention, high-confidence losses, insufficient samples, opening failures, retroactive evidence, owner influence, and swapped payloads.

## Current real-state eligibility

The read-only CLI derives `IneligibleNoProspectiveOutcomes` from the empty ledger rather than hard-coding it.

## Current event/maturity/candidate counts

Initial counts are zero events, mature outcomes, reward candidates, and applications.

## Ledger and reopen result

The ledger is digest-protected and deterministic; validation accepts an equivalent reopened value and this work does not write real state.

## Text/JSON determinism

`--learned-reward-eligibility` reports matching public statuses, counts, gates, and digests without raw labels, probabilities, rows, notes, or paths.

## Reward/voice mutation audit

No bridge code calls reward application, feedback application, voice/tier/cooldown/promotion/demotion/quarantine mutation, or execution.

## Network and label-access audit

The command is offline-only and reports zero provider, transport, consent, credential, and label reads.

## Old/prospective artifact freeze

Historical artifacts, earlier Sprint boundaries, prospective capsule/registry/vault/journal artifacts, acquisition blocking, and active Chair/Risk/Paper behavior are untouched.

## Files changed

Existing Rust modules were extended for the bridge, exports, CLI, and objective JSON representation; only the requested attribution/report documents were added and the existing reward document was extended.

## Complete final verification

Sequential final verification passed: formatting; default workspace check; default tests (328, 404, and 12-test groups); Metal workspace check; Metal tests (329, 404, and 12-test groups); and both working-tree and staged diff checks. The only warnings were the pre-existing unused-mut/dead-code warnings.

## Instruction-file boundary

No implementation or documentation artifact embeds or depends on the instruction-file name.

## Unrelated-file boundary

Only scoped implementation and requested documentation paths are staged.

## What was proven

Deterministic tests cover registration integrity, event attribution, maturity, objective separation, ineligibility derivation, compute-only candidate construction, and zero application counters.

## What remains unproven

No real prospective event has matured; no prospective performance, profit, real reward/penalty, voting, promotion, or execution claim is made.

## Commit/push result

The Phase-A registration commit `a8552ae` was pushed to `origin/main`. This CLI/report completion is committed and pushed immediately after this report is finalized.

## Next Sprint recommendation

After independently acquired future rows satisfy the sealed contracts, use a separately reviewed one-time opening and candidate computation before considering any Chair application.
