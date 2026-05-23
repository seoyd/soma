# Sprint 122 Core Prune and Offline Adapter

Sprint 122 reinforces the product path without adding report families or new CLI families.

- Added a local-only `OfflineMemberBrainAdapter` inside the existing minimal AI committee core.
- Added one offline opinion fixture for the existing `minimal-ai-committee-cycle` command.
- Kept DataRouter as a router only; it does not create opinions or recommendations.
- No legacy report modules were deleted because no additional target was clearly safe to remove without risking compatibility or assertion coverage.
- Legacy report/diagnostic surfaces remain deprecated for product direction: future AI core work should extend DataRouter, `AiMemberBrain`, committee event flow, chairman synthesis, Risk Governor veto, and paper score updates.

## Sprint 123 member core contract note

- Each AI member can now reference a deferred Mamba3 + Gated DeltaNet core spec.
- The registry uses lazy activation so a local Mac mini path does not try to load or run all 18 cores at once.
- RuntimeDeferred and TrainingDeferred cores are represented as `NeedMoreEvidence`, not fake AI opinions.
- This remains contract-only: no Mamba3 runtime, no Gated DeltaNet runtime, no training, and no live inference.

## Sprint 124 three-member pilot note

- The committee starts with three independent members: TrendEntryAI, RiskGuardAI, and EvidenceRegimeAI.
- Each member has its own role, deferred Mamba3 + Gated DeltaNet core spec, lightweight memory state, score, voice weight, and offline learning journal.
- The existing CLI can select `pilot_roster = "three_member"` without adding a new command family.
- Paper outcome feedback updates score/voice and journal entries only; it does not train or mutate runtime model weights.

## Sprint 125 archetype style-card note

- The 18 references are represented as public-philosophy-inspired archetype cards, not live agents or real-person clones.
- Style cards include do-not-learn guards, source confidence, review-required status, and prohibited-claim language.
- Style mapping injects lightweight influence into the existing three-member pilot only; it does not activate 18 AI members.
- Style influence can adjust deterministic/offline opinion confidence slightly, but Risk Governor remains final and no order path is created.
