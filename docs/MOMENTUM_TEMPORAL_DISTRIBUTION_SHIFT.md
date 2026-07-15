# Momentum temporal distribution shift

Temporal diagnostics preserve causal ordering: fit support envelopes on training representations, audit validation coverage, compute test support from label-free representations, then record test-label metrics only as offline research diagnostics.

The closure records the same train-to-test distribution bundle at raw-feature, train-normalized-feature, sequence, frozen-representation, representation-scale, logit, and probability stages. It records outcome shift only after the sealed support decision. Earliest-stage precedence is raw features, normalized features, sequences, frozen representations, representation scale, logits, probabilities, then outcomes. This prevents a later metric from overwriting an earlier measured breach.

The support envelope uses train means and scales. It reports standardized mean shift, log variance ratio, and train-support breaches. Validation coverage determines whether the gate is usable; fixed thresholds are not tuned from test outcomes.

An out-of-support test produces a Shadow abstention with promotion, voting, and execution disabled. Any later test metric is counterfactual research evidence only and cannot create a positive model-quality claim. Existing candidates, frozen encoder behavior, chronology, and ShadowOnly boundaries remain unchanged.
