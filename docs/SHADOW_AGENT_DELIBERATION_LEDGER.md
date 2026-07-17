# Shadow agent deliberation ledger

Cross-agent reveal requires two valid primary seals. The relationship layer
classifies compatibility, orthogonality, tension, abstention, or
incomparability without selecting a winner or action. The current replay is
limited to exactly two rounds: independent primary opinions, then one
structured response per agent.

The transcript is append-only in intent and keeps primary opinions immutable.
It produces an ineligible future-Chair packet only; it creates no Chair
observation, reward, penalty, vote, speaking-right change, or execution.
