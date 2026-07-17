# Independent learned-agent opinion protocol

Momentum and Cycle/Risk opinions use distinct objective-specific payloads. They
are historical-only, advisory-only records; no common numeric confidence,
cross-objective probability, or generic score exists. A probability may only be
an objective-specific sealed artifact reference; the current historical replay
does not expose one.

Each primary opinion is created without the counterpart opinion, from its own
sealed evidence and model-artifact identity, then sealed before any cross-agent
reveal. The envelope records typed uncertainty, assumptions, invalidation
conditions, temporal/evidence scope, source digest, and all-false authority
eligibility. Seal verification rejects a changed primary opinion.

Abstention is a complete opinion. It does not imply a missing opinion, a trade
action, or a technical failure. Historical opinions are not prospective
evidence, and a scope or digest violation invalidates an opinion rather than
turning it into a live claim.
