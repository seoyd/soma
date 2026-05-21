#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use soma_zero::{
    BaselineBackfillSource, BaselineReferenceBackfillPlan, BaselineReferenceBackfillPlanItem,
    ComparableCommitteeEvidenceBundle, ComparableCommitteeEvidenceConfig,
    ComparableCommitteeEvidenceRow, ComparableEvidenceSourceClass, CompleteRowClosureConfig,
    CounterfactualBackfillGapKind, CounterfactualBackfillPlan, CounterfactualBackfillPlanItem,
    CounterfactualBackfillSuggestedAction, OfficialReadyRowCompletenessStatus,
    OfficialReadyRowInventoryItem, OfficialReadyRowInventoryReport,
    OfficialReadyRowInventoryStatus, OutcomeBackfillGapKind, OutcomeBackfillSuggestedAction,
    OutcomeReferenceBackfillPlan, OutcomeReferenceBackfillPlanItem, ProviderMarket,
    ScenarioMaterializationV3Level, ScenarioMaterializationV3Record,
    ScenarioMaterializationV3Report, ScenarioMaterializationV3Status,
};

pub fn row(id: &str) -> ComparableCommitteeEvidenceRow {
    ComparableCommitteeEvidenceRow {
        row_id: id.to_string(),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        timeframe: "1d".to_string(),
        horizon_bars: 24,
        timestamp_ms: 1_700_000_000_000,
        source_kind: "OfficialApiCollected".to_string(),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        scenario_row_id: Some(format!("scenario-{id}")),
        committee_decision_id: Some(format!("committee-{id}")),
        committee_final_action: "Approve".to_string(),
        chair_decision: Some("Approve".to_string()),
        risk_governor_decision: Some("Allow".to_string()),
        baseline_action: Some("Approve".to_string()),
        external_action: None,
        no_trade_baseline_action: "NoTrade".to_string(),
        outcome_label: Some("TakeProfit".to_string()),
        net_return_pct: Some(0.03),
        cost_bps: 5.0,
        slippage_bps: 2.0,
        committee_vs_baseline_delta: Some(0.01),
        committee_vs_notrade_delta: Some(0.03),
        risk_denied_value_proxy: Some(-0.01),
        no_trade_value_proxy: Some(0.0),
        outcome_reference_available: true,
        baseline_reference_available: true,
        no_trade_counterfactual_available: true,
        risk_denied_counterfactual_available: true,
        external_reference_available: false,
        row_level: true,
        summary_derived: false,
        no_lookahead_safe: true,
        official_readiness_eligible: true,
        diagnostic_only: false,
        candle_coverage_available: true,
        matched_candle_series_id: Some("series-aapl-1d".to_string()),
        candle_match_status: Some("Matched".to_string()),
        candle_official_ready_match: true,
        candle_benchmark_ready_match: true,
        candle_diagnostic_only: false,
        reason_codes: Vec::new(),
    }
}

pub fn materialization_record(
    row_id: &str,
    level: ScenarioMaterializationV3Level,
    diagnostic_only: bool,
) -> ScenarioMaterializationV3Record {
    ScenarioMaterializationV3Record {
        row_id: row_id.to_string(),
        scenario_row_id: format!("scenario-{row_id}"),
        materialization_level: level,
        official_ready_match_used: true,
        candle_series_id: Some("series-aapl-1d".to_string()),
        feature_summary_available: level != ScenarioMaterializationV3Level::Rejected,
        limited_feature_summary: matches!(
            level,
            ScenarioMaterializationV3Level::LimitedFeatureProjected
                | ScenarioMaterializationV3Level::SummaryDerivedDiagnostic
        ),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        diagnostic_only,
        reason_codes: Vec::new(),
    }
}

pub fn materialization_report(
    records: Vec<ScenarioMaterializationV3Record>,
) -> ScenarioMaterializationV3Report {
    let materialized_count = records
        .iter()
        .filter(|record| record.materialization_level != ScenarioMaterializationV3Level::Rejected)
        .count();
    let row_level_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.materialization_level,
                ScenarioMaterializationV3Level::ExistingRowLevelScenario
                    | ScenarioMaterializationV3Level::OfficialReadyCandleProjected
                    | ScenarioMaterializationV3Level::CanonicalCsvProjected
            )
        })
        .count();
    let limited_feature_count = records
        .iter()
        .filter(|record| record.limited_feature_summary)
        .count();
    let rejected_count = records
        .iter()
        .filter(|record| record.materialization_level == ScenarioMaterializationV3Level::Rejected)
        .count();
    let official_materialized_count = materialized_count;
    let diagnostic_only_count = records
        .iter()
        .filter(|record| record.diagnostic_only)
        .count();
    ScenarioMaterializationV3Report {
        records,
        materialized_count,
        row_level_count,
        limited_feature_count,
        rejected_count,
        official_materialized_count,
        diagnostic_only_count,
        materialization_status: if materialized_count > 0 {
            ScenarioMaterializationV3Status::OfficialRowLevelMaterialized
        } else {
            ScenarioMaterializationV3Status::StillMissingScenarioRows
        },
        reason_codes: Vec::new(),
    }
}

pub fn outcome_plan_item(row_id: &str, can_build: bool) -> OutcomeReferenceBackfillPlanItem {
    OutcomeReferenceBackfillPlanItem {
        row_id: row_id.to_string(),
        gap_kind: if can_build {
            OutcomeBackfillGapKind::MissingTripleBarrierOutcome
        } else {
            OutcomeBackfillGapKind::MissingOutcomeWindow
        },
        can_build_from_candles: can_build,
        required_horizon_bars: 24,
        required_future_window: 25,
        suggested_action: if can_build {
            OutcomeBackfillSuggestedAction::BuildTripleBarrierOutcome
        } else {
            OutcomeBackfillSuggestedAction::ProvideLongerCandleWindow
        },
        reason_codes: Vec::new(),
    }
}

pub fn outcome_plan(items: Vec<OutcomeReferenceBackfillPlanItem>) -> OutcomeReferenceBackfillPlan {
    OutcomeReferenceBackfillPlan {
        plan_id: "outcome-plan".to_string(),
        buildable_count: items
            .iter()
            .filter(|item| item.can_build_from_candles)
            .count(),
        unavailable_count: items
            .iter()
            .filter(|item| !item.can_build_from_candles)
            .count(),
        missing_future_window_count: items
            .iter()
            .filter(|item| item.gap_kind == OutcomeBackfillGapKind::MissingFutureBars)
            .count(),
        no_lookahead_blocked_count: items
            .iter()
            .filter(|item| item.gap_kind == OutcomeBackfillGapKind::NoLookaheadViolation)
            .count(),
        items,
        reason_codes: Vec::new(),
    }
}

pub fn baseline_plan_item(
    row_id: &str,
    source: BaselineBackfillSource,
    can_backfill: bool,
    diagnostic_only: bool,
) -> BaselineReferenceBackfillPlanItem {
    BaselineReferenceBackfillPlanItem {
        row_id: row_id.to_string(),
        source,
        can_backfill,
        diagnostic_only,
        reason_codes: Vec::new(),
    }
}

pub fn baseline_plan(
    items: Vec<BaselineReferenceBackfillPlanItem>,
) -> BaselineReferenceBackfillPlan {
    BaselineReferenceBackfillPlan {
        plan_id: "baseline-plan".to_string(),
        existing_artifact_count: items
            .iter()
            .filter(|item| item.source == BaselineBackfillSource::ExistingBaselineArtifact)
            .count(),
        no_trade_fallback_count: items
            .iter()
            .filter(|item| item.source == BaselineBackfillSource::DeterministicNoTradeBaseline)
            .count(),
        approximation_count: items
            .iter()
            .filter(|item| {
                item.source == BaselineBackfillSource::DeterministicBaselineApproximation
            })
            .count(),
        unavailable_count: items
            .iter()
            .filter(|item| item.source == BaselineBackfillSource::Unavailable)
            .count(),
        items,
        reason_codes: Vec::new(),
    }
}

pub fn counterfactual_plan_item(
    row_id: &str,
    can_build_no_trade: bool,
    can_build_risk_denied: bool,
) -> CounterfactualBackfillPlanItem {
    CounterfactualBackfillPlanItem {
        row_id: row_id.to_string(),
        gap_kind: if can_build_risk_denied {
            CounterfactualBackfillGapKind::MissingRiskDeniedCounterfactual
        } else {
            CounterfactualBackfillGapKind::MissingNoTradeCounterfactual
        },
        can_build_no_trade,
        can_build_risk_denied,
        suggested_action: if can_build_risk_denied {
            CounterfactualBackfillSuggestedAction::BuildRiskDeniedCounterfactual
        } else if can_build_no_trade {
            CounterfactualBackfillSuggestedAction::BuildNoTradeCounterfactual
        } else {
            CounterfactualBackfillSuggestedAction::NoSafeAction
        },
        reason_codes: Vec::new(),
    }
}

pub fn counterfactual_plan(
    items: Vec<CounterfactualBackfillPlanItem>,
) -> CounterfactualBackfillPlan {
    CounterfactualBackfillPlan {
        plan_id: "counterfactual-plan".to_string(),
        no_trade_buildable_count: items.iter().filter(|item| item.can_build_no_trade).count(),
        risk_denied_buildable_count: items
            .iter()
            .filter(|item| item.can_build_risk_denied)
            .count(),
        unavailable_count: items
            .iter()
            .filter(|item| !item.can_build_no_trade && !item.can_build_risk_denied)
            .count(),
        no_lookahead_blocked_count: items
            .iter()
            .filter(|item| item.gap_kind == CounterfactualBackfillGapKind::NoLookaheadViolation)
            .count(),
        items,
        reason_codes: Vec::new(),
    }
}

pub fn inventory_item(row_id: &str) -> OfficialReadyRowInventoryItem {
    OfficialReadyRowInventoryItem {
        row_id: row_id.to_string(),
        scenario_row_id: Some(format!("scenario-{row_id}")),
        comparable_row_id: Some(row_id.to_string()),
        candle_series_id: Some("series-aapl-1d".to_string()),
        symbol: "AAPL".to_string(),
        market: ProviderMarket::USEquity,
        venue: Some("NASDAQ".to_string()),
        timeframe: "1d".to_string(),
        horizon_bars: 24,
        timestamp_ms: 1_700_000_000_000,
        source_kind: "OfficialApiCollected".to_string(),
        source_class: ComparableEvidenceSourceClass::OfficialNonCrypto,
        official_ready_match: true,
        benchmark_ready_match: true,
        row_level: true,
        summary_derived: false,
        no_lookahead_safe: true,
        has_scenario_row: true,
        has_committee_decision: true,
        has_chair_decision: true,
        has_risk_decision: true,
        has_outcome_reference: true,
        has_baseline_reference: true,
        has_no_trade_counterfactual: true,
        has_risk_denied_counterfactual: true,
        has_external_reference: false,
        completeness_statuses: vec![OfficialReadyRowCompletenessStatus::CompleteComparableRow],
        buildable_from_available_artifacts: true,
        reason_codes: Vec::new(),
    }
}

pub fn inventory_report(
    items: Vec<OfficialReadyRowInventoryItem>,
) -> OfficialReadyRowInventoryReport {
    let total_items = items.len();
    OfficialReadyRowInventoryReport {
        inventory_id: "inventory".to_string(),
        official_ready_match_count: items
            .iter()
            .filter(|item| item.official_ready_match)
            .count(),
        benchmark_ready_match_count: items
            .iter()
            .filter(|item| item.benchmark_ready_match)
            .count(),
        complete_comparable_row_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    == vec![OfficialReadyRowCompletenessStatus::CompleteComparableRow]
            })
            .count(),
        incomplete_row_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    != vec![OfficialReadyRowCompletenessStatus::CompleteComparableRow]
            })
            .count(),
        missing_outcome_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::MissingOutcomeReference)
            })
            .count(),
        missing_baseline_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::MissingBaselineReference)
            })
            .count(),
        missing_no_trade_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::MissingNoTradeCounterfactual)
            })
            .count(),
        missing_risk_denied_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::MissingRiskDeniedCounterfactual)
            })
            .count(),
        missing_scenario_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::MissingScenarioRow)
            })
            .count(),
        summary_derived_only_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::SummaryDerivedOnly)
            })
            .count(),
        source_ineligible_count: items
            .iter()
            .filter(|item| {
                item.completeness_statuses
                    .contains(&OfficialReadyRowCompletenessStatus::SourceIneligible)
            })
            .count(),
        inventory_status: if total_items > 0 {
            OfficialReadyRowInventoryStatus::HealthyCompleteRows
        } else {
            OfficialReadyRowInventoryStatus::InsufficientRows
        },
        items,
        total_items,
        reason_codes: Vec::new(),
    }
}

pub fn write_bundle(name: &str, rows: Vec<ComparableCommitteeEvidenceRow>) -> PathBuf {
    let config = ComparableCommitteeEvidenceConfig {
        comparable_id: name.to_string(),
        output_root: output_dir(&format!("{name}-bundle-out"))
            .display()
            .to_string(),
        ..ComparableCommitteeEvidenceConfig::default()
    };
    let bundle = ComparableCommitteeEvidenceBundle::from_rows(&config, rows);
    let dir = output_dir(&format!("{name}-bundle"));
    bundle.write_to_dir(&dir).expect("write bundle")
}

pub fn closure_config(name: &str, bundle_path: PathBuf) -> CompleteRowClosureConfig {
    CompleteRowClosureConfig {
        closure_id: name.to_string(),
        comparable_evidence_config_path: Some(bundle_path.display().to_string()),
        output_root: output_dir(&format!("{name}-closure")).display().to_string(),
        ..CompleteRowClosureConfig::default()
    }
}

pub fn write_file(path: PathBuf, contents: &str) -> PathBuf {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create dir");
    }
    fs::write(&path, contents).expect("write file");
    path
}

fn output_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("sprint45-tests")
        .join(name);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create dir");
    path
}
