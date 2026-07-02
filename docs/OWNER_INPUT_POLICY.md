# Owner Input Policy

Owner input is advisory. The owner may request review, reanalysis, stricter risk,
candidate hold/dismissal, or paper confirmation. Owner input cannot create an
order, force a trade, loosen a hard veto, enable a broker, or bypass Chair and
Risk Governor.

Every requested paper action follows the existing numeric path. `NoTrade`
remains the default. A paper action is eligible only when owner policy allows
the input and Risk Governor independently returns `ApprovePaper` with a paper
order plan.

Risk Governor may veto any owner request. Rejection returns stable numeric-path
reason codes:

- `OwnerRequestedButRiskDenied`
- `OwnerRequestedButLowEdge`
- `OwnerRequestedButBadSpread`
- `OwnerRequestedButStaleData`
- `OwnerRequestedButLowConfidence`
- `OwnerRequestedButUnknownRegime`
- `OwnerRequestedButPolicyBlocked`
- `OwnerRequestedButAgentInCooldown`
- `OwnerRequestedButDoctrineViolation`

Chair/Risk rejection explanations use fixed templates derived from these reason
codes. No runtime LLM is required or allowed. The template explains the
observed policy/risk cause without changing the underlying decision.
