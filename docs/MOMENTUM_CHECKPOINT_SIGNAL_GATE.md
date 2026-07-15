# Momentum checkpoint signal gate

The offline momentum campaign records every completed head-training epoch for each fixed C0–C3 candidate. Each observation contains train and validation metric bundles, parameter and update summaries, a deterministic head digest, collapse state, signal status, and checkpoint eligibility.

Validation uses a train-label-prevalence Constant baseline. Brier is decomposed deterministically into reliability, resolution, and uncertainty using fixed bins. Rank AUC uses average ranks for probability ties and reports an explicit undefined state for a single-class partition.

A checkpoint is eligible only when its values are finite, sample count is sufficient, it is not collapsed or constant-like, its resolution is non-trivial, it is not materially worse than the train-derived Constant baseline, and any configured rank requirement passes. The eligible frontier preserves every eligible epoch. Selection is deterministic: lower Brier, higher resolution, lower reliability error, smaller update, then earlier epoch.

If a frontier is empty, the test partition remains sealed and the campaign emits a Shadow abstention with voting, execution, and promotion all false. If a selected validation checkpoint collapses on its one sealed test evaluation, the result is classified as temporal generalization collapse; test results never revise selection.

All resulting learning remains offline and ShadowOnly. This gate does not add candidates, data, model gradients, runtime inference authority, promotion, voting, or execution.

Selected checkpoints additionally receive a train-fitted temporal support audit before future labels are consulted. An out-of-support decision is an operational abstention, not a model win; later metrics are counterfactual diagnostics only.
