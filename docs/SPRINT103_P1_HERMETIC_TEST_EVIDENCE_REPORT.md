# SOMA Sprint 103-P1-R4 Work Report

## 1. Mode

- MODE: IMPROVE_AND_VERIFY
- SCOPE: DOCUMENTATION_ONLY
- BRANCH: `agent/sprint103-prereq-hermetic-qualified-six-tests-v1`.
- STARTING HEAD: `d35bbd5c0451c1bed4764e091054934021e45080`.
- COMMIT: 수행하지 않음
- PUSH: 수행하지 않음
- PR ACTION: 없음

The runtime evidence below was produced during R3, when all Cargo commands were
executed one at a time with `CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and fresh
external Cargo targets. R4 is a documentation-only contract correction and did
not rerun runtime verification.

## 2. Failure Reproduction

- clean worktree: reproduced from the exact starting HEAD before the R3 edit.
- external Cargo target: yes; the worktree-local build-output parent was absent.
- integration target: `minimal_ai_committee_core`.
- exact failed tests:
  - `automated_news_intake_normalizes_local_fixture_without_network`;
  - `autonomous_paper_config_rejects_remote_and_unsafe_fields`;
  - `autonomous_paper_loop_runs_cycles_attention_queue_and_archive_safely`.
- missing parent paths: the three tests attempted their first fixture/config write
  beneath worktree-relative build-output paths whose parents had not been created.
- root causes: fixture setup depended on a worktree-local Cargo target or residue
  from an earlier test. With an external Cargo target, all three first writes
  returned `NotFound`. This was a fixture lifecycle defect, not a production
  semantic failure.

The authoritative R3 reproduction result was 401 passed and 3 failed. No retry
was used to replace that result. R4 did not reproduce the failure again.

## 3. Scope

- R3 integration code change: `tests/minimal_ai_committee_core.rs`, preserved
  without modification during R4.
- R4 changed file: this existing report only.
- production files changed during R4: none.
- R1 code identity preserved: yes. The combined binary diff digest of the six R1
  code files remained
  `50b226aea89bc638853acbea5ffb5f5b9ad76750c4d90e792d40909064a61104`.
- R3 test-helper identity preserved during R4: yes.
- new files created during R4: none.

No source file, dependency, placeholder directory, or ignored fixture file was
added. Existing R1/R2/R3 tracked changes were preserved.

## 4. Fixture Lifecycle Repair

The following lifecycle repair was implemented in R3 and was not modified in R4.

- workspace allocator: a test-local `IntegrationFixtureWorkspace` was added to the
  existing integration test file. It uses a named 128-attempt bound and explicit
  errors on exhaustion.
- exclusive root: each candidate is an exact direct child of the captured system
  temporary directory and is acquired with `fs::create_dir`; `AlreadyExists`
  candidates are skipped without reuse, mutation, or deletion.
- parent preparation: `prepare_parent` and `prepare_directory` create only paths
  resolved below the exclusively owned root, before the first write.
- relative-path validation: empty paths, absolute paths, missing filenames,
  `ParentDir`, `CurDir`, root, and platform-prefix components are rejected.
- ownership marker: `.soma-integration-fixture-owner` is created with
  `create_new(true)`, receives a process/sequence/label ownership token, and is
  written, flushed, and synced before publication.
- cleanup: recursive deletion occurs only after proving the root is the expected
  direct temporary child, is a real non-symlink directory, and contains the exact
  regular non-symlink marker with the exact token. Consuming cleanup reports
  errors; `Drop` performs the same checks best-effort.
- stale candidate behavior: stale roots are left byte-for-byte untouched and a
  later candidate is allocated. Marker mismatch leaves the root untouched; the
  regression test restores only its own marker before final cleanup.

Each repaired fixture uses a separate workspace. No fixture shares filesystem
state with another fixture or relies on a prior run.

## 5. Assertion Preservation

### Fixture 1 — autonomous loop

- test: `autonomous_paper_loop_runs_cycles_attention_queue_and_archive_safely`.
- original purpose: prove deterministic autonomous paper-loop cycles, attention
  queues, archives, watchlist rechecks, fixed-count behavior, memory progression,
  and no broker/order/account, model-training, or live-inference authority.
- preserved assertions: the first and second runs are equal; cycle, queue, archive,
  triage, recheck, paper-only, safety, fixed-count, and member-state assertions are
  unchanged.
- path lifecycle-only change: isolated config, output directory, and fixed config
  use one fixture-specific owned root. The owned output child is reset between the
  two deterministic runs.
- result: all original assertions pass; both config files are verified and the
  workspace is removed after cleanup.

### Fixture 2 — unsafe configuration rejection

- test: `autonomous_paper_config_rejects_remote_and_unsafe_fields`.
- original purpose: reject remote autonomous input and reject an unsafe config
  field.
- preserved assertions: remote market-data validation fails with the local-path
  reason; parsing the unsafe config fails with the unsafe-field reason.
- path lifecycle-only change: the unsafe config lives below a distinct owned
  workspace and its nested parent is prepared before writing.
- result: all original rejection assertions pass; the file is verified and the
  owned root is absent after cleanup.

### Fixture 3 — news acquisition fixture

- test: `automated_news_intake_normalizes_local_fixture_without_network`.
- original purpose: normalize a local news fixture without network access and
  enforce item capping and safe path/source policy.
- preserved assertions: two items collected, one snapshot after capping, expected
  symbol and positive sentiment, explicit no-network safety note, deterministic
  two-item conversion, remote domain rejection, and traversal rejection.
- path lifecycle-only change: the fixture writes below its own workspace after
  asserting that the parent is initially absent and preparing it explicitly.
- result: all original assertions pass; the written file is verified and the owned
  root is absent after explicit cleanup.

## 6. Focused Verification

These are preserved R3 results; R4 did not rerun them.

- exact test 1: news intake test, 1 passed.
- exact test 2: unsafe/remote config rejection test, 1 passed.
- exact test 3: autonomous paper-loop test, 1 passed.
- integration target: 411 passed, 0 failed.
- timeout target: 12 passed, 0 failed.
- allocator/path regressions: 7 passed, covering absent parents, uniqueness, stale
  candidates, traversal/absolute/curdir rejection, marker mismatch, repeated
  execution, and internal eight-thread allocation.
- external-target run: all focused commands used a fresh external Cargo target.
- parallel run: the seven workspace regressions passed with the default test
  thread setting; the Qualified-Six focused set also passed 116 tests with the
  default test thread setting.

## 7. Baseline Residue

These baseline measurements were produced during R3 and were not remeasured in
R4.

- baseline commit: `d35bbd5c0451c1bed4764e091054934021e45080`.
- Default library: 1,281 passed.
- integration: the authoritative target stopped at 401 passed and 3 failed for the
  reproduced missing-parent defects.
- regular files: 110 at that interruption point. Because Cargo did not reach the
  later existing timeout target, that target was run once in a separate fresh
  starting-HEAD worktree; it produced the same additional 612 pre-existing test
  outputs seen after repaired full completion. Matched baseline coverage is 722.
- symlinks: 0.
- non-empty directories: 7 at the interruption point; 32 under matched baseline
  coverage.
- empty directories: 2 under both interrupted and matched baseline coverage.
- classification: `BaselinePreExistingResidueUnchanged`. The supplemental target
  itself passed 12 tests and changed no source.

The supplemental baseline is reported separately because calling the interrupted
baseline a complete residue baseline would be misleading.

## 8. Clean Run 1

This is preserved R3 runtime evidence.

- Default library: 1,304 passed, 0 failed.
- Metal library: 1,305 passed, 0 failed.
- integration: complete pass; `minimal_ai_committee_core` reported 411 passed and
  the existing timeout target reported 12 passed.
- timeout: exact target rerun, 12 passed.
- parallel focused: Qualified-Six 116 passed; integration workspace 7 passed.
- integration fixture roots: 0 remaining; ownership markers: 0 remaining.
- new repository residue: 722 regular files, 32 non-empty directories, 2 empty
  directories, and 0 symlinks, exactly matching the starting-HEAD matched-command
  baseline. Increment versus that baseline: 0 in every class.

Formatting, Default check, and Metal check also passed before the Run 1 test
sequence.

## 9. Clean Run 2

This is preserved R3 runtime evidence.

- Default library: 1,304 passed, 0 failed.
- Metal library: 1,305 passed, 0 failed.
- integration: complete pass; `minimal_ai_committee_core` reported 411 passed and
  the existing timeout target reported 12 passed.
- integration fixture roots: 0 remaining; ownership markers: 0 remaining.
- new repository residue: 722 regular files, 32 non-empty directories, 2 empty
  directories, and 0 symlinks, exactly matching the matched-command baseline.
- result equal to Run 1: yes. Pass/fail results, warning class, repair-owned
  residue, and all repository-relative residue path/type sets matched.

Run 1's worktree and external Cargo target were removed before Run 2. Run 2 used a
new worktree and a new external Cargo target and copied no Run 1 filesystem state.

## 10. Differential Residue

- owned roots remaining: 0 integration workspace roots; 0 Qualified-Six or
  acquisition test-lease roots attributable to these runs.
- ownership markers remaining: 0.
- new files versus baseline: 0 with matched command coverage.
- new symlinks versus baseline: 0.
- new non-empty directories versus baseline: 0.
- new empty directories versus baseline: 0.
- pre-existing residue: the same 722 regular files, 32 non-empty directories, and
  2 empty directories are produced by starting HEAD under matched command coverage.
- strict zero-residue claimed: no. The correct verdict is unchanged pre-existing
  test residue plus zero R3-owned residue and zero differential repository residue.

No workspace parent created by any repaired fixture remained after cleanup.

## 11. Full Verification

- full runtime verification provenance: R3.
- R4 runtime rerun: 수행하지 않음 — documentation-only correction.

- cargo fmt: pass.
- Default check: pass.
- Metal check: pass with `backend-metal`.
- Default test count: 1,304 passed in each clean run.
- Metal test count: 1,305 passed in each clean run.
- integration count: full command passed in both runs; repaired integration target
  411 passed.
- timeout count: 12 passed in the full integration runs and in the required exact
  Run 1 execution.
- focused count: Qualified-Six 116 passed; integration workspace regressions 7
  passed; each of the three repaired fixtures passed individually.
- R3 git diff --check: pass.
- R4 documentation-only checks: `git diff --check`, 16-section structure, MODE,
  Final Status, and Exactly One Next Step checks passed; no runtime test was run.
- warnings: four pre-existing dead-code warnings in both check configurations;
  new warnings introduced by R3: 0.

Verification-harness notes: an initial Run 1 output session could not be recovered
and was not counted. The first logged Default rerun reported all 1,304 tests as
passing, but its missing log directory made the shell wrapper exit nonzero; after
creating that directory, the exact command passed again with exit code 0. One
Metal attempt named the nonexistent `metal` feature and Cargo rejected it before
building or testing; the required `backend-metal` command then passed. No source
change occurred between these harness corrections and the recorded passing runs.

## 12. Documentation

- R2 failure recorded: yes; the exact three failures and their missing-parent cause
  are retained above.
- R3 repair recorded: yes; ownership, path validation, cleanup, assertion
  preservation, verification, and residue results are recorded.
- R4 report contract correction recorded: yes.
- concrete external user filenames present: 0.
- external instruction references present: 0.
- actual results only: yes. The initial failure, baseline interruption, supplemental
  baseline measurement, and every completed verification are distinguished.
- 16-section structure verified: yes.

## 13. Safety

- external user content read: 0.
- external user content touched: 0.
- market network: 0 requests and 0 downloads.
- live: 0 live requests and 0 live inference.
- holdout: 0 sealed holdout reads.
- real model operations: 0 real fits, predictions, or target reveals.
- real predictions: 0.
- trading: 0 paper trades, live trades, orders, or account access.
- Chair execution, committee vote, reward mutation, and penalty mutation: 0.
- production source mutation during R4: 0.
- test source mutation during R4: 0.
- PR #35 mutation: none.
- PR #36 remote mutation: none.
- Sprint 104 changes: none.

No automatic promotion or Formula mutation occurred.

## 14. Remaining Risks

R3는 확인된 세 fixture의 경로 생명주기 문제를 제한적으로 교정했다.
이번 결과만으로 integration test 파일 전체가 완전히 hermetic하다고 증명된
것은 아니다.

Read-only inspection of the tracked integration test confirms that existing tests
still use repository-relative shared `target` paths:

- `watchlist_recheck_direct_cycle_loads_local_paths` creates the shared `target`
  parent and writes its market-data and news fixtures directly beneath it.
- `owner_intent_policy_table_loads_prioritizes_and_rejects_safely` writes JSON and
  TOML fixtures directly beneath the shared parent without acquiring an exclusive
  test-owned workspace.
- `news_provider_layer_collects_local_and_defers_remote_safely` likewise writes
  local news fixtures directly beneath the shared parent without independently
  owning that parent lifecycle.

These tests were not observed failing in the completed R3 integration run, so this
section does not claim a current failure or a production defect. It records that
some existing tests may not independently own their parent lifecycle and that the
entire integration target's execution-order independence and full hermeticity have
not yet been separately proven. This limitation does not invalidate the passing R3
result for the three repaired fixtures. A complete integration-fixture inventory
and broader hermeticization remain a separate follow-up scope.

## 15. Final Status

- READY_FOR_REVIEW

## 16. Exactly One Next Step

- 수정된 보고서와 전체 8-file local diff에 대해 한 번의 독립적인
  review-only 검토를 수행한다.
