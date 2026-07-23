# Restart Sprint 79 Report

1. **PR #10 review and merge** — Reviewed all seven changed files and the V1
   failure replay, V2 collapse audit, fresh split, preregistration, frozen
   encoder, three head-only variants, role-aware qualification, Protobuf
   replay, protected state, and safety boundaries. No blocking defect remained.
   PR #10 was marked ready and merged as
   `6dd5078beb8ba14d4afb32a485d511894ac566ee`; its remote feature branch was
   deleted. Commits `47c1602` and `50e3286` remain ancestors.

2. **Post-merge verification** — Synchronized `main` and passed complete
   default and Metal verification before V3 implementation. Default counts were
   575 library, 404 committee-core integration, and 12 workspace-control tests;
   Metal counts were 576, 404, and 12.

3. **Protected before-state** — The verified snapshot, migrated intent/view,
   V1 artifacts, V2 artifacts, prospective outcomes, reward eligibility, V0
   aggregate, and active three-agent state were frozen. The protected byte
   aggregate was
   `54950d2e034a698a241d47494ab53a937525e136d8ff3d9c4b500b4b218359ff`.

4. **Head-only repair exhaustion** — V1 remains
   `RejectedProbabilityCollapse`; all three V2 learned participants remain
   `RejectedProbabilityCollapse`, while only Linear and Constant qualified by
   role. No V2 roster or future registration exists. V3 explicitly records
   `V2HeadOnlyRepairExhausted` and forbids another head-control sweep.

5. **Representation probe audit** — The audit used only V2-consumed training
   and validation evidence and did not access V3 fresh validation. Audit digest:
   `190d01aaf87681a9`.

6. **Raw-feature probe** — Status is `NonCollapsedPrediction`; representation
   diagnostic digest is `3803b764eb6cbf30`.

7. **Last-output probe** — Status is `SingleSidedPrediction`; representation
   diagnostic digest is `69646c99b529112f`.

8. **Mean-output probe** — Status is `SingleSidedPrediction`; representation
   diagnostic digest is `8bea1baa802dbd81`.

9. **Last+Mean probe** — Status is `LowEffectiveRank`; representation
   diagnostic digest is `bc35b8d2d745fc2c`. All four representation
   diagnostic identities are distinct.

10. **V3 split and untouched reserve** — The derived layout is training
    `[0,224)`, purge `[224,240)`, fresh validation `[240,264)`, and untouched
    final reserve `[264,312)`. Split digest: `44828a88d5ae2c11`; final-reserve
    identity: `9b1b9f400b429b83`. All overlap counts are zero.

11. **Representation preregistration** — Audit, split, and exact route
    registration `6250b81ff35d8dd8` were atomically persisted and reopened
    before fresh-validation inference. Maximum variants is four.

12. **Four frozen learned routes** — Last, Mean, Last+Mean, and
    Last-plus-raw-residual are the complete preregistered set. No result-driven
    fifth route was added. The encoder is frozen and the head hyperparameters
    are fixed before results.

13. **Fresh-validation execution** — Qualification selected exactly the 24
    labels in `[240,264)` while allowing purge context for feature and sequence
    construction. Every route used the same validation timestamp digest, zero
    validation updates, training-only normalizers, and no final-reserve row.

14. **Mamba-only qualifications** — Last, Mean, and Last+Mean were each
    `RejectedRepresentationInvariant`. No Mamba-only route qualified.

15. **Residual qualification** — Last-plus-raw-residual was
    `RejectedRepresentationInvariant`. Its contribution classification could
    not override the failed base qualification.

16. **Mamba contribution audit** — The residual block-zero audit was
    deterministic and classified the Mamba block as `MaterialContribution`.
    Separate Mamba/raw block and prediction identities were persisted; private
    effect values were not published.

17. **Linear and Constant qualifications** — `LinearMomentumBaselineV3` was
    `ComparatorQualified`; `ConstantProbabilityBaselineV3` was
    `BenchmarkQualified`. Both used the V3 training and validation ranges.

18. **V3 family** — Family `afc3aa14cc1622da` retains six participants, six
    qualification receipts, and four learned-route contribution audits. It
    selected no winner, read no historical test, and has no active, promotion,
    or reward eligibility.

19. **Genuine Mamba count** — Zero genuine Mamba participants qualified.

20. **Raw fallback count** — Zero participants qualified as
    `QualifiedRawFallbackNotMamba`.

21. **Representation route decision** — The decision is
    `AllRepresentationRoutesCollapsed`, digest `106e0e00dd7506f9`. Further
    head-only repair and further frozen-representation sweeps are forbidden for
    this path.

22. **Future roster** — Status is
    `FrozenMambaRepresentationPathRejected`; no roster artifact exists.
    Comparator-only registration was rejected.

23. **Evaluation registration** — Status is
    `FrozenMambaRepresentationPathRejected`; no evaluation registration
    artifact exists and no future evidence was acquired.

24. **Exclusions and minimum timestamp** — The implemented optional contract
    binds source boundary, four protected timestamps, provider finality,
    protected registrations, previous validation/reserve identities, hidden
    labels/probabilities, one request, one concurrency, and zero retries. The
    roster gate failed, so no minimum accepted timestamp was assigned.

25. **Cycle/Risk blocker** — The independent blocker remains
    `ProviderContractUnverified`; Momentum evidence was not repurposed.

26. **Value/Quality blocker** — The independent blocker remains
    `TrainerUnavailable`; no generic trainer or family was created.

27. **Prospective attribution replay** — The persisted opening remains
    `Opened`, with one attempt and two events. Momentum remains
    `MissedMaterialOpportunity`; Cycle/Risk remains `CorrectUncertainty`. Replay
    matches the persisted outcomes.

28. **Reward eligibility** — Both existing outcomes remain
    `IneligibleMinimumSamples`. Reward and penalty applications are zero;
    voice, cooldown, promotion, quarantine, and authority actions are also zero.

29. **Protobuf persistence and replay** — Twelve hand-written Protobuf
    categories are implemented. The completed terminal state has 26 applicable
    sidecars. The verified rerun wrote zero and duplicate-rejected all 26;
    corruption is rejected.

30. **Network and authority counters** — Network, transport, credential,
    prospective-row, prospective-label, historical-test, future-evaluation,
    active-model, Chair, vote, reward, penalty, voice, cooldown, promotion,
    quarantine, and execution counts are zero. Active committee count is three.
    Network and authority flags reject before local execution.

31. **Protected artifact validation** — The protected byte aggregate after
    execution and replay remained
    `54950d2e034a698a241d47494ab53a937525e136d8ff3d9c4b500b4b218359ff`.
    In-process protected-byte and active-state audits also passed.

32. **Files changed** — The focused implementation is
    `src/model/momentum_mamba_representation.rs`; existing repair, model export,
    and CLI files contain only required reuse and wiring. Documentation is
    limited to the three approved Sprint 79 learning-session files. Runtime
    Protobuf sidecars remain ignored and uncommitted.

33. **Complete verification** — Formatting and default/Metal workspace checks
    passed. Default tests passed with 615 library, 404 committee-core
    integration, and 12 workspace-control tests; Metal passed with 616, 404,
    and 12. Git diff checks passed. Every Rust command used one build job, and
    every test run used one test thread.

34. **Boundary audits** — Both required boundary audits passed.

35. **What was proven** — The frozen failure chain can be reopened exactly;
    probe and route registration precede fresh inference; V3 uses its label
    range without touching the final reserve; contribution, role, roster,
    Protobuf, idempotency, protected-byte, and zero-authority contracts are
    enforced.

36. **What remains unproven** — No frozen-Mamba route qualified. No model
    improvement, participant superiority, winner, future outcome, promotion
    readiness, reward effectiveness, Chair learning, or trading readiness was
    proven.

37. **Commit, push, and draft PR** — Commits `e224b9a`, `af60eea`, and
    `5b92c24` were pushed on `agent/sprint79-mamba-representation-path`.
    Draft PR [#11](https://github.com/seoyd/soma/pull/11) targets `main`; this
    final PR metadata is recorded in a follow-up report-only commit.

38. **Next architecture recommendation** — Preserve V1, V2, and V3 as terminal
    frozen-Mamba evidence. If another experiment is authorized, preregister a
    non-Mamba representation architecture with a new untouched evidence split;
    do not reuse `[240,264)`, weaken qualification, rank private metrics, open
    historical-test evidence, or register comparators alone.
