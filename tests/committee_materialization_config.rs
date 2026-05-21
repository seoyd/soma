mod common;

use soma_zero::{
    CommitteeArtifactKind, CommitteeMaterializationConfig, CommitteeScenarioMaterializerV2,
};

#[test]
fn materialization_config_can_be_constructed() {
    let cfg = CommitteeMaterializationConfig::default();
    assert!(cfg.prefer_row_level_artifacts);
    assert!(cfg.max_rows <= 100);
    assert!(
        cfg.allowed_artifact_kinds
            .contains(&CommitteeArtifactKind::FixtureScenario)
    );
}

#[test]
fn materialization_config_rejects_remote_paths_and_enforces_bounds() {
    let remote = CommitteeMaterializationConfig {
        input_artifact_paths: vec!["https://example.com/data.json".to_string()],
        ..CommitteeMaterializationConfig::default()
    };
    let max_rows = CommitteeMaterializationConfig {
        max_rows: 101,
        ..CommitteeMaterializationConfig::default()
    };
    let max_symbols = CommitteeMaterializationConfig {
        max_symbols: 51,
        ..CommitteeMaterializationConfig::default()
    };
    let max_bytes = CommitteeMaterializationConfig {
        max_bytes: 5_000_001,
        ..CommitteeMaterializationConfig::default()
    };
    assert!(remote.validate().is_err());
    assert!(max_rows.validate().is_err());
    assert!(max_symbols.validate().is_err());
    assert!(max_bytes.validate().is_err());
}

#[test]
fn prefer_row_level_defaults_true_and_summary_fallback_reason_coded() {
    let cfg = CommitteeMaterializationConfig {
        materialization_id: "materialization-fallback".to_string(),
        input_artifact_paths: vec!["virtual-yfinance".to_string()],
        allowed_artifact_kinds: vec![CommitteeArtifactKind::YahooResearchEvidenceReport],
        output_root: common::output_dir("materialization-fallback")
            .display()
            .to_string(),
        allow_summary_derived_rows: true,
        ..CommitteeMaterializationConfig::default()
    };
    let set = CommitteeScenarioMaterializerV2::default()
        .materialize(&cfg)
        .expect("materialize");
    assert!(cfg.prefer_row_level_artifacts);
    assert!(set.rows.iter().any(|row| {
        row.reason_codes
            .contains(&soma_zero::ReasonCode::CommitteeSummaryFallbackUsed)
    }));
}
