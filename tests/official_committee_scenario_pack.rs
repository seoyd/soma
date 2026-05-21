mod common;
#[path = "support/official_committee_support.rs"]
mod official_committee_support;

use soma_zero::{
    OfficialCommitteePackSourceKind, OfficialCommitteeScenarioPackBuilder,
    OfficialCommitteeScenarioPackConfig,
};

#[test]
fn evidence_lane_row_level_rows_are_included_and_storage_counted() {
    let cfg = official_committee_support::crypto_pack_config("official-pack-evidence");
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&cfg)
        .expect("build pack");
    assert_eq!(pack.row_count(), 3);
    assert_eq!(pack.official_row_count, 3);
    assert_eq!(pack.crypto_only_row_count, 3);
    assert_eq!(pack.row_level_count, 3);
    assert!(pack.storage_bytes > 0);
}

#[test]
fn official_rows_require_provenance_and_preflight_when_configured() {
    let cfg = official_committee_support::controlled_pack_config("official-pack-preflight", true);
    let pack = OfficialCommitteeScenarioPackBuilder::default()
        .build(&cfg)
        .expect("build pack");
    assert_eq!(pack.row_count(), 0);
}

#[test]
fn yfinance_and_fixture_rows_are_excluded_when_not_allowed() {
    let yfinance = OfficialCommitteeScenarioPackConfig {
        input_artifact_paths: vec!["virtual-yfinance".to_string()],
        allowed_source_kinds: vec![OfficialCommitteePackSourceKind::YFinanceResearch],
        output_root: common::output_dir("official-pack-yfinance")
            .display()
            .to_string(),
        allow_yfinance_research: false,
        allow_summary_derived_rows: true,
        require_preflight: false,
        require_provenance: false,
        ..OfficialCommitteeScenarioPackConfig::default()
    };
    let fixture = OfficialCommitteeScenarioPackConfig {
        input_artifact_paths: vec!["virtual-fixture".to_string()],
        allowed_source_kinds: vec![OfficialCommitteePackSourceKind::Fixture],
        output_root: common::output_dir("official-pack-fixture")
            .display()
            .to_string(),
        allow_fixture: false,
        allow_summary_derived_rows: true,
        require_preflight: false,
        require_provenance: false,
        ..OfficialCommitteeScenarioPackConfig::default()
    };
    assert_eq!(
        OfficialCommitteeScenarioPackBuilder::default()
            .build(&yfinance)
            .expect("yfinance")
            .row_count(),
        0
    );
    assert_eq!(
        OfficialCommitteeScenarioPackBuilder::default()
            .build(&fixture)
            .expect("fixture")
            .row_count(),
        0
    );
}

#[test]
fn summary_derived_rows_are_reason_coded_and_pack_is_deterministic() {
    let cfg = official_committee_support::fixture_pack_config("official-pack-summary");
    let first = OfficialCommitteeScenarioPackBuilder::default()
        .build(&cfg)
        .expect("first");
    let second = OfficialCommitteeScenarioPackBuilder::default()
        .build(&cfg)
        .expect("second");
    assert!(first.summary_derived_count > 0);
    assert_eq!(first.to_text(), second.to_text());
}
