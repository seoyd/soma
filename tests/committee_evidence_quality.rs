use soma_zero::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioSet,
    CommitteeScenarioSourceKind, ReasonCode, build_committee_evidence_quality_report,
};

fn scenario_set(source_kind: CommitteeScenarioSourceKind) -> CommitteeScenarioSet {
    CommitteeScenarioLoader::default()
        .load(&CommitteeScenarioLoadConfig {
            scenario_id: format!("quality-{source_kind:?}"),
            source_kind,
            reason_codes: vec![ReasonCode::CommitteeScenarioLoaderBuilt],
            ..CommitteeScenarioLoadConfig::default()
        })
        .expect("load")
}

#[test]
fn fixture_and_yfinance_quality_are_conservative() {
    let fixture = build_committee_evidence_quality_report(&scenario_set(
        CommitteeScenarioSourceKind::Fixture,
    ));
    let yfinance = build_committee_evidence_quality_report(&scenario_set(
        CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
    ));
    assert_eq!(
        fixture.quality_status,
        soma_zero::CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
    );
    assert_eq!(
        yfinance.quality_status,
        soma_zero::CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
    );
    assert!(!fixture.enough_for_design_review);
    assert!(!yfinance.enough_for_design_review);
}

#[test]
fn official_quality_counts_and_missing_provenance_are_reported() {
    let mut set = scenario_set(CommitteeScenarioSourceKind::OfficialBenchmarkReport);
    set.rows[0].provenance_summary.clear();
    set.rows[0].data_quality_score = 0.5;
    let report = build_committee_evidence_quality_report(&set);
    assert_eq!(report.official_count, 1);
    assert_eq!(report.missing_provenance_count, 1);
    assert_eq!(report.low_quality_count, 1);
}
