#[path = "support/sprint69_support.rs"]
mod support;

use std::collections::BTreeSet;

use soma_zero::{
    BriefingDeltaStatus, BriefingSectionKind, BriefingSeverity, OperatorBriefingConfig,
};

#[test]
fn operator_briefing_matches_expected_fixture_and_generates_all_sections() {
    let bundle = support::run_briefing("soma_operator_briefing.toml", "operator-briefing");
    let expected = support::read_json::<soma_zero::OperatorBriefingReport>(support::example_path(
        "sprint71_data/operator_briefing_expected.json",
    ));
    let _renderer = soma_zero::StaticBriefingRenderer::default();

    assert_eq!(bundle.operator_briefing_report, expected);
    assert_eq!(
        bundle.operator_briefing_report.overall_severity,
        BriefingSeverity::NeedsAction
    );

    let kinds = bundle
        .operator_briefing_report
        .sections
        .iter()
        .map(|section| section.section_kind)
        .collect::<BTreeSet<_>>();
    for kind in [
        BriefingSectionKind::SystemHealth,
        BriefingSectionKind::KISData,
        BriefingSectionKind::Evidence,
        BriefingSectionKind::Committee,
        BriefingSectionKind::Chair,
        BriefingSectionKind::Risk,
        BriefingSectionKind::OwnerReview,
        BriefingSectionKind::PaperOps,
        BriefingSectionKind::SequenceDataset,
        BriefingSectionKind::ExternalModels,
        BriefingSectionKind::Leaderboard,
        BriefingSectionKind::DiffTrace,
        BriefingSectionKind::DeferredItems,
        BriefingSectionKind::NextActions,
    ] {
        assert!(kinds.contains(&kind), "missing section kind: {kind:?}");
    }

    assert!(!bundle.daily_briefing_snapshot.sections_hash.is_empty());
    assert!(
        !bundle
            .daily_briefing_snapshot
            .source_artifact_hash
            .is_empty()
    );
    assert_eq!(
        bundle.daily_briefing_snapshot.owner_attention_count,
        bundle.operator_briefing_report.owner_attention_items.len()
    );
    assert_eq!(
        bundle.daily_briefing_snapshot.deferred_count,
        bundle.operator_briefing_report.deferred_items.len()
    );
    assert_eq!(
        bundle
            .briefing_delta_report
            .as_ref()
            .map(|item| item.report_status),
        Some(BriefingDeltaStatus::DeltaReady)
    );
}

#[test]
fn operator_briefing_config_validation_and_limits_work() {
    let mut config = OperatorBriefingConfig::default();
    assert!(config.validate().is_ok());

    config.unexpected_diff_triage_paths = vec!["https://example.com/remote.toml".to_string()];
    assert!(config.validate().is_err());

    let mut limited = support::briefing_config_from_example(
        "soma_operator_briefing.toml",
        "operator-briefing-limited",
    );
    limited.max_sections = 3;
    let limited_bundle = soma_zero::OperatorBriefingRunner::default()
        .run(&limited)
        .expect("limited run");
    assert_eq!(limited_bundle.operator_briefing_report.sections.len(), 3);

    let mut limited_items = support::briefing_config_from_example(
        "soma_operator_briefing.toml",
        "operator-briefing-limited-items",
    );
    limited_items.max_items_per_section = 2;
    let limited_items_bundle = soma_zero::OperatorBriefingRunner::default()
        .run(&limited_items)
        .expect("limited items run");
    for section in &limited_items_bundle.operator_briefing_report.sections {
        assert!(section.bullet_items.len() <= 2);
        assert!(section.blockers.len() <= 2);
        assert!(section.warnings.len() <= 2);
        assert!(section.next_actions.len() <= 2);
    }

    let mut limited_commands = support::briefing_config_from_example(
        "soma_operator_briefing.toml",
        "operator-briefing-limited-commands",
    );
    limited_commands.max_commands = 3;
    let limited_commands_bundle = soma_zero::OperatorBriefingRunner::default()
        .run(&limited_commands)
        .expect("limited commands run");
    assert!(
        limited_commands_bundle
            .operator_briefing_report
            .copyable_commands
            .len()
            <= 3
    );

    let mut tiny_budget = support::briefing_config_from_example(
        "soma_operator_briefing.toml",
        "operator-briefing-tiny-budget",
    );
    tiny_budget.max_bytes = 32;
    let err = soma_zero::OperatorBriefingRunner::default()
        .run(&tiny_budget)
        .expect_err("budget should fail");
    assert!(err.contains("exceeded max_bytes"));

    let serialized = toml::to_string(&OperatorBriefingConfig::default()).expect("config toml");
    assert!(!serialized.contains("live"));
    assert!(!serialized.contains("training"));
    assert!(!serialized.contains("broker"));
    assert!(!serialized.contains("order"));
    assert!(!serialized.contains("account"));
}

#[test]
fn operator_briefing_delta_without_previous_report_is_reported_conservatively() {
    let mut config = support::briefing_config_from_example(
        "soma_operator_briefing.toml",
        "operator-briefing-no-previous",
    );
    config.previous_briefing_paths.clear();
    let report = soma_zero::OperatorBriefingRunner::default()
        .run_briefing_delta(&config)
        .expect("delta report");
    assert_eq!(
        report.report_status,
        BriefingDeltaStatus::NoPreviousBriefing
    );
}
