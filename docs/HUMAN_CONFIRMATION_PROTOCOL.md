# Human Confirmation Protocol

Sprint 53 introduces a deterministic **paper-only** human confirmation protocol.

## Rules

- `PaperConfirm` never creates a real order.
- `PaperConfirmed` is terminal for the **paper path only**.
- `RiskBlocked` cannot transition to `PaperConfirmed`.
- `NoTrade` can move to reviewed/dismissed states, not paper-confirmed.
- research-only and diagnostic-only confirmation remain blocked by default.

## Allowed transitions

- `PendingReview + MarkReviewed -> Reviewed`
- `PendingReview + CandidateDismiss -> Dismissed`
- `PendingReview + CandidateHold -> Deferred`
- `HumanConfirmRequired + PaperConfirm -> PaperConfirmed` when policy allows paper-only confirmation

## Forbidden transitions

- any transition to live order execution
- `RiskBlocked + PaperConfirm`
- `NoTrade + PaperConfirm`
- owner actions that loosen a hard veto or bypass risk
