# Sprint 103-P1 Hermetic Test Evidence Report

## 1. Mode and scope

- Mode: implementation-only prerequisite repair.
- Branch: `agent/sprint103-prereq-hermetic-qualified-six-tests-v1`.
- Base: `origin/main` at `f3a8ec255add0588c395700f605f6170188cdf49`.
- Scope: repair the 38 pre-existing non-hermetic tests, add the required hermeticity
  regressions, and preserve production fail-closed behavior.
- No commit, push, PR mutation, M3 implementation, runtime evidence generation, or
  network access was performed.

## 2. Reproduction on authoritative main

A detached temporary worktree was created directly from `origin/main`. It contained
no production runtime state or ignored local evidence. All Rust commands used
`CARGO_BUILD_JOBS=1`, `CARGO_INCREMENTAL=0`, and one test thread.

Baseline full-library result:

```text
1271 total
1233 passed
38 failed
```

Exact failure categories and tests:

| Category | Count | Tests |
| --- | ---: | --- |
| Data acquisition | 1 | `learning_network_pilot_is_deferred_and_isolated_with_zero_authority` |
| Agent learning session | 1 | `v1_four_prospective_timestamps_enter_exclusion` |
| Qualified-Six replay | 32 | `sprint98_01`–`12`, `14`–`19`, `24`–`31`, `33`–`34`, `43`–`45`, and `47` |
| Qualified-Six diagnostics | 4 | `sprint99_01`, `02`, `07`, and `42` |

The result is classified as `ConfirmedPreExistingNonHermeticTests`.

## 3. Root cause

The failures were test prerequisite defects, not a production fallback defect.

| Tests | First hidden dependency | Expected location | Producer / consumer | Repair |
| --- | --- | --- | --- | --- |
| Data acquisition test | Eight protected-file sentinels | `config/local/...` | Developer-local protected files / acquisition safety assertion | Test-owned root with deterministic sentinel bytes |
| Agent learning test | Opening, momentum, and risk reservation metadata | `config/local/...` | Local prospective registrations / reservation loader | Existing deterministic `v1_reservation()` fixture |
| Replay source-dependent tests | Qualified-Six foundation, pause, and source views | `state/historical_replay/momentum_multitimeframe/v1` | Multi-timeframe persistence / `prepare_replay()` | Explicit evidence input plus test-owned foundation and pause |
| Replay `sprint98_44`–`45` | Persisted pause and live protected-tree state | historical root and `state/learning_data` | Protected-state loader / live safety assertions | Explicit historical and live roots |
| Diagnostics `sprint99_01`, `02`, `07` | Completed replay header artifacts | `state/historical_replay/momentum_qualified_six/v1` | Replay serializers / diagnostic header loader | Test-owned completed replay header |
| Diagnostics `sprint99_42` | Persisted pause and live protected-tree state | historical and live roots | Protected-state loader / live safety assertion | Explicit test-owned roots |

The replay `OnceLock` cached a value produced through repository-relative production
state. That made many otherwise pure contract tests fail when run before a local
runtime producer, or pass only on a developer machine that already had ignored
artifacts. Diagnostics had the same repository-relative replay-header dependency.
The other two failures had separate `config/local` dependencies and were repaired
locally rather than being coupled to the Qualified-Six fixture.

## 4. Evidence dependency graph

```text
deterministic canonical daily candles
    -> production foundation constructor
    -> validated pause + foundation + acquisition plan
    -> official encode / atomic persist / reopen / digest verification

deterministic six-view replay source
    -> explicit prepare_replay_from_evidence
    -> replay registration and partition contracts
    -> existing aggregate / benchmark / contribution / report builders
    -> official encode / atomic persist / decode
    -> diagnostic source header loaded from an explicit replay root
```

Each test materializes only the stage it needs. Negative and corruption tests own
private roots and do not share mutable evidence.

## 5. Architecture decision

- Existing production constants remain the default paths.
- Minimal internal `_at` functions accept explicit roots for foundation loading,
  protected-state loading, replay artifact persistence/reading, and diagnostic
  source-header loading.
- Existing production wrappers call those functions with their original constants.
- `prepare_replay_from_evidence` contains the deterministic core; the production
  `prepare_replay` wrapper still loads production evidence and therefore still fails
  closed when evidence is absent.
- `QualifiedSixTestWorldV1` is `#[cfg(test)]`, creates unique roots from the process
  id plus an atomic sequence, and removes only its owned root in `Drop`.
- No environment override, current-directory mutation, global mutable path, new
  dependency, or repository-root inference was introduced.

## 6. Foundation fixture

The fixture creates two strictly ordered, finite, positive, completed daily candles.
Changing the seed changes an allowed candle value and therefore changes the canonical
dataset digest. The fixture then uses the existing production `build_foundation` and
`build_plan` constructors.

Pause, foundation, and plan artifacts use the existing validators, encoders, atomic
persistence, decoders, and `reopen_foundation` path. Reopened values must exactly
match the constructed values. No production digest, report bytes, or developer
runtime artifact is copied.

## 7. Live-pause fixture

The pause represents the existing
`PausedAfterSealedEpochTwo` contract with deterministic identity bindings:

- outcome requests and openings are zero;
- epoch three is not registered;
- training, tournament, and live authority remain forbidden;
- time boundaries derive from constants rather than the current date;
- persistence and reopen use the normal pause artifact path.

The fixture performs no network request, market outcome read, live trade, or real
live-state copy.

## 8. Test migrations

- Data acquisition now snapshots eight deterministic sentinels inside its own
  temporary root, retaining the original before/after byte-equality assertions.
- Agent learning now uses the existing deterministic protected reservation fixture,
  retaining the four reserved timestamps and exclusion assertions.
- Qualified-Six replay tests use a deterministic in-memory six-timeframe source.
  Tests that specifically require pause/foundation state create a private world.
- Qualified-Six diagnostics tests create only a private foundation, completed replay
  header, or protected state as required.
- Ten `sprint103_p1_*` tests implement the required A–J hermeticity checks.
- No assertion was removed, ignored, converted to an early success, or relaxed to
  accept missing production evidence.

## 9. Hermeticity proof

The A–J regression set proves:

- an empty explicit root fails with `qualified-six foundation unavailable` and
  performs zero materialization;
- foundation and pause artifacts persist, reopen, bind, and validate;
- two physical roots produce equal semantic identities for equal source data;
- a canonical candle mutation changes source and dependent foundation identities;
- explicit evidence preparation succeeds without repository state;
- independent workspace creation order does not change results;
- corruption in one private workspace fails closed and leaves another valid;
- short and long root paths do not enter semantic digests;
- the production replay artifact default remains
  `state/historical_replay/momentum_qualified_six/v1`.

The code contains no test root environment override, `set_current_dir`, shared
mutable evidence store, or production-state copy.

## 10. Verification

All commands were sequential with one Cargo build job and one test thread.

| Verification | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo check --lib` | pass |
| Data acquisition focused | 1 passed |
| Agent learning focused | 1 passed |
| Qualified-Six replay focused | 61 passed |
| Qualified-Six diagnostics focused | 48 passed |
| New A–J hermetic tests | 10 passed |
| Default full library | 1281 passed |
| Metal full library | 1282 passed |
| `cargo test --tests` | library 1281, integration 404, timeout queue 12 passed |
| Exact `workspace_timeout_reduction_queue` target | 12 passed |
| Clean-style full library Run 1 | 1281 passed |
| Clean-style full library Run 2 | 1281 passed |
| `git diff --check` | pass |

For clean-style verification, the tracked source diff was applied to a detached
`origin/main` worktree with no runtime evidence. Run 1 left no runtime
artifact files. Its empty `state/learning_data` directory was removed before Run 2,
which produced the same result. The temporary worktree was then removed.

The optional default-thread parallel focused run was not performed because the
explicit machine-safety instruction required Rust execution to remain single-job
and single-threaded. Root collision, order independence, and corruption isolation
are instead covered directly by A–J.

## 11. Warnings

No new warning was introduced. The existing library check still reports four
dead-code warnings:

- `apply_agent_feedback`
- `proposal_is_valid_for`
- `train_encoded_head`
- `empty_result`

The library-test build reports the existing `train_encoded_head` warning.

## 12. Safety

- Production missing-evidence behavior remains fail-closed.
- Production default roots and public behavior remain unchanged.
- No network, live outcome, trading, reward, chair, or model-selection authority was
  added.
- No opaque generated replay bundle or production runtime evidence was committed.
- No unrelated source, M3 implementation, commit, push, or PR state was changed.
- Temporary cleanup targeted only test-owned roots and the dedicated validation
  worktree.

## 13. What this proves

- The original 38 failures reproduce on authoritative `origin/main`.
- The repaired tests run without developer-local runtime evidence.
- Qualified-Six test prerequisites are explicit and path-injected.
- Representative ordering, path identity, empty-root, and corruption contracts are
  regression-tested.
- Production missing evidence still fails closed.
- The branch is ready for independent review and later PR validation.

## 14. What this does not prove

- It does not reproduce real market outcomes or Qualified-Six investment performance.
- It does not validate M3-Micro predictive quality or authorize production trading.
- It does not make production execution succeed without genuine production evidence.
- It does not claim that every future repository test can never become flaky.

## 15. Remaining risks

- The optional multi-threaded focused run was intentionally omitted under the
  machine-safety constraint. Collision resistance is supported by unique roots and
  direct regression tests rather than an additional parallel stress run.
- Synthetic fixtures validate artifact and causal contracts; they are not substitutes
  for real production evidence or market evaluation.

## 16. Final status

`READY_FOR_REVIEW`

## 17. Exactly one next step

Have an independent reviewer inspect this diff and evidence report before any
authorized commit, push, or draft-PR publication.
