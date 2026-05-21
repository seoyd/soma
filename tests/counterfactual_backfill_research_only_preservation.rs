mod support;

use soma_zero::CounterfactualBackfillResearchOnlyPreservationStatus;
use support::sprint69_support as sprint;

#[test]
fn counterfactual_backfill_research_only_preservation_stays_green() {
    let report = sprint::run_sprint97_bundle(
        "soma_sprint97_counterfactual_backfill_recover.toml",
        "counterfactual-research-only",
    )
    .counterfactual_backfill_research_only_preservation_report;
    assert_eq!(
        report.research_status,
        CounterfactualBackfillResearchOnlyPreservationStatus::ResearchOnlyPreserved
    );
}
