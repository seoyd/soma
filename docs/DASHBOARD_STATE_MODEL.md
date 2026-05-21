# Dashboard State Model

`DashboardState` is the deterministic snapshot for Soma Control Tower v0.

Core fields:
- `dashboard_id`
- `generated_from_reports`
- `system_mode`
- provider/evidence/committee/chair/risk panels
- candidate queue
- paper position panel
- human confirm panel
- bottleneck panel
- audit timeline
- warnings/blockers/reason codes/fingerprint

Candidate lifecycle:
`Detected -> UnderAnalysis -> CommitteeVoting -> ChairReviewed -> RiskReview -> Candidate -> HumanConfirmRequired -> PaperApproved -> PaperPositionOpen -> PaperClosed`

Additional conservative states:
- `NoTrade`
- `RiskBlocked`
- `Expired`
- `DiagnosticOnly`

Committee members expose only numeric/archetype status. Paper positions are simulated only. Human confirm remains view-only and cannot trigger execution.
