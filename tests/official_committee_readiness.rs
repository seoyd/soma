mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    CommitteeOutcomeLinker, OfficialCommitteeEvidenceReadinessStatus,
    OfficialCommitteeScenarioPackBuilder, build_official_committee_evidence_readiness_report,
};

#[test]
fn research_fixture_crypto_and_summary_dominance_block_readiness() {
    let yfinance = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::yfinance_pack_config(
            "official-readiness-yf",
        ))
        .expect("yfinance");
    assert_eq!(
        build_official_committee_evidence_readiness_report(
            &yfinance, None, 1, 1, 1, 0, 0, 0.4, 0.0, 0.0
        )
        .readiness_status,
        OfficialCommitteeEvidenceReadinessStatus::NotReadyResearchOnly
    );

    let fixture = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::fixture_pack_config(
            "official-readiness-fixture",
        ))
        .expect("fixture");
    assert_eq!(
        build_official_committee_evidence_readiness_report(
            &fixture, None, 1, 1, 1, 0, 0, 0.4, 0.0, 0.0
        )
        .readiness_status,
        OfficialCommitteeEvidenceReadinessStatus::NotReadyFixtureOnly
    );

    let summary = OfficialCommitteeScenarioPackBuilder::default()
        .build(&official_committee_support::fixture_pack_config(
            "official-readiness-summary",
        ))
        .expect("summary");
    assert!(summary.summary_derived_ratio() > 0.5);
}

#[test]
fn no_lookahead_violation_and_sufficient_rows_behave_as_expected() {
    let pack_cfg =
        official_committee_support::controlled_pack_config("official-readiness-linked", false);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&pack_cfg)
        .expect("pack");
    let pack_dir = common::output_dir("official-readiness-linked-pack");
    pack.write_to_dir(&pack_dir).expect("write");

    let blocked = CommitteeOutcomeLinker::default()
        .link_from_config(&official_committee_support::controlled_linker_config(
            "official-readiness-blocked",
            &pack_dir.join("official_scenario_pack.json"),
            false,
        ))
        .expect("blocked");
    assert_eq!(
        build_official_committee_evidence_readiness_report(
            &pack,
            Some(&blocked),
            3,
            3,
            3,
            1,
            1,
            0.4,
            0.0,
            0.0
        )
        .readiness_status,
        OfficialCommitteeEvidenceReadinessStatus::NotReadyNoLookaheadViolation
    );

    let ready = CommitteeOutcomeLinker::default()
        .link_from_config(&official_committee_support::controlled_linker_config(
            "official-readiness-ready",
            &pack_dir.join("official_scenario_pack.json"),
            true,
        ))
        .expect("ready");
    let report = build_official_committee_evidence_readiness_report(
        &pack,
        Some(&ready),
        3,
        3,
        3,
        1,
        1,
        0.4,
        0.0,
        0.0,
    );
    assert_eq!(
        report.readiness_status,
        OfficialCommitteeEvidenceReadinessStatus::ReadyForOfficialCommitteeBenchmark
    );
    assert!(!report.enough_for_six_person_design_review || report.enough_for_committee_benchmark);
}
