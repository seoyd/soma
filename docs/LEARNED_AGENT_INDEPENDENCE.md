# Learned Agent Independence

The Cycle/Risk Shadow and Momentum Shadow may read the same immutable raw historical OHLCV, but that is their only shared learning input. `LearnedAgentIndependenceProofV0` verifies distinct agent identities, feature and label schemas, feature normalizers, frozen-encoder parameters, logistic-head parameters, recurrent state, model-version namespace, journal namespace, and the absence of prediction dependency.

The risk agent computes labels itself from future adverse excursion and trains its own R0/R1/R2 models. It does not call Momentum feature builders, Momentum labels, Momentum normalizers, Momentum model versions, Momentum journal entries, or Momentum predictions. Generic tensor, logistic, Brier, and AUC routines are reused only as stateless mathematical utilities.

The proof is successful only when every invariant is true. Even then it demonstrates architectural separation and deterministic offline replay, not live predictive value, external conformance, trading fitness, or eligibility for committee participation.
