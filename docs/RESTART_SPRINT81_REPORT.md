# Restart Sprint 81 Report

1. **PR #12 review** — Reviewed the V4 split, registration, participant,
   qualification, contribution, family, decision, roster, evaluation,
   persistence, replay, protected-state, CLI, and authority invariants. The
   implementation was structurally complete, but the terminal taxonomy had one
   blocking semantic defect.

2. **Semantic decision defect** — Every zero-qualified family was mapped to
   `NoQualifiedRawFeatureLearner`, even when all three receipts were
   `RejectedInsufficientValidation`. The hardened logic now derives
   `InsufficientFreshValidation` for the all-insufficient case while preserving
   `NoQualifiedRawFeatureLearner` for a minimum-reached substantive rejection.
   The original three receipts remain unchanged.

3. **Validation-yield audit** — Additive audit `62069972025760a4` derives the
   24-index validation block as 23 valid labelled examples, one neutral
   exclusion, zero horizon exclusions, and zero feature exclusions. Categories
   are mutually exclusive and their total is exactly 24. The unchanged minimum
   is 24, so substantive qualification was not possible in the original pass.

4. **PR #12 merge result** — Hardening commit
   `45d025b50400c65de9200393c7e2502fa72bb439` was pushed, PR #12 was marked
   ready, and it was merged with merge commit
   `8049c26569309994735c5bf9274b148e917471ee`. The remote Sprint 80 feature
   branch was deleted and local `main` was synchronized.

5. **Post-merge verification** — Complete default and Metal verification
   passed on merged `main` before Sprint 81 implementation. Default counts were
   657 library, 404 committee-core integration, and 12 workspace-control tests;
   Metal counts were 658, 404, and 12.

6. **Protected before-state** — Froze 108 pre-existing research-history files,
   including all V1–V4 artifacts, the 312-row snapshot, canonical Momentum
   intent/view, prospective and outcome records, reward-eligibility records,
   and active three-agent state. The pre-execution verification aggregate was
   `dd80d1fc8bb49b2455aafd2e6585916245047b343c15b02da7c6ef33da5a982b`.

7. **Frozen participant reconstruction** — Reopened the exact V4 closure,
   split, registration, family, validation-yield audit, corrected decision, and
   receipts. The raw head, interaction head, and training-prevalence benchmark
   were reconstructed only from the registered configuration and V4 training
   prefix. Parameter, normalizer, model-artifact, training-identity,
   participant, contribution-audit, family, and decision identities matched
   the persisted V4 values. Configuration, participant identities, and
   parameters did not change.

8. **Supplemental preregistration** — Registration `0e9762c34cae048b` binds
   the source, intent, view, V4 split/registration/family/audit, all three
   participant, parameter, and normalizer identities, derived original and
   supplemental ranges, and the unchanged minimum of 24. It was atomically
   persisted and reopened before reserve example or label construction.

9. **Reserve opening result** — A `Ready` receipt was persisted before reserve
   access. The successful `Opened` receipt records one opening attempt and the
   exact 24-index persisted reserve. Repeated execution reports
   `AlreadyOpened`, with zero new row reads, label reads, model work, or writes.

10. **Original valid-sample count** — The original `[264,288)` validation block
    contributes 23 valid labelled examples and one neutral exclusion. No
    private label, return, probability, or metric value is exposed.

11. **Supplemental valid-sample count** — The derived `[288,312)` supplemental
    block contributes 23 valid labelled examples and one neutral exclusion. No
    row trained a head, fit a normalizer, changed a parameter, or crossed the
    312-row source boundary.

12. **Accumulated valid-sample count** — The exact duplicate-free union
    contains 46 valid examples and reaches the unchanged minimum of 24. Metrics
    and predictions were recomputed directly over this union rather than
    averaging earlier statuses or summaries.

13. **Raw-logistic accumulated qualification** —
    `RawFeatureLogisticV4` derives `QualifiedLearned` under the frozen
    qualification policy. This is a bounded policy classification, not an
    improvement or superiority claim.

14. **Interaction accumulated qualification** —
    `RawFeatureInteractionLogisticV4` derives `QualifiedLearned` under the
    frozen qualification and contribution policies. Its frozen parameters and
    normalizers are unchanged.

15. **Benchmark accumulated qualification** —
    `TrainingPrevalenceConstantV4` derives `BenchmarkQualified`. It remains a
    benchmark and is not counted as a learned participant.

16. **Accumulated contribution audit** — Additive audit
    `1701e17d2dc56f8d` derives `MaterialInteractionContribution` from full and
    nonlinear-block-ablated predictions over the accumulated set. The original
    V4 contribution audit was not overwritten, and no ranking or winner was
    created.

17. **Accumulated family** — Family `4900d33cd7f0eb60` references the original
    V4 family and contains all three frozen participants and additive receipts.
    It records two qualified learned participants and one qualified benchmark;
    winner selection, parameter change, active-committee eligibility, promotion
    eligibility, and reward eligibility are all false.

18. **Accumulated path decision** — The bounded decision is
    `RawFeatureLearnedPathViable`, digest `d08e8a1d2ecfef4f`. It follows the
    minimum-reached plus materially contributing interaction rule and does not
    select a participant.

19. **Future roster** — Roster `4883ef775c6e5589` is `Ready` and includes both
    qualified learned participants plus the qualified benchmark. It contains
    every qualifying role without private-metric ranking; no semantic duplicate
    was admitted.

20. **Future evaluation registration** — Registration
    `372b95d7dfee4bef` binds the original and accumulated families,
    supplemental registration, opening receipt, yield, all accumulated
    receipts, contribution audit, full source boundary, consumed V1–V4
    validation identities, protected registrations/timestamps, and provider
    finality. Its minimum accepted timestamp is `1784764800000`, with one
    request, concurrency one, and zero retries. No future evidence was acquired
    or opened.

21. **Additional-evidence requirement** — None was created because 46 valid
    accumulated examples reached the unchanged minimum. The implemented
    insufficient path deterministically derives the remaining gap and requires
    a separate new-evidence preregistration.

22. **Cycle/Risk blocker** — The independent state remains
    `ProviderContractUnverified`. Momentum evidence was not reused or relabelled
    as index or volatility evidence.

23. **Value/Quality blocker** — The independent state remains
    `TrainerUnavailable`. No Value trainer or generic training path was added.

24. **Prospective attribution replay** — Read-only replay remains
    `MissedMaterialOpportunity` for Momentum and `CorrectUncertainty` for
    Cycle/Risk and matches the persisted prospective records.

25. **Reward eligibility and zero application** — Both persisted outcomes
    remain `IneligibleMinimumSamples`. Reward and penalty applications are
    zero; voice, cooldown, promotion, quarantine, and Chair actions are also
    zero.

26. **Protobuf persistence and replay** — Hand-written `prost::Message`
    contracts cover supplemental registration, two opening receipts,
    supplemental yield, three accumulated receipts, interaction audit,
    accumulated family, decision, optional roster, optional future
    registration, optional additional-evidence requirement, and journal. The
    completed pass contains 13 additive sidecars. An identical replay writes
    zero and duplicate-rejects all 13; malformed Protobuf is rejected.

27. **Network and authority counters** — Network requests, transports,
    credential reads, new prospective reads/openings, historical-test reads,
    future-evaluation reads, participant changes, active-model changes, Chair
    decisions, votes, rewards, penalties, voice changes, cooldowns, promotions,
    quarantines, and executions are zero. The authorized first opening records
    one attempt, 24 reserve-row reads, and 24 reserve-label reads; replay records
    zero for all three. Active committee count remains three. Network and
    authority flags reject before local execution.

28. **Protected artifact validation** — All 108 pre-existing files remain
    byte-identical after status, dry-run, authority rejection, first execution,
    and repeated execution. The verification aggregate remains
    `dd80d1fc8bb49b2455aafd2e6585916245047b343c15b02da7c6ef33da5a982b`;
    the 16 V4 sidecars retain aggregate
    `91ab6365f709ebdad0860b50d3a0efd94ccb89f9dcf22a458b7164b7b704ec88`.
    In-process protected-byte and active-state checks also passed.

29. **Files changed** — Implementation is focused in
    `src/model/momentum_raw_feature_supplemental.rs`. Existing V4 source is
    reused for deterministic reconstruction and accumulated evaluation, with
    only model export and CLI wiring in existing files. Documentation is
    limited to the raw-feature path, supplemental protocol, and this report.
    Seven tracked files comprise the final change; runtime sidecars remain
    ignored and uncommitted.

30. **Complete verification** — Formatting, default workspace check, and Metal
    workspace check passed. Default tests passed with 686 library, 404
    committee-core integration, and 12 workspace-control tests; Metal passed
    with 687, 404, and 12. Focused supplemental tests passed 29 of 29. Git diff
    checks passed. Every Rust build used one job and every test run used one
    test thread.

31. **Boundary audits** — Both boundary audits passed.

32. **What was proven** — The implementation enforces corrected V4 taxonomy,
    actual yield accounting, frozen deterministic reconstruction,
    preregistration-before-access, one-time reserve opening, exact accumulated
    evidence, direct metric recomputation, additive contribution and
    qualification artifacts, all-qualified roster inclusion, future-contract
    binding, Protobuf integrity/idempotency, protected-state immutability, and
    zero external authority.

33. **What remains unproven** — Qualification does not prove model improvement,
    participant superiority, a winner, future performance, reward or penalty
    effect, promotion readiness, Chair learning, or live-trading readiness. The
    registered future evaluation has not acquired or opened new evidence.

34. **Commit, push, and draft PR** — Implementation commit
    `70499c98af6e84c3003431eff3a845692793ce9e` was pushed on
    `agent/sprint81-v4-supplemental-qualification`. This verified report is
    included in the follow-up commit on the same branch. Draft PR
    [#13](https://github.com/seoyd/soma/pull/13) targets `main`.

35. **Next Sprint recommendation** — Preserve V1 through V4.1 as immutable
    research history. A later, separately authorized Sprint may execute the
    registered one-time future evaluation only with evidence strictly newer
    than `1784764800000` and only after its finality and identity checks pass.
    Do not retune on accumulated qualification evidence, rank private metrics,
    weaken policy, promote a participant, or alter the active committee.
