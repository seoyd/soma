# Candidate Lifecycle

Sprint 56 adds a deterministic candidate lifecycle.

## Main states

- Detected
- EvidenceReady
- UnderAnalysis
- CommitteeVoting
- ChairReviewed
- RiskReview
- HumanConfirmRequired
- PaperApproved
- PaperPositionOpen
- PaperPositionClosed
- NoTrade
- RiskBlocked
- OwnerHeld
- OwnerDismissed
- ReanalysisRequested
- ResearchOnly
- DiagnosticOnly
- Error

## Forbidden transitions

- RiskBlocked -> PaperApproved
- NoTrade -> PaperApproved
- ResearchOnly -> Official paper approval
- DiagnosticOnly -> Official paper approval
- PaperApproved -> RealOrder
- PaperPositionOpen -> BrokerPosition
- any -> LiveTrading

## Safety meaning

`PaperApproved` means simulated paper approval only. It is not a real order and does not touch broker state.
