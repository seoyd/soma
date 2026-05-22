# Sprint 123 Member Core Contract

Sprint 123 defines each AI committee member as a deferred Mamba3 + Gated DeltaNet style core.

- This is a contract only: no Mamba3 runtime, no Gated DeltaNet runtime, no training, and no live inference.
- `AiMemberCoreRegistry` keeps core specs and market-scope bindings so the scheduler can activate only relevant members.
- The default policy uses lazy activation and caps active members to avoid running all 18 cores at once.
- `CoreAwareMemberBrainAdapter` maps `OfflineFixture` to local fixture opinions, `MockLocal` to deterministic mock opinions, and deferred runtime/training states to `NeedMoreEvidence`.
- The Mac mini policy is conservative: activate roughly 3-5 members per event cycle, keep Risk Governor lightweight, and unload after cycle.
- The program remains the orchestrator; AI members produce opinions and Risk Governor can still veto.
