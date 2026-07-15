# Momentum Probability-Collapse Forensics

The offline campaign uses a validated probability-collapse contract: probability standard deviation, entropy, single-side fraction, saturation fractions, unique probability bins, and a minimum sample count. The measured subtypes are near-constant, near-zero, near-one, single-side, saturated, low-entropy, and insufficient-unique predictions.

The forensic candidate registry is deterministic and capped at four entries: C0 reference, train-fitted representation normalization, training-prevalence bias initialization, and their combination. All candidates train on train data and are compared on validation only. The first validation-eligible candidate selected by lowest Brier with deterministic tie-breaking is the sole candidate whose test partition is opened.

Representation normalization fits only encoded training examples. Bias initialization uses only training-label prevalence with a bounded finite logit. The frozen encoder remains unchanged, all output remains ShadowOnly, and no candidate grants promotion, voting, or execution authority.
