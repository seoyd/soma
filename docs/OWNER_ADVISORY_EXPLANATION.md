# Owner Advisory Explanation

## Policy

Owner input is advisory. The owner may:

- add or remove a watchlist item,
- request review, reanalysis, or more evidence,
- hold or dismiss a candidate,
- add a thesis note,
- request tighter risk,
- provide a paper confirmation.

The owner cannot force a trade, enable live execution, enable a broker, loosen
a hard veto, or bypass Chair and Risk Governor.

## Decision Flow

```text
owner request
-> owner policy validation
-> agent review and votes
-> Chair candidate decision
-> Risk Governor
-> paper-only action or NoTrade
-> stable reason codes and explanation
```

A paper confirmation is eligible only when policy allows it and Risk Governor
independently returned `ApprovePaper` with a paper-only plan.

## Example: "Buy This Now"

The system may return:

```text
advisory_only=true
owner_forced_trade=false
paper_action_allowed=false
reason_codes=[
  OwnerRequestedButRiskDenied,
  OwnerRequestedButLowEdge,
  OwnerRequestedButBadSpread
]
explanation="Risk Governor did not approve the requested paper action.
Expected numeric edge is below policy.
Observed spread exceeds the conservative limit."
```

The response records that the request was considered. It does not translate the
imperative wording into execution authority.

## Stable Rejection Reasons

| Reason code | Fixed explanation meaning |
| --- | --- |
| `OwnerRequestedButRiskDenied` | Risk Governor did not approve |
| `OwnerRequestedButLowEdge` | Numeric edge is below policy |
| `OwnerRequestedButBadSpread` | Spread exceeds the conservative limit |
| `OwnerRequestedButStaleData` | Market data is stale |
| `OwnerRequestedButLowConfidence` | Signal confidence is insufficient |
| `OwnerRequestedButUnknownRegime` | Market regime is unknown |
| `OwnerRequestedButPolicyBlocked` | Requested action is outside owner policy |
| `OwnerRequestedButAgentInCooldown` | Requested agent is in safety cooldown |
| `OwnerRequestedButDoctrineViolation` | Requested action conflicts with immutable doctrine |

`owner_rejection_explanation` sorts stable reason codes and maps them to fixed
templates. It does not invoke a language model and does not alter the underlying
decision.

## Existing Implementation

- `OwnerInputKind` represents watchlist, review, evidence, thesis, hold/dismiss,
  paper-confirm, and risk requests.
- `validate_owner_input` blocks forbidden runtime actions and marks diagnostic
  input.
- `review_owner_trade_request` combines policy, market quality, and
  `RiskDecision`.
- `owner_rejection_explanation` creates deterministic text from reason codes.

## Safety Invariants

- `NoTrade` remains the fallback.
- Missing explanation never turns rejection into approval.
- Owner identity or seniority does not change risk thresholds.
- Free-form text has no direct trading effect.
- Runtime LLM use is unnecessary and forbidden.
- Explanations must not contain credentials, private provider payloads, or raw
  account data.
