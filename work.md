You are operating as a gstack-style engineering team for the project “Soma Zero”.

Current state:
Sprint 02~118 are complete.

Critical current problem:
The project is stuck in report/test expansion.
Sprint 118 added more reports, tests, CLI, docs, examples, and panels, but workspace acceptance is still blocked.

Current truth:
- cargo fmt --all --check passed.
- cargo check --workspace --quiet passed.
- cargo build --bin soma_experiment --quiet passed.
- focused Sprint 118 tests passed.
- representative CLI smoke passed.
- cargo test --workspace --no-run --quiet timed out.
- cargo test --workspace --quiet timed out.
- NoRunStillBlocked.
- FullWorkspaceStillBlocked.
- AcceptanceTruthReadyWithWarnings.
- can_claim_full_acceptance=false.
- ConsolidationStillPaused.
- FifthPatchStillNotApplied.
- Runtime/training/live/broker/order/account remain deferred.

Strategic decision:
Stop adding new report layers.
Stop adding new Sprint-specific integration tests.
Stop adding new CLI surfaces.
Stop adding new docs/examples/fixtures unless absolutely required to remove or consolidate existing surfaces.
The next sprint must reduce the test surface and recover workspace no-run.

Sprint 119 objective:
Freeze report growth, freeze CLI/docs/example growth, reduce integration-test binary count, consolidate duplicate Sprint timeout/acceptance/report tests, preserve assertions, preserve safety sentinels, and make cargo test --workspace --no-run --quiet finish if possible.

This sprint is not about adding features.
This sprint is not about adding reports.
This sprint is not about adding diagnostic panels.
This sprint is not about adding more acceptance gates.
This sprint is about reducing what already exists.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 119:
Report Freeze + Test Surface Reduction + Workspace No-Run Pass

────────────────────────────────────────
1. HARD RULES

1. Do not add new report structs.
2. Do not add new report files.
3. Do not add new Control Tower panels.
4. Do not add new CLI commands.
5. Do not add new example TOMLs.
6. Do not add new fixture directories.
7. Do not add new docs except one short Sprint119 summary if needed.
8. Do not create a new giant Sprint bundle.
9. Do not create another timeout evidence matrix.
10. Do not create another acceptance gate version.
11. Do not create another V20/V21/V22 report family.
12. Do not add another focused Sprint test family unless it replaces multiple existing tests.
13. Do not increase the number of integration test binaries.
14. Do not delete assertions silently.
15. Do not delete safety tests.
16. Do not introduce hidden skips.
17. Do not weaken CLI safety.
18. Do not weaken determinism checks.
19. Do not weaken acceptance truth.
20. Do not claim full workspace acceptance unless cargo test --workspace --quiet finishes and passes.

Allowed work:
- Merge duplicate integration test targets.
- Move assertions into shared consolidated test targets.
- Delete retired duplicate test files only after assertion migration.
- Remove redundant Sprint-only smoke tests if equivalent CLI/help/safety coverage remains.
- Replace many narrow report tests with one consolidated workspace acceptance/reduction test.
- Delete duplicated fixtures if surviving fixtures cover the same assertions.
- Remove or stop wiring redundant example configs if no longer needed.
- Keep one minimal summary of what was removed/migrated.
- Run cargo test --workspace --no-run --quiet with a real timeout.
- Run cargo test --workspace --quiet only after no-run is recovered or if explicitly configured.

────────────────────────────────────────
2. ROLE STACK

Role 1: Product Chair
- Enforces report freeze.
- Blocks new feature/report/test expansion.

Role 2: Test Surface Reduction Architect
- Finds high-volume integration test targets.
- Merges duplicate tests.
- Reduces binary count.

Role 3: Assertion Preservation Architect
- Moves assertions before deleting test files.
- Produces a simple migration list, not a new report system.

Role 4: Safety Sentinel Architect
- Keeps safety tests, CLI safety, determinism, no-live/no-order/no-training guards.

Role 5: Workspace Acceptance Architect
- Focuses on cargo test --workspace --no-run --quiet completion.
- Keeps no-run distinct from full acceptance.

Role 6: Rust Cleanup Engineer
- Removes duplicate test files, duplicated fixtures, duplicated examples, and redundant CLI smoke surfaces.
- Does not add feature code.

Role 7: Verification Engineer
- Runs fmt/check/build/no-run.
- Confirms no hidden skips or assertion deletion.

────────────────────────────────────────
3. WORK PLAN

STEP 1 — Inventory

Create a short internal inventory:
- current integration test target count.
- largest duplicated Sprint timeout/acceptance test groups.
- duplicated Control Tower panel tests.
- duplicated acceptance truth gate tests.
- duplicated cargo JSON/timeout tests.
- duplicated fixture/example support files.
- safety sentinel tests that must not be touched.

Do not create a new report module for this.
Use a simple markdown or text note if needed:
docs/SPRINT119_TEST_SURFACE_REDUCTION.md

STEP 2 — Select reduction targets

Select only low-risk duplicate families:
- Sprint 111~118 timeout/acceptance tests.
- repeated acceptance truth gate tests.
- repeated cargo JSON diagnostic tests.
- repeated Control Tower read-only panel tests.
- repeated CLI smoke tests where help text coverage overlaps.
- repeated fixture/support modules.

Do not select:
- workspace CLI safety sentinel.
- workspace determinism sentinel.
- safety guard tests.
- Risk Governor veto tests.
- no-live/no-order/no-account tests.
- no-hidden-skip tests.

STEP 3 — Consolidate tests

For each selected duplicate group:
- choose one surviving consolidated test target.
- move assertions into it.
- delete the now-redundant narrow test file.
- keep assertion names/comments traceable.
- do not skip assertions.
- do not remove safety semantics.
- update mod/support wiring.

Target outcome:
- reduce integration test binary count by at least 10 if safe.
- if 10 is not safe, reduce by the maximum safe count and explain why.

STEP 4 — Reduce CLI smoke duplication

Do not add CLI commands.
Do not add new smoke files.

Reduce repeated CLI smoke by:
- keeping one representative CLI safety test.
- keeping one deterministic CLI help test.
- removing duplicated Sprint-specific help tests where identical safety text is already covered.
- preserving remote path rejection coverage.

STEP 5 — Reduce fixture/example duplication

Remove or consolidate duplicate:
- sprint timeout fixtures.
- acceptance truth fixtures.
- cargo JSON sample fixtures.
- repeated example TOMLs used only by retired tests.

Keep only fixtures needed by surviving tests.

STEP 6 — Verify no safety loss

Run targeted checks:
- cargo fmt --all
- cargo check --workspace
- cargo build --bin soma_experiment
- surviving consolidated test target
- CLI safety test
- determinism test
- no-hidden-skip/safety sentinel tests if present

STEP 7 — Workspace no-run attempt

Run:

cargo test --workspace --no-run --quiet

Rules:
- If it finishes and passes, mark NoRunRecovered.
- If it times out, report honest timeout.
- If it fails, fix real compile/test-build failures.
- Do not claim full workspace acceptance from no-run.

STEP 8 — Full workspace attempt

Only run:

cargo test --workspace --quiet

after no-run completes, or if explicitly configured.

Rules:
- FullWorkspaceAccepted only if finished and passed.
- Timeout is not pass.
- Focused tests are not full pass.
- CLI smoke is not full pass.
- cargo build is not full pass.

────────────────────────────────────────
4. WHAT TO REMOVE / MERGE FIRST

Prioritize likely duplicate families:

1. Sprint 111~118 timeout root-cause tests.
2. Sprint 111~118 acceptance truth gate tests.
3. Sprint 111~118 Control Tower timeout/acceptance panel tests.
4. Sprint 111~118 cargo JSON diagnostic tests.
5. Sprint 111~118 CLI safety duplicates.
6. Sprint 111~118 determinism duplicates.
7. Repeated `tests/support/sprintXXX_support.rs` files with near-identical helpers.
8. Repeated example TOMLs that only feed retired test targets.
9. Repeated fixture JSONs with same status combinations.

The goal is not to preserve every Sprint-specific test file.
The goal is to preserve the assertions.

────────────────────────────────────────
5. STRICT DEFER LIST

Do not implement:
- new AI core work,
- Mamba3Fin runtime,
- Gated DeltaNet runtime,
- model training,
- runtime LLM,
- live inference,
- live trading,
- broker/order/account APIs,
- Tauri/Svelte,
- dashboard serve,
- browser execution,
- new Control Tower feature panels,
- fifth patch,
- assertion movement for consolidation beyond test reduction,
- test target retirement without migrated assertions,
- new report families,
- new diagnostics,
- new examples unless replacing old ones.

────────────────────────────────────────
6. ACCEPTANCE CRITERIA

Sprint 119 is successful if:

- no new report family is added.
- no new CLI family is added.
- no new diagnostic bundle family is added.
- integration test target count decreases.
- retired test files have migrated assertions.
- no safety test is deleted.
- no hidden skip is introduced.
- cargo fmt --all passes.
- cargo check --workspace passes.
- cargo build --bin soma_experiment passes.
- consolidated focused tests pass.
- CLI safety still passes.
- determinism still passes.
- cargo test --workspace --no-run --quiet is attempted honestly.
- if no-run finishes, report NoRunRecovered.
- if no-run times out, report timeout honestly and show reduced target count.
- full workspace acceptance is claimed only if cargo test --workspace --quiet finishes and passes.

────────────────────────────────────────
7. FINAL RESPONSE FORMAT

When done, respond with:

## 1. Sprint summary

## 2. Files removed

## 3. Files merged

## 4. Files changed

## 5. Assertions migrated

## 6. Test targets retired

## 7. Surviving consolidated test targets

## 8. Fixture/example reductions

## 9. CLI smoke reduction

## 10. Safety sentinels preserved

## 11. Determinism preserved

## 12. Target-count before/after

## 13. cargo fmt/check/build results

## 14. Focused consolidated test results

## 15. cargo test --workspace --no-run result

## 16. cargo test --workspace result, only if run

## 17. No-run recovery status

## 18. Full workspace acceptance status

## 19. Remaining blockers

## 20. Next recommendation

Do not add a 60-section report.
Do not create another report bundle.
Do not create another diagnostic queue.
Do not over-explain.
Be direct.