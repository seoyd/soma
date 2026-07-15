# Restart Sprint37 Report

## Scope

This change adds deterministic probability-collapse metrics and a validation-only candidate-forensics path to the existing offline momentum campaign. It does not add data access, providers, trading, model promotion, encoder training, GPU training, or online learning.

## Implementation

The campaign now carries a validated collapse contract and runs the pre-registered C0--C3 forensic candidates per eligible chronological window. Only an eligible validation winner is evaluated on test; non-selected candidates retain no test metrics. The original campaign path and ShadowOnly versioning remain intact.

## Verification

Default and Metal baseline commands were run sequentially before edits. Focused learning-campaign tests cover collapse detection, canonical evidence safety, Shadow isolation, and the one-selected-candidate test seal.

## Result boundary

This report describes code and test behavior only. It makes no claim of collapse resolution, market edge, profitability, promotion readiness, voting readiness, execution readiness, or official conformance.
