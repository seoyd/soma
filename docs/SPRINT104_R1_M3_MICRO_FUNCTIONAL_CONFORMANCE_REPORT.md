# SOMA Sprint 104-R1-R10-D2-R4-R2 Functional Conformance Report

## 1. Mode and Git State

- Mode: MINIMAL_PARENT_CHECKPOINT_COVERAGE_REPAIR
- Staged changes: 0
- Existing mixed worktree: preserved.

## 2. Starting Blocking Defect

- Clean authority runtime root binding was serialized into Actual Clean semantic evidence through the authority token.
- Historical D2-R3: BlockedByPostPublicationVerification.
- Historical D2-R4: NOT_APPROVED because parent execution evidence was not retained for re-audit.
- D2-R4-R1: CHANGES_REQUESTED because durable candidate checkpoint coverage was incomplete.
- D2-R4-R2: centralized checkpoint evidence repair implemented; fresh D2 remains NOT_RUN.
- D2-R3-R1 Functional Report repair: APPROVED.
- D2-R3-R2 Clean provenance authority repair: IMPLEMENTED.
- D2-R3-R2-R1 Runtime root binding / semantic projection separation: IMPLEMENTED.

## 3. Progress Ledger

| Stage | Status | Evidence | Blocker |
|---|---|---|---|
| Starting state | VERIFIED | source audit | none |
| Authority serialization audit | VERIFIED | source | none |
| root_binding classification | RUNTIME_ONLY | audit | none |
| candidate identity classification | SEMANTIC | audit | none |
| checkout provenance classification | SEMANTIC | audit | none |
| checkout provenance path audit | CLEAN | source | none |
| Existing semantic projection | NONE | source | none |
| Explicit projection | IMPLEMENTED | source | none |
| Authority Serialize | REMOVED | source | none |
| root runtime binding | PRESERVED | focused | none |
| Same semantics/different roots | PASS | focused | none |
| Wrong-root rejection | PASS | focused | none |
| Stale-candidate rejection | PASS | focused | none |
| Missing authority | PASS | focused | none |
| Provenance semantic binding | PASS | focused | none |
| Direct-Binding path independence | PASS | focused | none |
| Raw boolean bypass | CLOSED | audit | none |
| Main unaffected | PASS | focused | none |
| Synthetic promotion | BLOCKED | audit | none |
| Actual origin | PRESERVED | audit | none |
| Candidate Manifest | PRESERVED | diff | none |
| Verification Input | PRESERVED | diff | none |
| D1-R2 | PRESERVED | focused | none |
| Functional Report R1 | PASS | focused | none |
| Capability currentness | CURRENT | read-only | none |
| Metal currentness | CURRENT | read-only | none |
| Delivery currentness | CURRENT | read-only | none |
| Metal | FROZEN | diff | none |
| Model | UNCHANGED | diff | none |
| Heavy D2 | NOT_RUN | policy | none |
| Generators | NOT_RUN | policy | none |
| fmt/check | PASS | sequential command | none |
| New warnings | 0 | compiler audit | none |
| D2-R3-R2-R1 | READY_FOR_INDEPENDENT_REVIEW | focused evidence | none |
| D2-R4 parent evidence | NOT_RETAINED | independent closure review | evidence re-auditability |
| D2-R4-R1 parent logger lifecycle | PRESERVED | focused logger tests | none |
| D2-R4-R1 concurrency contract | PRESERVED | focused process and seriality tests | none |
| D2-R4-R1 checkpoint coverage | INCOMPLETE | independent review | durable checkpoint events missing |
| D2-R4-R2 checkpoint ownership | CENTRALIZED | source call-site audit | none |
| D2-R4-R2 checkpoint durability | PASS | focused helper tests | none |

## 4. Authority Serialization Call Graph

- Actual Clean verifier issues the private authority after detached checkout and clean-state verification.
- Actual observation construction uses the authority for root and candidate runtime checks.
- Semantic digest consumes the authority's private semantic projection, not the authority token.

## 5. Authority Field Classification

| Field | Runtime | Semantic | Result |
|---|---|---|---|
| root_binding | YES | NO | runtime-only private binding |
| candidate_scope_identity | YES | YES | candidate semantic identity |
| checkout_provenance_identity | YES | YES | verified provenance semantic identity |

## 6. root_binding

- Canonical-root binding remains private and is compared only by the Actual Clean builder.

## 7. Candidate Identity

- The verified candidate identity remains part of runtime validation and semantic evidence.

## 8. Checkout Provenance Identity

- The verified detached checkout provenance identity remains part of runtime authority and semantic evidence.

## 9. Checkout Provenance Path Audit

- The provenance identity is derived from expected HEAD, detached state, and normalized clean-state digest; it does not serialize an absolute checkout root.

## 10. Semantic Projection Decision

- No suitable projection existed. A private projection containing only candidate and checkout provenance identities was added in the existing authority module.

## 11. Authority Serialize Boundary

- The verified Clean authority no longer derives serialization.
- Only the private semantic projection is serialized for the observation digest.

## 12. Runtime Root Binding Preservation

- Root A authority remains rejected by Root B construction, including when candidate bytes match.

## 13. Same Semantics / Different Roots

- Two detached worktrees at distinct canonical roots with the same candidate and provenance produce the same Actual Clean semantic digest.

## 14. Wrong-Root Negative

- A Root A authority used at Root B is rejected.

## 15. Stale-Candidate Negative

- Candidate mutation after authority issuance is rejected.

## 16. Missing-Authority Negative

- CleanDetached Actual construction without a verified authority is rejected.

## 17. Provenance Semantic Binding

- A genuine checkout provenance change on the same candidate root produces distinct Actual Clean semantic evidence.

## 18. Direct-Binding Path Independence

- Equivalent complete Actual evidence from distinct Clean worktree roots produces the same direct-binding execution-set digest.

## 19. Raw Boolean Bypass Audit

- Raw boolean authority: REMOVED.
- No boolean Clean authority constructor, default, deserialization, or synthetic promotion path is exposed.

## 20. Main Scope

- Main Actual observations still require no Clean authority.

## 21. Synthetic Separation

- Synthetic fixtures cannot receive or promote a verified Clean authority.

## 22. Actual-Origin Preservation

- Actual origin remains private and is issued only by the existing Actual builder.

## 23. Generic-Assembler Preservation

- The existing private assembler remains the only observation assembly path.

## 24. Candidate Manifest Preservation

- Candidate Manifest: PRESERVED.

## 25. Verification Input Preservation

- Verification Input: PRESERVED.

## 26. D1-R2 Preservation

- Approved branch/detached provenance identity semantics are preserved.

## 27. Functional Report R1 Preservation

- D2-R3-R1 Functional Report repair: APPROVED.
- The report validator retains the existing currentness and source-state checks while using the R2-R1 section contract.
- Historical D2-R4 remains NOT_APPROVED; D2-R4-R1 is CHANGES_REQUESTED for incomplete checkpoint coverage.
- D2-R4-R2 centralizes checkpoint event retention; the fresh D2 after this repair is NOT_RUN.

## 28. Receipt Currentness

- Current Capability Receipt: CURRENT
- Current Capability Model: BlockedByLength32ConfidenceCapability
- Current Metal Receipt: CURRENT
- Current Metal: PASSED_ACTUAL_SEMANTIC_COMMAND_TOPOLOGY
- Current Delivery Receipt: CURRENT
- Current Delivery: PASSED
- Current Overall: BlockedByLength32ConfidenceCapability
- Current Direct Binding: PASSED
- Read-only actual state only; no receipt was written.

## 29. Frozen Metal Boundary

- Metal: FROZEN.
- No Metal topology or hardware execution change was made.

## 30. AI/Model Immutability

- Model: UNCHANGED.

## 31. Hardcoding Audit

- No root path, HEAD, candidate, receipt, or test-total literal was added.
- Runtime root binding is derived from canonicalization only at verification and Actual construction.

## 32. Focused Verification

- Semantic path independence, wrong-root, stale-candidate, missing-authority, provenance sensitivity, Main scope, and synthetic separation: PASS.
- D1-R3B, R3D, R1F, D1-R2, Case C, report validation, and read-only receipt checks: PASS.
- Formatting and default library checks ran sequentially.
- Parent-log create-once, append/flush, blocked partial retention, completion marker, semantic isolation, unrelated-process allowance, and owned-child seriality: PASS.
- Arbitrary and required Pre/Post checkpoint labels, one-event-per-invocation, mismatch retention, and checkpoint flush/reopen: PASS.

## 33. Explicitly Not Run

- Full: NOT_RUN
- Metal Full: NOT_RUN
- Integration: NOT_RUN
- Clean: NOT_RUN
- D2: NOT_RUN
- Hardware: NOT_RUN
- Generators: NOT_RUN

## 34. Warning Audit

- New warnings: 0.
- Existing test-build warnings remain outside this repair.

## 35. Status Separation

- Historical D2-R3: BlockedByPostPublicationVerification
- Historical D2-R4: NOT_APPROVED
- D2-R4-R1: CHANGES_REQUESTED
- D2-R4-R2: READY_FOR_INDEPENDENT_REVIEW
- D2-R3-R1: APPROVED
- D2-R3-R2: IMPLEMENTED
- D2-R3-R2-R1: READY_FOR_INDEPENDENT_REVIEW
- Clean Runtime Authority: PRESERVED
- Clean Semantic Evidence: IMPLEMENTED
- Candidate Manifest: PRESERVED
- Direct Binding: PRESERVED
- Capability: BlockedByLength32ConfidenceCapability
- Metal: PASSED_ACTUAL_SEMANTIC_COMMAND_TOPOLOGY
- Delivery: PASSED
- Overall: BlockedByLength32ConfidenceCapability

## 36. What This Fixes

- Separates the absolute-root runtime binding from semantic evidence while retaining fail-closed Clean authority validation.
- Makes equivalent Actual Clean evidence path-independent without weakening candidate or checkout provenance binding.
- Adds durable, non-semantic parent orchestration evidence and removes unrelated host Cargo/Rust activity from the D2 correctness gate.
- Centralizes durable candidate checkpoint evidence so every D2 checkpoint helper invocation records its actual re-derived candidate.

## 37. What This Does Not Prove

- It does not rerun D2, Full, integration, hardware, or generators.
- It does not publish or regenerate any receipt.
- It does not close the historical D2-R3 execution.
- It does not approve historical D2-R4 or execute a fresh D2 after the harness repair.
- It does not run a fresh D2-R4-R2 closure, write a receipt, or declare parent evidence closed.

## 38. Final Status

- READY_FOR_INDEPENDENT_REVIEW

## 39. Exactly One Next Step

- 독립 targeted review로 넘긴다.
