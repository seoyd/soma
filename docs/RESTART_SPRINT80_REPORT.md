# Restart Sprint 80 Report

1. **PR #11 review and merge** — Reviewed all seven changed files and the V3
   probe, split, preregistration, route, contribution, qualification, decision,
   persistence, replay, protected-state, and authority invariants. No blocking
   defect remained. PR #11 was marked ready, merged with merge commit
   `84658009395d73080a8f737ae93d69d308ae8564`, and its remote feature branch
   was deleted. Both Sprint 79 commits remain ancestors.

2. **Post-merge verification** — Synchronized `main` and passed complete
   default and Metal verification before V4 implementation. Default counts were
   615 library, 404 committee-core integration, and 12 workspace-control tests;
   Metal counts were 616, 404, and 12.

3. **Protected before-state** — Froze 92 existing research-history files,
   including the verified snapshot, canonical intent/view, V1/V2/V3 artifacts,
   prospective and reward records, aggregate, and active three-agent state. The
   protected byte aggregate was
   `2ec8a4dd60432170f0f026643ac789a04ab622e71fa08c9eaa3a5579f39d9cbb`.

4. **Frozen-Mamba closure scope** — Closure
   `2d7667c8cf72ed5e` is `ClosedForCurrentEvidenceAndPolicy` and binds the exact
   frozen encoder, 312-row evidence, feature policy, label policy, and verified
   V1/V2/V3 chain. Its genuine-Mamba qualified count is zero. Head-only repair,
   another frozen-representation sweep, and use of this closed encoder as a V4
   parent are forbidden; reopening requires new encoder, evidence, and
   preregistration identities. No global architecture claim is made.

5. **V4 split and untouched reserve** — The split is derived from the persisted
   V3 split: training `[0,240)`, purge `[240,264)`, fresh validation
   `[264,288)`, and final untouched reserve `[288,312)`. Split digest:
   `a072cecd98df1f44`. All required overlap counts are zero, and the purge covers
   the feature, sequence, and label horizons.

6. **V4 preregistration** — Registration `b6ce06fd9f226277` was atomically
   persisted and reopened before fresh-validation inference. It freezes exactly
   two learned configurations and one benchmark, with no result-dependent
   second batch, winner selection, active promotion, reward, test, reserve, or
   future-evaluation authority.

7. **Raw logistic participant** — `RawFeatureLogisticV4` is freshly initialized
   and consumes the existing ordered engineered feature vector. It uses a
   training-only normalizer, the existing Brier-loss SGD, fixed registered
   hyperparameters, finite guards, and no prior parameters, normalizer, or
   predictions.

8. **Interaction logistic participant** —
   `RawFeatureInteractionLogisticV4` deterministically orders originals,
   squares, and all pairwise `i < j` products. Its schema-bound expansion and
   independent training-only normalizer reject nonfinite input/output, width or
   order inconsistency, duplicate identity, and normalization leakage. Cubic,
   random, learned, and validation-selected features are absent.

9. **Constant benchmark** — `TrainingPrevalenceConstantV4` derives its finite
   probability from V4 training labels only. Zero variance is permitted for the
   benchmark role, and it is never counted as a learned participant.

10. **Shared fresh-validation execution** — All three participants share the
    same snapshot, training timestamps, purge, fresh-validation timestamps, and
    label policy. Validation parameter updates are zero. The 24-row validation
    index block does not yield the policy-required 24 valid labelled samples,
    so qualification records an insufficient-sample result without weakening
    the threshold or reading the reserve.

11. **Participant qualification statuses** — `RawFeatureLogisticV4`,
    `RawFeatureInteractionLogisticV4`, and `TrainingPrevalenceConstantV4` are
    each `RejectedInsufficientValidation`. These statuses are derived from the
    common execution rather than hardcoded production results.

12. **Interaction contribution audit** — The deterministic block-zero audit is
    `MaterialInteractionContribution`. It records parameter contribution only;
    it neither overrides failed qualification nor proves nonlinear progress or
    model improvement.

13. **V4 family** — Family `bca7665e1e1b2012` retains all three participants,
    all three qualification receipts, and the contribution audit. Qualified
    benchmark count is zero. No winner was selected, and active-committee,
    promotion, and reward eligibility are false.

14. **Qualified learned count** — Zero learned participants qualified. The
    constant benchmark is excluded from this count by role.

15. **Path decision** — The derived decision is
    `NoQualifiedRawFeatureLearner`, digest `d454a95486fc3e77`.

16. **Future roster** — Status is `NoQualifiedLearnedParticipant`; no roster
    artifact exists. The implemented ready path includes every qualified
    learned participant plus a qualified benchmark and semantically deduplicates
    a linear-equivalent interaction without private-metric ranking.

17. **Evaluation registration** — Status is
    `NoQualifiedLearnedParticipant`; no evaluation-registration artifact exists
    and no future evidence was acquired or opened.

18. **Exclusions and minimum accepted timestamp** — The optional contract binds
    closure, family, roster, split, registration, receipts, contribution audit,
    source boundary, protected prospective identities and timestamps, provider
    finality, prior validation identities, and the V4 reserve identity. Labels
    and probabilities stay hidden; opening is one-time with concurrency one and
    zero retries. The roster gate failed, so no minimum accepted timestamp was
    assigned.

19. **Final-reserve zero-access proof** — Final-reserve row reads and label
    reads are both zero. The `[288,312)` reserve was not built, read, or opened
    during preregistration, qualification, status, dry-run, execution, replay,
    or optional-registration handling.

20. **Cycle/Risk blocker** — The independent state remains
    `ProviderContractUnverified`; Momentum evidence was not repurposed or
    relabelled.

21. **Value/Quality blocker** — The independent state remains
    `TrainerUnavailable`; no Value training or generic trainer was added.

22. **Prospective attribution replay** — Read-only replay preserves Momentum as
    `MissedMaterialOpportunity` and Cycle/Risk as `CorrectUncertainty` and
    matches the persisted prospective records.

23. **Reward eligibility and zero application** — Both existing outcomes
    derive as `IneligibleMinimumSamples`. Reward and penalty applications are
    zero; voice, cooldown, promotion, quarantine, and Chair actions are also
    zero.

24. **Protobuf persistence and replay** — Hand-written `prost::Message`
    contracts cover closure, split, registration, participants, receipts,
    contribution audit, family, decision, optional roster, optional evaluation
    registration, and journal. The applicable result contains 13 Protobuf
    sidecars. An identical rerun wrote zero and duplicate-rejected all 13;
    corruption is rejected.

25. **Network and authority counters** — Network requests, transports,
    credential reads, prospective reads/openings, historical-test reads,
    future-evaluation reads, reserve reads, active-model changes, Chair
    decisions, votes, rewards, penalties, voice changes, cooldowns, promotions,
    quarantines, and executions are zero. Active committee count is three.
    Network and authority flags reject before local execution.

26. **Protected artifact validation** — All 92 protected files remain
    byte-identical after V4 execution and replay. The protected aggregate is
    unchanged at
    `2ec8a4dd60432170f0f026643ac789a04ab622e71fa08c9eaa3a5579f39d9cbb`;
    in-process protected-byte and active-state audits also passed.

27. **Files changed** — Implementation is focused in
    `src/model/momentum_raw_feature_v4.rs`, with only model export and CLI wiring
    in existing source. Documentation is limited to the three required Sprint
    80 documents. Six tracked files comprise the completed change; runtime
    Protobuf sidecars remain ignored and uncommitted.

28. **Complete verification** — Formatting and default/Metal workspace checks
    passed. Default tests passed with 655 library, 404 committee-core
    integration, and 12 workspace-control tests; Metal passed with 656, 404,
    and 12. Git diff checks passed. Every Rust command used one build job, and
    every test run used one test thread.

29. **Boundary audits** — Both required boundary audits passed.

30. **What was proven** — Exact prior-history closure, derived splitting,
    preregistration-before-inference, fresh deterministic training, role-aware
    qualification, contribution auditing, no-qualified roster gating, optional
    registration exclusions, Protobuf idempotency and corruption rejection,
    protected-state immutability, and zero-authority boundaries are enforced.

31. **What remains unproven** — No participant qualified on the current valid
    sample count. No model improvement, participant superiority, winner, future
    outcome, reward or penalty effect, promotion readiness, Chair learning, or
    live-trading readiness was proven. The frozen-Mamba closure says nothing
    about another encoder, evidence identity, or policy contract.

32. **Commit, push, and draft PR** — Implementation commit `4e136ed` is on
    `agent/sprint80-momentum-raw-feature-v4`. Publication and Draft PR metadata
    will be recorded after the verified report commit is pushed.

33. **Next Sprint recommendation** — Preserve V1 through V4 and the final
    reserve as immutable history. If a new experiment is authorized,
    preregister a separate evidence identity with enough policy-valid validation
    and reserve samples before training; do not weaken qualification, tune on
    this validation result, rank private metrics, or open protected evidence.
