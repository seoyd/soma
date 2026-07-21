# Sprint 73 Restart Report

1. **PR #4 scope review.** Reviewed the complete seven-file change set,
   candidate construction, dataset materialization, Protobuf schemas, CLI,
   persistence, repeat execution, and active/prospective guards.

2. **Complete-view binding defect.** Removed the production largest-row-count
   shortcut and replacement one-dataset policy. Sessions now consume a verified
   persisted view or the canonical planner's complete semantically resolved
   view.

3. **Trainer projection hardening.** Added digest-bound explicit projections.
   Momentum consumes one supported price series, Cycle/Risk consumes one
   supported market-index series, extra authorized evidence remains referenced,
   and Value/Quality receives no borrowed adapter.

4. **PR #4 merge result.** Hardening commit `b8beb4225aaebbaf04bdd633a22954fa1c36287e`
   was merged through PR #4 as merge commit
   `7a089278438c3ad51e86064758f1d30f3411aa5b`; the remote feature branch was
   deleted and local `main` synchronized.

5. **Protected preliminary artifacts.** Existing registry, session, dataset,
   candidate, and journal files remained byte-identical. Hardened identities are
   digest-addressed and preliminary candidates are classified as superseded
   retrospective records, never active candidates.

6. **Candidate evidence-usage ledgers.** Added semantic, append-only ledgers for
   Momentum and Cycle/Risk covering binding, projection, feature/label work,
   normalization, training, validation, checkpoint selection, historical test,
   candidate identity, and unused referenced evidence.

7. **Momentum historical-test status.** The local candidate is
   `InfluencedCandidateIdentity`; its historical test cannot be claimed fresh.

8. **Cycle/Risk historical-test status.** The local candidate is
   `InfluencedCandidateIdentity`; its historical test cannot be claimed fresh.

9. **Value unavailable status.** Value/Quality remains trainer-unavailable with
   no candidate, audit, or evaluation registration.

10. **Candidate identity audits.** Additive audits identify model and metric
    identity inputs, test evidence in identity, freshness eligibility, future
    registration eligibility, and supersession by input-binding hardening. The
    original envelopes were not rewritten.

11. **Future evaluation cutoffs.** Cutoffs derive from stored lineage and any
    protected prospective boundary, never a hardcoded date. The current derived
    exclusive cutoff for both candidates is `1784073600000`; only later
    timestamps may be admitted by a valid registration.

12. **Comparator registrations.** Comparator sets are frozen before future
    data. Momentum retains its one actual parent comparator; Cycle/Risk has no
    represented comparator; no comparator was invented.

13. **Evaluation registration results.** Both current preliminary candidates
    are `PolicyInvalid` because their persisted sessions predate hardened source
    policy and projection binding. Value/Quality is `CandidateUnavailable`. No
    future evaluation was executed.

14. **Protobuf persistence and reopen.** Projection, ledger, identity audit,
    registration, and journal use manual Protobuf, semantic digests, verified
    temporary writes, atomic rename, and final reopen. A repeated registration
    rejected all ten artifacts as duplicates with zero storage failures.

15. **No-promotion boundary.** No active replacement, voice or speaking-right
    change, vote, winner selection, promotion, reward, penalty, or execution was
    added or performed.

16. **Prospective-lane freeze.** Prospective requests, row reads, label reads,
    and mutations remained zero. Evaluation artifacts were isolated under the
    ignored candidate-evaluation namespace.

17. **Network and authority counters.** Active committee count remained three;
    network, credential, active-model, Chair, vote, reward, penalty, voice,
    promotion, and execution counters all remained zero.

18. **Files changed.** Phase B/C changes are limited to
    `src/model/agent_learning_session.rs`, `src/model/mod.rs`, `src/cli.rs`,
    `docs/AGENT_PRIVATE_LEARNING_SESSION.md`,
    `docs/CANDIDATE_EVIDENCE_USAGE_AND_EVALUATION.md`, and this report.

19. **Complete verification.** Formatting plus default and Metal workspace checks
    passed. Default single-threaded suites passed `447 + 404 + 12` tests and
    Metal suites passed `448 + 404 + 12` tests. Git diff checks passed without
    whitespace errors.

20. **Boundary audits.** Both boundary audits passed.

21. **What was proven.** Candidate lineage can be verified without retraining;
    retrospective usage is machine-verifiable; historical tests influenced both
    candidate identities; preliminary files are preserved; and a future-only
    evaluation contract fails closed when policy or comparator prerequisites are
    absent.

22. **What remains unproven.** Candidate improvement, future performance,
    historical-test freshness, promotion readiness, reward eligibility, Chair
    learning, and trading readiness remain unproven.

23. **Commit, push, and draft PR.** Phase B/C uses branch
    `agent/sprint73-candidate-evaluation-prereg` and commit title
    `Pre-register private candidate evaluations`. It is published only as a
    draft pull request; registration and review remain separate from execution.

24. **Next Sprint recommendation.** Produce hardened V1 candidates from complete
    canonical views, validate or explicitly preregister eligible comparators,
    then collect only finalized evidence strictly after each frozen cutoff under
    the one-time-opening protocol. Keep performance evaluation and any promotion
    decision in later, separately authorized work.
