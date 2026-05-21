mod common;
#[path = "support/sprint45_support.rs"]
mod sprint45_support;
#[path = "support/sprint46_support.rs"]
mod sprint46_support;

use soma_zero::{
    ComparableEvidenceSourceClass, CompleteRowClosureRunner, CompleteRowClosureStatus,
    CompleteRowClosureV2Runner, CompleteRowClosureV2Status,
};

#[test]
fn complete_row_closure_v2_improves_official_complete_rows() {
    let bundle = CompleteRowClosureV2Runner::default()
        .run(&sprint46_support::closure_v2_config(
            "complete-row-closure-suite-v2",
        ))
        .expect("closure v2");
    assert!(bundle.complete_row_closure_v2_report.after_complete_rows >= 1);
    assert!(
        bundle
            .complete_row_closure_v2_report
            .after_official_complete_rows
            >= 1
    );
    assert!(matches!(
        bundle.complete_row_closure_v2_report.closure_status,
        CompleteRowClosureV2Status::OfficialCompleteRowsImproved
            | CompleteRowClosureV2Status::CompleteRowsImproved
    ));
}

#[test]
fn closure_reports_missing_outcome_when_not_buildable() {
    let mut row = sprint45_support::row("missing-outcome");
    row.outcome_reference_available = false;
    row.candle_coverage_available = false;
    row.candle_official_ready_match = true;
    let bundle_path = sprint45_support::write_bundle("closure-suite-missing-outcome", vec![row]);
    let config = sprint45_support::closure_config("closure-suite-missing-outcome", bundle_path);
    let bundle = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("closure");
    assert!(matches!(
        bundle.complete_row_closure_report.closure_status,
        CompleteRowClosureStatus::StillMissingOutcomeReferences
            | CompleteRowClosureStatus::StillScenarioMaterializationWeak
    ));
}

#[test]
fn closure_improves_with_safe_backfills() {
    let mut row = sprint45_support::row("improve");
    row.outcome_reference_available = false;
    row.baseline_reference_available = false;
    row.no_trade_counterfactual_available = false;
    row.risk_denied_counterfactual_available = false;
    let bundle_path = sprint45_support::write_bundle("closure-suite-improves", vec![row]);
    let config = sprint45_support::closure_config("closure-suite-improves", bundle_path);
    let bundle = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("closure");
    assert!(bundle.complete_row_closure_report.added_complete_rows >= 1);
    assert!(bundle.complete_row_closure_report.after_complete_rows >= 1);
}

#[test]
fn closure_preserves_controlled_crypto_and_research_boundaries() {
    let mut controlled = sprint45_support::row("controlled");
    controlled.source_class = ComparableEvidenceSourceClass::ControlledDiagnostic;
    let mut crypto = sprint45_support::row("crypto");
    crypto.source_class = ComparableEvidenceSourceClass::OfficialCryptoOnly;
    let mut research = sprint45_support::row("research");
    research.source_class = ComparableEvidenceSourceClass::YFinanceResearch;
    let bundle_path = sprint45_support::write_bundle(
        "closure-suite-boundaries",
        vec![controlled, crypto, research],
    );
    let config = sprint45_support::closure_config("closure-suite-boundaries", bundle_path);
    let bundle = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("closure");
    assert_eq!(
        bundle.complete_comparable_row_bundle.official_complete_rows,
        0
    );
}

#[test]
fn closure_scopes_progress_to_official_ready_inventory_rows() {
    let mut row = sprint45_support::row("non-official-ready");
    row.candle_official_ready_match = false;
    row.outcome_reference_available = false;
    row.baseline_reference_available = false;
    row.no_trade_counterfactual_available = false;
    row.risk_denied_counterfactual_available = false;
    let bundle_path =
        sprint45_support::write_bundle("closure-suite-scope-official-ready", vec![row]);
    let config =
        sprint45_support::closure_config("closure-suite-scope-official-ready", bundle_path);
    let bundle = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("closure");
    assert_eq!(
        bundle.complete_row_closure_report.before_complete_rows,
        Some(0)
    );
    assert_eq!(bundle.complete_row_closure_report.after_complete_rows, 0);
    assert_eq!(
        bundle.complete_row_closure_report.added_outcome_references,
        0
    );
    assert!(matches!(
        bundle.complete_row_closure_report.closure_status,
        CompleteRowClosureStatus::StillScenarioMaterializationWeak
            | CompleteRowClosureStatus::NoImprovement
    ));
}

#[test]
fn closure_rejects_no_lookahead_unsafe_rows() {
    let mut row = sprint45_support::row("no-lookahead");
    row.no_lookahead_safe = false;
    row.outcome_reference_available = false;
    row.baseline_reference_available = false;
    row.no_trade_counterfactual_available = false;
    row.risk_denied_counterfactual_available = false;
    let bundle_path = sprint45_support::write_bundle("closure-suite-no-lookahead", vec![row]);
    let config = sprint45_support::closure_config("closure-suite-no-lookahead", bundle_path);
    let bundle = CompleteRowClosureRunner::default()
        .run(&config)
        .expect("closure");
    assert_eq!(bundle.complete_row_closure_report.after_complete_rows, 0);
    assert_eq!(bundle.complete_row_closure_report.added_complete_rows, 0);
    assert_eq!(
        bundle.complete_comparable_row_bundle.official_complete_rows,
        0
    );
}
