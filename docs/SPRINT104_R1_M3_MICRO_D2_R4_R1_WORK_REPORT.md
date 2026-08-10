# SOMA Sprint 104-R1-R10-D2-R4-R1 Work Report

## 1. Mode and Git State

- Mode: MINIMAL_D2_EXECUTION_HARNESS_REPAIR.
- Existing mixed worktree was preserved; staged changes remain 0.
- This repair changes only the D2 test harness and project reports.

## 2. D2-R4 Closure Review Blocker

- Historical D2-R4 is NOT_APPROVED because its parent printed typed execution facts without retaining them in the task-owned log root.

## 3. User Concurrency Requirement Correction

- D2 now treats runner-owned child overlap as the relevant concurrency boundary.
- Unrelated host Cargo and rustc activity is not a D2 correctness blocker.

## 4. Progress Ledger

| Stage | Status | Evidence | Blocker |
|---|---|---|---|
| Starting state | VERIFIED | source and git audit | none |
| D2-R4 evidence blocker | CONFIRMED | historical closure record | none |
| Existing child logging | PRESERVED | source | none |
| Parent log lifecycle | PASS | focused tests | none |
| Create-once / append-only / flush | PASS | focused tests | none |
| Partial failure retention | PASS | focused test | none |
| Parent event retention | IMPLEMENTED | source audit | none |
| Parent semantic isolation | PASS | focused test | none |
| Machine-wide Cargo gate | REMOVED | source audit | none |
| Unrelated Cargo / rustc | ALLOWED | focused test | none |
| Soma child sequencing | PASS | focused test and synchronous runner | none |
| Candidate / authority / binding | PRESERVED | focused representatives | none |
| Functional report | PASS | current-state validator | none |
| Receipt currentness | CURRENT | read-only checks | none |
| Heavy D2 / generators | NOT_RUN | policy | none |
| D2-R4-R1 | CHANGES_REQUESTED | independent review | durable candidate checkpoint coverage incomplete |

## 5. Previous Parent-Evidence Gap

- Raw child logs existed, but no write-once parent record retained candidate, authority, observation, binding, publication, or multiplicity facts.

## 6. Parent Log Design

- The existing task-owned `logs/` directory now receives `d2-parent.log`.
- It is an operational audit projection only and is not semantic evidence.

## 7. Create-Once / Append-Only Contract

- Parent-log creation uses create-new semantics.
- Each event is appended, synchronized, and never truncates or overwrites an existing log.

## 8. Failure Durability

- An incomplete parent log receives `RUN_BLOCKED` on scope exit and never receives `RUN_COMPLETE`.

## 9. Parent Event Inventory

- The runner records start, base identity, candidate, authority, suite, inventory, binding, publication, currentness, post-publication, final-candidate, retry, and completion events when reached.

## 10. Candidate Evidence Events

- Candidate freeze, Main candidate, Clean candidate, pre-run equality, and final candidate match are recorded from runtime-derived values.

## 11. Clean Authority Evidence Events

- Clean authority issuance records the verified candidate and semantic checkout provenance identity.

## 12. Actual Observation Evidence Events

- Each validated Actual observation records suite, scope, candidate, Actual origin, outcome counts, and its existing semantic digest.

## 13. Direct-Binding Evidence Events

- Actual inventory and Direct Binding records retain the existing required-set and execution-set digests without changing their derivation.

## 14. Publication / Currentness Events

- Capability, Metal, and Delivery decisions and currentness results are recorded after the existing validation flow.

## 15. Post-Publication Events

- The canonical runner executes the existing focused post-publication contract one test at a time and records each started/passed result.

## 16. Retry / Multiplicity Events

- The parent records one invocation, completed runner-owned child count, zero duplicates, and zero automatic/manual retries for a successful closure.

## 17. Parent-Log Semantic Isolation

- Parent-log content and path are outside candidate derivation and do not enter Actual, Direct Binding, or receipt semantic inputs.

## 18. Child-Log Preservation

- Existing raw, write-once child logs remain unchanged and continue to complement the parent log.

## 19. Previous Machine-Wide Concurrency Policy

- The obsolete host-wide Cargo/rustc audit was removed from the D2 launch path.

## 20. Project-Scoped Concurrency Decision

- D2 owns a single synchronous child slot; uncertain external project identity is never guessed or blocked.

## 21. Unrelated Cargo Behavior

- Synthetic unrelated Cargo process input is allowed by the focused regression.

## 22. Unrelated rustc Behavior

- Synthetic unrelated rustc process input is allowed by the same focused regression.

## 23. Soma Internal Sequential Execution

- The child-slot tracker rejects a second active owned child, while the existing synchronous command execution waits before the next child starts.

## 24. Candidate Manifest Preservation

- PRESERVED.

## 25. Verification Input Preservation

- PRESERVED.

## 26. Clean Authority Preservation

- PRESERVED.

## 27. Actual Authority Preservation

- PRESERVED.

## 28. Direct-Binding Preservation

- PRESERVED.

## 29. Frozen Metal Boundary

- FROZEN. No Metal topology or hardware semantics changed.

## 30. AI/Model Immutability

- UNCHANGED.

## 31. Functional Report Preservation

- The existing functional report structure and current-state validator were preserved; only the historical D2-R4 and repair status were added.

## 32. Receipt Currentness

- Capability, Metal V11, and Delivery currentness were checked read-only.

## 33. Hardcoding Audit

- No current candidate, observation, binding, receipt, revision, branch, test-total, task-root, or external-process identifier was added as a constant.

## 34. Focused Verification

- Parent logger success, blocked partial log, duplicate rejection, flush/reopen, semantic isolation, unrelated Cargo/rustc allowance, and child seriality: PASS.
- Clean authority, root-path independence, Direct Binding, R3D publication exclusion, R1F, D1-R2, and retirement representatives: PASS.

## 35. Explicitly Not Run

- Full: NOT_RUN
- Metal Full: NOT_RUN
- Integration: NOT_RUN
- Clean: NOT_RUN
- D2: NOT_RUN
- Hardware: NOT_RUN
- Generators: NOT_RUN

## 36. Warning Audit

- New warnings: 0.

## 37. Status Separation

- Historical D2-R4: NOT_APPROVED
- D2-R4-R1: CHANGES_REQUESTED
- Parent Evidence: PARTIAL
- Concurrency Contract: IMPLEMENTED
- Candidate Manifest: PRESERVED
- Clean Authority: PRESERVED
- Direct Binding: PRESERVED
- Capability: BlockedByLength32ConfidenceCapability
- Metal: PASSED_ACTUAL_SEMANTIC_COMMAND_TOPOLOGY
- Delivery: PASSED
- Overall: BlockedByLength32ConfidenceCapability

## 38. What This Fixes

- Retains durable parent orchestration evidence and limits concurrency enforcement to D2-owned children; it does not centralize every candidate checkpoint event.

## 39. What This Does Not Prove

- It does not approve historical D2-R4, run a fresh D2, guarantee durable coverage for every candidate checkpoint, change semantic verification contracts, or alter the capability-model status.

## 40. Final Status

- CHANGES_REQUESTED — candidate checkpoint durable coverage incomplete.

## 41. Exactly One Next Step

- 독립 targeted review로 넘긴다.
