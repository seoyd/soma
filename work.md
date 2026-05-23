You are operating as a gstack-style engineering team for Soma Zero.

Current state:
Sprint 138 is complete.

Sprint 138 result:
- Cargo.toml now uses autotests = false.
- Active workspace integration test targets are explicitly listed.
- Active test targets:
  - minimal_ai_committee_core
  - workspace_timeout_reduction_queue
- Integration test files reduced:
  - around 1193/1149 previous top-level files
  - around 907 remaining top-level files
- 242 legacy sprint/report/diagnostic/gate/panel/timeout tests deleted.
- cargo test --workspace --no-run --quiet now exits 0.
- NoRunRecovered can be claimed for the explicit manifest target set.
- cargo test --workspace --quiet has not yet been run as the final full acceptance check.
- Full workspace acceptance is not yet claimable until cargo test --workspace --quiet finishes and passes.

Important interpretation:
No-run recovery was achieved by reducing active Cargo test targets.
This is acceptable only if:
- essential product assertions are preserved.
- safety sentinels are preserved.
- old legacy tests are either duplicated, obsolete, or intentionally no longer active.
- explicit test manifest is treated as the new workspace test contract.

Sprint 139 objective:
Lock the explicit test manifest strategy, verify Sprint 138 deletion safety one final time, run full workspace test against the new explicit test target set, and prepare a clean commit for the no-run recovery changes.

This sprint must not add new product features.
This sprint must not add report bloat.
This sprint must not create new diagnostic gates.
This sprint must not re-enable broad test auto-discovery.
This sprint must not touch AI core unless fixing a compile/test regression.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 139:
Explicit Test Manifest Lock + Full Workspace Pass + No-Run Recovery Commit

────────────────────────────────────────
1. HARD RULES

Do not add:
- new product feature
- new AI member logic
- new report family
- new diagnostic matrix
- new Control Tower panel
- new CLI family
- new broad test suite
- new fixture family
- real Mamba3 runtime
- real Gated DeltaNet runtime
- model training
- live inference
- runtime LLM debate
- broker/order/account
- live trading
- dashboard/browser/Tauri/Svelte

Do not:
- re-enable autotests automatically
- add back hundreds of integration tests
- hide safety failures
- delete safety assertions
- delete determinism assertions
- delete no-order/no-account/no-training assertions
- delete full-acceptance truth assertions
- claim legacy-autodiscovery-wide acceptance
- claim full workspace acceptance before cargo test --workspace --quiet finishes and passes
- treat no-run as full test pass

Allowed:
- verify Cargo.toml explicit test manifest
- verify deleted legacy tests are safe to remain deleted
- restore any uncertain safety-critical test
- run full workspace test
- create one short docs/SPRINT139_EXPLICIT_TEST_MANIFEST.md if needed
- commit Sprint 138 no-run recovery changes after validation

Main rule:
Do not expand.
Lock and verify.

────────────────────────────────────────
2. TEST CONTRACT

The new workspace test contract is:

Cargo.toml:
- autotests = false
- explicit [[test]] targets only

Required explicit targets:
1. minimal_ai_committee_core
   Protects:
   - AI committee core
   - DataRouter
   - AiMemberBrain
   - OfflineMemberBrainAdapter
   - CoreAwareMemberBrainAdapter
   - Mamba3/Gated deferred contract
   - 3-member pilot
   - style cards
   - offline batch
   - owner summary
   - owner console
   - owner feedback
   - autonomous paper loop
   - attention inbox
   - watchlist recheck
   - no broker/order/account
   - no training/live inference
   - deterministic core behavior

2. workspace_timeout_reduction_queue
   Protects:
   - no-run/full-acceptance truth
   - timeout safety
   - cargo JSON diagnostic safety
   - CLI warning/forbidden command safety
   - read-only/control-tower legacy safety assertions if migrated
   - no false full-acceptance claim
   - determinism assertions migrated from old sprint tests

Do not add many explicit targets.
Only add a third explicit target if a safety assertion is clearly not covered by the two survivors.

────────────────────────────────────────
3. PHASE 1 — INSPECT CURRENT STATE

Run:

git status --short
git diff --name-status
git diff --stat
git diff -- Cargo.toml
find tests -maxdepth 1 -name "*.rs" | wc -l
cargo test --list --workspace

Confirm:
- autotests = false is present.
- only intended explicit integration test targets are active.
- 242 deleted files are visible in diff.
- no current AI core file was accidentally damaged.
- work.md is ignored as scratchpad.

Do not stage or commit yet.

────────────────────────────────────────
4. PHASE 2 — VERIFY DELETED TEST SAFETY

Review deleted test categories.

For every deleted group, classify briefly:

- SafeDeleteDuplicate
- SafeDeleteObsoleteReportOnly
- SafeDeleteObsoleteDiagnosticOnly
- RestoreSafetyCritical
- RestoreUncertain

Deletion can stay only if:
- assertion is covered by minimal_ai_committee_core, or
- assertion is covered by workspace_timeout_reduction_queue, or
- file only tested obsolete report formatting / old diagnostic bundle shape, or
- file was not part of product direction and not safety-critical.

Must restore if deleted test contained unique:
- broker/order/account guard
- live trading guard
- model training guard
- live inference guard
- Risk Governor veto assertion
- acceptance truth assertion
- determinism assertion
- hidden skip guard
- local-only/remote path rejection guard

No vague phrase “covered elsewhere.”
Name the survivor:
- tests/minimal_ai_committee_core.rs
- tests/workspace_timeout_reduction_queue.rs

────────────────────────────────────────
5. PHASE 3 — VERIFY EXPLICIT TEST TARGETS

Run:

cargo fmt --all
cargo fmt --all --check
cargo check --workspace
cargo build --bin soma_experiment
cargo test --test minimal_ai_committee_core --quiet
cargo test --test workspace_timeout_reduction_queue --quiet

Required:
- both explicit tests pass.
- no ignored/hidden skip is introduced.
- no safety warning disappears.
- CLI smoke still works.

Run CLI smoke:

cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml

────────────────────────────────────────
6. PHASE 4 — VERIFY NO-RUN RECOVERY

Run:

cargo test --workspace --no-run --quiet

Expected:
- should finish and pass now.

If pass:
- mark NoRunRecovered.
- record that recovery applies to explicit manifest target set.

If fail:
- fix compile/no-run failure.
- do not reintroduce broad auto tests.
- rerun.

If timeout:
- report timeout honestly.
- do not claim recovery.
- inspect active target list again.

────────────────────────────────────────
7. PHASE 5 — FULL WORKSPACE TEST

Now that no-run has recovered, run:

cargo test --workspace --quiet

Rules:
- FullWorkspaceAccepted only if this finishes and passes.
- If it times out, report timeout honestly.
- If it fails, fix real failure.
- Do not claim full acceptance from focused tests.
- Do not claim full acceptance from no-run.

Expected:
Because active test targets are now capped, this should be much more likely to finish.

────────────────────────────────────────
8. PHASE 6 — SHORT MANIFEST NOTE

Create or update one short doc only if useful:

docs/SPRINT139_EXPLICIT_TEST_MANIFEST.md

Contents:
- why autotests=false was introduced
- which explicit tests are active
- what each survivor protects
- deleted legacy tests are old report/diagnostic surface
- full old auto-discovery is no longer the test contract
- safety assertions are preserved in survivor targets
- future tests must be added deliberately to Cargo.toml

Do not write a giant report.

────────────────────────────────────────
9. PHASE 7 — COMMIT PREP

Inspect:

git status --short
git diff --name-status
git diff --check

Commit candidate should include:
- Cargo.toml
- deleted legacy test files
- possibly docs/SPRINT139_EXPLICIT_TEST_MANIFEST.md
- any small survivor test adjustment required for safety preservation

Commit candidate should not include:
- new product features
- work.md
- unrelated docs
- unrelated examples
- Group B leftovers unrelated to no-run recovery
- accidental core changes

Suggested commit message:

test: cap integration targets and recover workspace no-run

Do not push unless explicitly instructed.

────────────────────────────────────────
10. ACCEPTANCE CRITERIA

Sprint 139 succeeds if:

- autotests=false strategy is verified.
- explicit test targets are intentional.
- deleted legacy tests are classified.
- uncertain safety-critical deletions are restored.
- minimal_ai_committee_core passes.
- workspace_timeout_reduction_queue passes.
- CLI smoke passes.
- cargo check passes.
- cargo build passes.
- cargo test --workspace --no-run --quiet passes.
- cargo test --workspace --quiet is attempted.
- FullWorkspaceAccepted is claimed only if full workspace test finishes and passes.
- no report bloat is added.
- no new CLI family is added.
- no product feature is added.
- no broker/order/account path is added.
- no training/live inference path is added.
- no real Mamba/Gated runtime is added.

────────────────────────────────────────
11. FINAL RESPONSE FORMAT

Keep final response short:

## 1. What changed

## 2. Explicit test manifest

## 3. Deleted legacy test classification

## 4. Restored files, if any

## 5. Tests run

## 6. Workspace no-run result

## 7. Full workspace test result

## 8. Commit readiness

## 9. Remaining risk

## 10. Next step

No giant report.
No 60-section output.