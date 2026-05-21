#[path = "support/sprint45_support.rs"]
mod sprint45_support;

use soma_zero::{
    BaselineBackfillSource, ComparableEvidenceSourceClass, CompleteComparableRowBuildStatus,
    CompleteComparableRowBuilder, CompleteComparableRowBuilderConfig,
    ScenarioMaterializationV3Level,
};

#[test]
fn builder_requires_scenario_and_all_references_for_official_complete() {
    let mut row = sprint45_support::row("a");
    row.outcome_reference_available = false;
    row.baseline_reference_available = false;
    row.no_trade_counterfactual_available = false;
    row.risk_denied_counterfactual_available = false;
    let bundle = CompleteComparableRowBuilder::default().build(
        &CompleteComparableRowBuilderConfig::default(),
        &[row],
        &sprint45_support::materialization_report(vec![sprint45_support::materialization_record(
            "a",
            ScenarioMaterializationV3Level::ExistingRowLevelScenario,
            false,
        )]),
        &sprint45_support::outcome_plan(vec![sprint45_support::outcome_plan_item("a", true)]),
        &sprint45_support::baseline_plan(vec![sprint45_support::baseline_plan_item(
            "a",
            BaselineBackfillSource::ExistingBaselineArtifact,
            true,
            false,
        )]),
        &sprint45_support::counterfactual_plan(vec![sprint45_support::counterfactual_plan_item(
            "a", true, true,
        )]),
    );
    assert_eq!(
        bundle.build_records[0].status,
        CompleteComparableRowBuildStatus::BuiltComplete
    );
    assert_eq!(bundle.official_complete_rows, 1);
}

#[test]
fn builder_skips_missing_outcome_and_counterfactuals() {
    let mut row = sprint45_support::row("b");
    row.outcome_reference_available = false;
    row.no_trade_counterfactual_available = false;
    row.risk_denied_counterfactual_available = false;
    let bundle = CompleteComparableRowBuilder::default().build(
        &CompleteComparableRowBuilderConfig::default(),
        &[row],
        &sprint45_support::materialization_report(vec![sprint45_support::materialization_record(
            "b",
            ScenarioMaterializationV3Level::ExistingRowLevelScenario,
            false,
        )]),
        &sprint45_support::outcome_plan(vec![sprint45_support::outcome_plan_item("b", false)]),
        &sprint45_support::baseline_plan(vec![]),
        &sprint45_support::counterfactual_plan(vec![]),
    );
    assert_eq!(
        bundle.build_records[0].status,
        CompleteComparableRowBuildStatus::SkippedMissingOutcome
    );
}

#[test]
fn builder_keeps_controlled_and_yfinance_out_of_official_complete_and_is_deterministic() {
    let mut controlled = sprint45_support::row("controlled");
    controlled.source_class = ComparableEvidenceSourceClass::ControlledDiagnostic;
    let mut yfinance = sprint45_support::row("yf");
    yfinance.source_class = ComparableEvidenceSourceClass::YFinanceResearch;
    let config = CompleteComparableRowBuilderConfig {
        allow_diagnostic_complete: true,
        allow_controlled_diagnostic: true,
        allow_yfinance_research: true,
        ..CompleteComparableRowBuilderConfig::default()
    };
    let report = sprint45_support::materialization_report(vec![
        sprint45_support::materialization_record(
            "controlled",
            ScenarioMaterializationV3Level::ExistingRowLevelScenario,
            true,
        ),
        sprint45_support::materialization_record(
            "yf",
            ScenarioMaterializationV3Level::ExistingRowLevelScenario,
            true,
        ),
    ]);
    let first = CompleteComparableRowBuilder::default().build(
        &config,
        &[controlled.clone(), yfinance.clone()],
        &report,
        &sprint45_support::outcome_plan(vec![]),
        &sprint45_support::baseline_plan(vec![]),
        &sprint45_support::counterfactual_plan(vec![]),
    );
    let second = CompleteComparableRowBuilder::default().build(
        &config,
        &[controlled, yfinance],
        &report,
        &sprint45_support::outcome_plan(vec![]),
        &sprint45_support::baseline_plan(vec![]),
        &sprint45_support::counterfactual_plan(vec![]),
    );
    assert_eq!(first.to_text(), second.to_text());
    assert_eq!(first.official_complete_rows, 0);
}
