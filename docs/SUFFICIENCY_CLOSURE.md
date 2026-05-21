# Sufficiency Closure

Sprint 38 adds a closure report that compares previous coverage/sufficiency state against a regenerated reference pack.

## What it compares
- previous outcome links vs current outcome links
- previous baseline references vs current baseline references
- previous no-trade/risk-denied counterfactual depth vs current depth
- previous official-ready count vs current official-ready count

## Closure statuses
- `ImprovedButStillInsufficient`
- `OutcomeLinksClosed`
- `CounterfactualDepthClosed`
- `SufficiencyGatePassedForControlledEvidence`
- `SufficiencyGatePassedForOfficialEvidence`
- `StillNeedMoreCandleData`
- `StillNeedMoreOfficialRows`
- `NoImprovement`

## Important boundary
A controlled evidence pass is still not official readiness. Official sufficiency requires true official or real-local evidence, while controlled or fixture-backed closure remains research-only.
