# Chair v0

Chair v0 is the Sprint 32 committee aggregator.

## Speaker selection

- inactive personas are ignored
- incompatible source/horizon votes are filtered
- speakers are selected in deterministic score order

## Decision logic

- regime fit and voice power weight each vote
- cluster penalty reduces repeated same-cluster voices
- optional contrarian inclusion prevents one-sided debates
- high disagreement or high groupthink risk forces conservative outcomes
- hard veto remains absolute when enabled

## Risk Governor handoff

Chair v0 does **not** approve execution by itself. Every candidate still passes through the existing Risk Governor, and the resulting path remains paper/research only.

