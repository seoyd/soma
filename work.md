You are operating as a gstack-style engineering team for Soma Zero.

Current state:
Sprint 139 is complete.

Important result:
- cargo test --workspace --no-run --quiet now passes.
- cargo test --workspace --quiet now passes.
- This acceptance is for the new explicit test manifest target set.
- Cargo.toml has autotests = false.
- Active integration test targets are:
  1. minimal_ai_committee_core
  2. workspace_timeout_reduction_queue
- 242 legacy sprint/report/diagnostic/gate/panel/timeout tests were deleted.
- Safety assertions were preserved in the two survivor tests.
- Full old auto-discovery over all remaining 907 test files is no longer the test contract.

Current risk:
The worktree still has multiple kinds of changes mixed together:
1. Sprint 139 no-run recovery / explicit manifest changes.
2. Product feature changes from recent sprints if not committed.
3. legacy prune leftovers.
4. work.md scratchpad.
5. unrelated docs/examples.

Sprint 140 objective:
Finalize repository state after no-run recovery. Separate and prepare clean commit boundaries so the project can safely return to feature implementation.

This sprint is not about new features.
This sprint is not about reports.
This sprint is not about more tests.
This sprint is about locking the recovered test contract and cleaning the working tree boundaries.

────────────────────────────────────────
0. SPRINT NAME
────────────────────────────────────────

gstack Sprint 140:
Explicit Manifest Commit Boundary + Repository Clean State + Feature Track Reopen

────────────────────────────────────────
1. HARD RULES

Do not add:
- new feature code
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
- Tauri/Svelte/dashboard/browser

Do not:
- re-enable autotests
- re-enable broad integration test auto-discovery
- restore 242 deleted legacy tests unless safety-critical
- stage work.md
- stage unrelated scratch docs
- mix product feature changes with test-manifest cleanup
- claim old 907-test auto-discovery acceptance
- claim full acceptance beyond the explicit manifest contract

Allowed:
- inspect worktree
- separate changes into clean groups
- stage Sprint 139 no-run recovery bundle only
- optionally commit Sprint 139 bundle if user allows
- keep product changes separate
- keep scratch files ignored/unstaged
- run full validation
- produce short summary

Main rule:
Lock the test contract first.
Then return to product features.

────────────────────────────────────────
2. CURRENT TEST CONTRACT

The official workspace test contract is now:

Cargo.toml:
- autotests = false
- explicit [[test]] targets only

Active integration test targets:
1. tests/minimal_ai_committee_core.rs

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
- owner attention inbox
- watchlist recheck
- no broker/order/account
- no training/live inference
- deterministic product behavior

2. tests/workspace_timeout_reduction_queue.rs

Protects:
- no-run/full-acceptance truth
- timeout safety
- cargo JSON diagnostic safety
- CLI warning/forbidden command safety
- read-only/control-tower legacy safety assertions if migrated
- no false full-acceptance claim
- determinism assertions migrated from old sprint tests
- hidden-skip / no-assertion-deletion guard added in Sprint 139

Do not add more explicit targets unless a unique safety assertion has no surviving home.

────────────────────────────────────────
3. PHASE 1 — INSPECT WORKTREE

Run:

git status --short
git status -sb
git diff --name-status
git diff --stat
git diff --cached --name-status
git log --oneline -5

Also inspect test contract:

grep -n "autotests" Cargo.toml
cargo test --list --workspace

Confirm:
- Cargo.toml has autotests=false.
- active integration targets are only minimal_ai_committee_core and workspace_timeout_reduction_queue.
- work.md is not staged.
- product feature files are not mixed with no-run recovery bundle unless intentionally part of same commit.

────────────────────────────────────────
4. PHASE 2 — CLASSIFY CURRENT CHANGES

Classify every changed file into one group.

Group A — Sprint 139 no-run recovery bundle:
- Cargo.toml
- deleted legacy tests, 242 files
- tests/workspace_timeout_reduction_queue.rs
- docs/SPRINT139_EXPLICIT_TEST_MANIFEST.md if present

Group B — product feature changes:
- src/league/minimal_ai_committee_core.rs
- src/bin/soma_experiment.rs
- examples/soma_minimal_ai_committee_core.toml
- examples/minimal_* product samples
- tests/minimal_ai_committee_core.rs
Only include these if they are not already committed.

Group C — scratch / ignore:
- work.md
- session plan.md
- temporary prompt notes

Group D — old docs / optional cleanup:
- docs/SPRINT121...
- docs/SPRINT122...
- unrelated legacy docs
- old examples not part of product

Rules:
- Group A should be handled first.
- Group B should be separate.
- Group C should never be committed.
- Group D should be either ignored, deferred, or cleaned separately.

────────────────────────────────────────
5. PHASE 3 — VERIFY SPRINT 139 BUNDLE ONLY

If Group A is not yet committed, isolate it.

Stage Group A only:

git add Cargo.toml
git add tests/workspace_timeout_reduction_queue.rs
git add docs/SPRINT139_EXPLICIT_TEST_MANIFEST.md

For deleted legacy tests:
git add -u tests

But before staging all test deletions, confirm that only intended 242 legacy deletions are included.

Use:
git diff --cached --name-status

Do not stage:
- work.md
- product feature changes
- unrelated docs
- unrelated examples

If product changes are dirty and would affect validation, optionally stash them:

git stash push --include-untracked --keep-index -m "sprint140-product-and-scratch-deferred"

Then validate Group A.

────────────────────────────────────────
6. PHASE 4 — VALIDATION

Run:

cargo fmt --all
cargo fmt --all --check
cargo check --workspace
cargo build --bin soma_experiment
cargo test --test minimal_ai_committee_core --quiet
cargo test --test workspace_timeout_reduction_queue --quiet
cargo run --quiet --bin soma_experiment -- minimal-ai-committee-cycle --config examples/soma_minimal_ai_committee_core.toml
cargo test --workspace --no-run --quiet
cargo test --workspace --quiet
git diff --check

Expected:
- no-run passes.
- full workspace passes.
- explicit manifest target set is accepted.
- no full old auto-discovery claim.

If any command fails:
- fix only the minimal cause.
- do not add reports.
- do not re-enable broad tests.
- do not hide failures.

────────────────────────────────────────
7. PHASE 5 — COMMIT DECISION

If validation passes and user allows commit:

Suggested commit message:

test: cap integration targets and recover workspace no-run

Commit should include:
- Cargo.toml
- deleted legacy tests
- tests/workspace_timeout_reduction_queue.rs
- docs/SPRINT139_EXPLICIT_TEST_MANIFEST.md

Commit should not include:
- product feature changes
- work.md
- session plan
- unrelated docs
- unrelated examples

If user does not want commits:
- do not commit.
- provide exact staging list and readiness summary.

Do not push unless explicitly instructed.

────────────────────────────────────────
8. PHASE 6 — RESTORE DEFERRED CHANGES

If stash was used:

git stash pop

Then inspect:

git status --short
git diff --name-status

Confirm:
- Sprint 139 bundle is committed or staged cleanly.
- product feature changes remain separate.
- work.md remains ignored/unstaged.
- no conflicts.

If stash conflict occurs:
- stop.
- report conflict list.
- do not auto-resolve unless trivial.

────────────────────────────────────────
9. PHASE 7 — FEATURE TRACK REOPEN DECISION

After Sprint 139 bundle is clean, choose next product direction.

Recommended next feature sprint:
Sprint 141:
Owner Daily Brief Storage + Committee State Export for Future UI

Rationale:
- autonomous paper loop exists.
- owner attention inbox exists.
- watchlist recheck exists.
- daily brief exists.
- before UI, we need stable state/export shape.

Do not jump to Tauri/Svelte yet.
First make a stable JSON export that UI can consume.

────────────────────────────────────────
10. ACCEPTANCE CRITERIA

Sprint 140 succeeds if:

- current worktree is classified.
- Sprint 139 no-run recovery bundle is isolated.
- work.md is excluded.
- product changes are not mixed with test-manifest cleanup.
- cargo fmt passes.
- cargo check passes.
- cargo build passes.
- minimal_ai_committee_core passes.
- workspace_timeout_reduction_queue passes.
- CLI smoke passes.
- cargo test --workspace --no-run --quiet passes.
- cargo test --workspace --quiet passes.
- no full old auto-discovery claim is made.
- commit readiness is clear.
- next product feature direction is clear.

────────────────────────────────────────
11. FINAL RESPONSE FORMAT

Keep short:

## 1. What changed

## 2. Worktree classification

## 3. Explicit test manifest status

## 4. Validation results

## 5. No-run result

## 6. Full workspace result

## 7. Commit readiness

## 8. Excluded files

## 9. Remaining risk

## 10. Next feature recommendation

No giant report.
No 60-section output.