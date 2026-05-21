# Mamba3Fin vs GatedDeltaNet

Both candidate families now share the same sequence-core comparison boundary.

- shared contract: input tensor context, feature schema hash, label manifest hash, dataset version, split policy, no-lookahead proof
- family difference: `GatedDeltaNet` keeps its gated-delta contract/state-spec references, while `Mamba3Fin` keeps its own candidate contract identity
- limitation: Sprint 80 compares external prototype CSV outputs only; no runtime, no training, no deployment

