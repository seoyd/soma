# Shadow agent deliberation ledger

Cross-agent reveal requires two valid primary seals for distinct objectives.
The relationship layer records temporal and evidence alignment and classifies
compatibility, orthogonality, tension, direct conflict, abstention, or
incomparability without selecting a winner or action. Tension is a record of
objective-aware disagreement dimensions, never an action mapping.

The replay is limited to exactly two rounds: independent primary opinions, then
one structured response per agent. Arguments contain a claim, evidence and
uncertainty references, and an optional requested resolution; they cannot alter
their sealed primary opinion.

Older and newer regimes replay independently. A historical aggregate is only
created when every included Momentum regime has a corresponding Cycle/Risk
regime; an incomplete aggregate scope is rejected rather than inferred.

Scope attestations are external immutable references. Canonical row identity,
effective-anchor identity, and forecast horizon are checked before a pair is
eligible for aggregate composition; opinion, seal, argument, and transcript
digests are never rewritten to add this information.

The ledger keeps deterministic typed opinion and relationship indexes and only
appends unique valid transcripts. It preserves primary-opinion immutability and
produces an ineligible future-Chair packet only; it creates no Chair
observation, reward, penalty, vote, speaking-right change, or execution.
