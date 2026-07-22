# Restart Sprint 78 Report

1. **PR #9 review and merge** — Reviewed all six changed files, authoritative
   field provenance, policy compatibility, normal intent validation, ordinary
   view construction, five Protobuf sidecars, idempotent replay, V1 family and
   qualification separation, reward replay, and safety counters. No blocking
   defect was found. PR #9 was marked ready and merged with merge commit
   `2fa70929e1d896db63fd60196875b03eac858f4f`; its remote feature branch was
   deleted. Commits `b252634`, `95484c9`, and `1d1c897` remain ancestors.

2. **Post-merge verification** — Synchronized `main` and passed complete
   default and Metal verification before implementation. Counts were 546
   library, 404 committee-core integration, and 12 workspace-control tests by
   default; Metal had 547, 404, and 12.

3. **Protected before-state** — The migrated intent, migrated view, policy
   proof, migration proof, migration journal, 312-row snapshot, V1 session,
   projection, three participants, three qualification receipts, family, usage
   ledger, blocked registrations, prospective opening/outcomes, V0 aggregate,
   and active three-agent state were frozen. The protected byte aggregate was
   `8d32a599223ae365e59782cc525b66e58688676d7c63733ef3114ac760d50786`.

4. **Failed participant identity** — The failed participant is exactly
   `FrozenMambaHeadV1`, digest `22632d7a5f0e1ab2`, in source family
   `72cd657ea8a1f039`; its receipt remains `RejectedProbabilityCollapse`.

5. **Collapse root cause** — The machine-derived audit classified the failure
   as `ProbabilitySingleSided`. Audit digest: `5f047849bef0a026`.

6. **Representation diagnostics** — Finite status, per-dimension variance,
   constant dimensions, aggregate variance, approximate rank, unique count, and
   normalization status were derived separately. They did not become the
   collapse root cause. Digest: `2ab09dee0d95ceb7`.

7. **Head optimization diagnostics** — Initial/final identities, finite state,
   parameter-delta and gradient classes, update count, schedule, loss trajectory,
   and stop reason were separated from representation and probability evidence.
   They did not become the collapse root cause. Digest: `11a514c7bdb9f5ac`.

8. **Probability subtype** — The preserved subtype is
   `ProbabilitySingleSided`. Probability diagnostic digest:
   `a6230ff281c5def5`.

9. **Repair capability** — The derived capability is
   `RepairableWithBoundedHeadRegularization`; no encoder backpropagation,
   architecture replacement, or unsupported repair was introduced.

10. **Fresh repair split** — From prior unused `[96, 312)`, V2 uses training
    `[0, 160)`, purge `[160, 176)`, fresh validation `[176, 200)`, and remaining
    reserved `[200, 312)`. Split digest: `e9029f489e01a87b`. Prior-validation,
    prospective, and future-evaluation overlap counts are zero.

11. **Repair preregistration** — Registration `68eb18d97c06f4f3` was atomically
    persisted and reopened before fresh inference. Its cap is four; three
    concrete variants were frozen.

12. **Registered variants** — `v1-control` (`79f4d70dfbed9760`),
    `l2-regularized` (`fe7dca3d5adc4081`), and `low-rate-l2`
    (`c3ed36386e8fc092`) use a frozen encoder, fresh deterministic heads,
    training-only normalizers, no class weights, no V1 warm start, and no V1
    head reuse.

13. **Fresh-validation execution** — Every V2 Mamba variant plus fresh Linear
    and Constant used the exact same fresh validation timestamp identity.
    Normalizers fit repair training only; validation parameter updates were zero.

14. **Mamba qualification** — `v1-control`, `l2-regularized`, and
    `low-rate-l2` were each `RejectedProbabilityCollapse`. No status was
    hardcoded and no second result-dependent variant batch was added.

15. **Comparator qualification** — `LinearMomentumBaselineV2` was `Qualified`;
    `ConstantProbabilityBaselineV2` was `BenchmarkQualified` under the explicit
    constant-benchmark role.

16. **V2 family** — Family `fb7d3825c2ae8911` retains all five participants and
    all five receipts. It selected no winner, accessed no historical test, and
    is ineligible for active use, promotion, and reward.

17. **Qualified learned count** — Zero learned Mamba participants qualified.
    Two comparators qualified.

18. **Future roster** — Status is `NoQualifiedLearnedParticipant`; no roster
    artifact exists. Rejected variants remain in the family and audit.

19. **Evaluation registration** — Status is
    `NoQualifiedLearnedParticipant`; no future evaluation registration exists.
    Linear versus Constant was not registered alone.

20. **Exclusions and timestamp** — The source boundary, protected registration
    identities, four protected timestamps, provider-finality boundary, and all
    prior reserved-range identities remain enforced by the implemented V2
    contract. Because the roster gate failed, no minimum accepted timestamp was
    assigned.

21. **Cycle/Risk** — The independent blocker remains
    `ProviderContractUnverified`; Momentum evidence was not repurposed.

22. **Value/Quality** — The independent blocker remains `TrainerUnavailable`;
    no generic trainer or family was created.

23. **Prospective attribution replay** — The existing opening stayed `Opened`
    with one attempt and two events. Momentum remained
    `MissedMaterialOpportunity`; Cycle/Risk remained `CorrectUncertainty`. The
    replay exactly matched persisted outcomes.

24. **Reward eligibility** — Both agents remain `IneligibleMinimumSamples`.
    Reward, penalty, voice, cooldown, promotion, quarantine, and Chair actions
    remain zero.

25. **Protobuf persistence and replay** — Nine manual-Protobuf contracts are
    implemented: audit, split, registration, participant, qualification receipt,
    family, roster, future registration, and journal. The passing execution
    wrote 15 applicable sidecars; the repeated execution wrote zero and
    duplicate-rejected all 15. Corruption rejects.

26. **Network and authority counters** — Network, transport, credential,
    prospective-row, prospective-label, future-evaluation, historical-test,
    active-model, Chair, vote, reward, penalty, voice, cooldown, promotion,
    quarantine, and execution counts are zero. Active committee count is three.

27. **Protected artifact validation** — The protected byte aggregate after both
    executions remained
    `8d32a599223ae365e59782cc525b66e58688676d7c63733ef3114ac760d50786`.
    In-process protected-byte and active-state audits also passed.

28. **Files changed** — The focused implementation is in
    `src/model/momentum_mamba_repair.rs`; existing `src/model/mod.rs`,
    `src/model/agent_learning_session.rs`, and `src/cli.rs` contain only module,
    shared atomic-write, export, and CLI wiring. Documentation is limited to the
    three approved learning-session and Sprint 78 files. Runtime sidecars remain
    ignored and uncommitted.

29. **Complete verification** — Formatting and default/Metal workspace checks
    passed. Default tests passed with 575 library, 404 committee-core
    integration, and 12 workspace-control tests; Metal passed with 576, 404,
    and 12. Git diff checks passed. Every Rust command used one build job and
    one test thread.

30. **Boundary audits** — Both required boundary audits passed.

31. **What was proven** — The V1 collapse can be reproduced and classified
    without touching fresh evidence; split and registration precede fresh
    inference; roles, identities, Protobuf persistence, idempotency, protected
    bytes, and baselines-only registration rejection are enforced.

32. **What remains unproven** — No repaired Mamba passed qualification. No model
    improvement, participant superiority, winner, future outcome, promotion
    readiness, reward effectiveness, Chair learning, or trading readiness was
    proven.

33. **Commit, push, and draft PR** — Implementation commit `47c1602` was
    pushed to remote branch `agent/sprint78-mamba-collapse-repair`. Draft PR
    [#10](https://github.com/seoyd/soma/pull/10) targets `main`; the approved
    documentation is committed separately on the same branch.

34. **Next Sprint recommendation** — Preserve this failed bounded experiment and
    investigate an explicitly supported representation-path repair before
    defining another fresh split. Do not reuse `[176, 200)`, weaken the learned
    gate, rank private metrics, open historical-test evidence, or register
    baselines alone.
