use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::{
    control_tower_v1::ControlTowerRefreshPlanner,
    dashboard_panels::{CandidatePanel, RiskPanel},
    kis_monitor_panel::KISMonitorPanel,
    owner_panel::OwnerPanel,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextActionKind {
    RunKISAuthCheck,
    RunKISDryRun,
    RunKISCollectionPlan,
    RunKISMarketDataActivate,
    RunKISCandleSufficiency,
    RunKISOutcomeLinkClose,
    RunKISEvidenceDepth,
    RunControlTowerRefresh,
    RunOperationalRunbook,
    RunDashboardSnapshot,
    RunDashboardRender,
    RunOwnerReviewQueue,
    RunOwnerImpactReport,
    RunCorePerformance,
    RunCommitteeBenchmark,
    ImproveOutcomeLinkDepth,
    ImproveKISEvidence,
    ReviewRiskBlockedCandidate,
    ReviewHumanConfirmCandidate,
    #[default]
    NoAction,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NextActionPriority {
    #[default]
    Required,
    Recommended,
    Optional,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextActionItem {
    pub action_id: String,
    pub action_kind: NextActionKind,
    pub priority: NextActionPriority,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_output_artifact: Option<String>,
    pub safe_to_run: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextActionPanel {
    pub primary_next_action: NextActionItem,
    #[serde(default)]
    pub recommended_actions: Vec<NextActionItem>,
    #[serde(default)]
    pub blocked_actions: Vec<NextActionItem>,
    #[serde(default)]
    pub operator_actions: Vec<NextActionItem>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl NextActionPanel {
    pub fn stabilize(&mut self) {
        for items in [
            &mut self.recommended_actions,
            &mut self.blocked_actions,
            &mut self.operator_actions,
        ] {
            items.sort_by(|left, right| left.action_id.cmp(&right.action_id));
            items.dedup_by(|left, right| left.action_id == right.action_id);
            for item in items.iter_mut() {
                item.reason_codes = stable_reason_codes(&item.reason_codes);
            }
        }
        self.primary_next_action.reason_codes =
            stable_reason_codes(&self.primary_next_action.reason_codes);
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

pub fn build_next_action_panel(
    config_path: Option<&str>,
    output_root: &str,
    kis_monitor_panel: &KISMonitorPanel,
    owner_panel: &OwnerPanel,
    risk_panel: &RiskPanel,
    candidate_panel: &CandidatePanel,
    refresh_planner: &ControlTowerRefreshPlanner,
) -> NextActionPanel {
    let mut recommended_actions = Vec::new();
    let mut blocked_actions = Vec::new();
    let mut operator_actions = Vec::new();
    let mut reason_codes = vec![ReasonCode::DashboardRendered];

    if !kis_monitor_panel.auth_ready || !kis_monitor_panel.base_url_ready {
        reason_codes.push(ReasonCode::MissingAuth);
        blocked_actions.push(item(
            "blocked-kis-market-data-activate",
            NextActionKind::RunKISMarketDataActivate,
            NextActionPriority::Required,
            Some("cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_fixture_replay.toml".to_string()),
            Some("target/sprint51/kis_market_data_activate_fixture_replay/".to_string()),
            false,
            vec![ReasonCode::MissingAuth],
        ));
    }

    if kis_monitor_panel.official_row_count == 0 || kis_monitor_panel.canonical_csv_count == 0 {
        recommended_actions.push(item(
            "kis-market-data-activate",
            NextActionKind::RunKISMarketDataActivate,
            NextActionPriority::Required,
            Some("cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_local_import.toml".to_string()),
            Some("target/sprint51/kis_market_data_activate_local_import/".to_string()),
            true,
            vec![ReasonCode::EvidenceStillInsufficient],
        ));
    } else if kis_monitor_panel
        .collection_plan_status
        .to_ascii_lowercase()
        .contains("dry")
    {
        recommended_actions.push(item(
            "kis-dry-run",
            NextActionKind::RunKISDryRun,
            NextActionPriority::Recommended,
            Some("cargo run --quiet --bin soma_experiment -- kis-market-data-activate --config examples/soma_kis_market_data_activate_fixture_replay.toml".to_string()),
            Some("target/sprint51/kis_market_data_activate_fixture_replay/".to_string()),
            true,
            vec![ReasonCode::ProviderRequestPlanned],
        ));
    }

    if kis_monitor_panel
        .candle_sufficiency_status
        .contains("MissingFuture")
        || kis_monitor_panel
            .candle_sufficiency_status
            .contains("Insufficient")
    {
        recommended_actions.push(item(
            "kis-candle-sufficiency",
            NextActionKind::RunKISCandleSufficiency,
            NextActionPriority::Required,
            Some("cargo run --quiet --bin soma_experiment -- kis-candle-sufficiency --config examples/soma_kis_candle_sufficiency.toml".to_string()),
            Some("target/sprint51/kis_candle_sufficiency/".to_string()),
            true,
            vec![ReasonCode::InsufficientBars],
        ));
    }

    if kis_monitor_panel.outcome_links == 0 || kis_monitor_panel.complete_rows == 0 {
        recommended_actions.push(item(
            "kis-outcome-link-close",
            NextActionKind::RunKISOutcomeLinkClose,
            NextActionPriority::Required,
            Some("cargo run --quiet --bin soma_experiment -- kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml".to_string()),
            Some("target/sprint51/kis_outcome_link_close/".to_string()),
            true,
            vec![ReasonCode::EvidenceStillInsufficient],
        ));
        recommended_actions.push(item(
            "improve-outcome-link-depth",
            NextActionKind::ImproveOutcomeLinkDepth,
            NextActionPriority::Recommended,
            Some("cargo run --quiet --bin soma_experiment -- kis-outcome-link-close --config examples/soma_kis_outcome_link_close.toml".to_string()),
            Some("target/sprint51/kis_outcome_link_close/".to_string()),
            true,
            vec![ReasonCode::EvidenceStillInsufficient],
        ));
    }

    if !owner_panel.pending_review_items.is_empty() {
        recommended_actions.push(item(
            "owner-review-queue",
            NextActionKind::RunOwnerReviewQueue,
            NextActionPriority::Required,
            Some("cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml".to_string()),
            Some("owner review queue text output".to_string()),
            true,
            vec![ReasonCode::OwnerReviewQueueBuilt],
        ));
        recommended_actions.push(item(
            "review-human-confirm-candidate",
            NextActionKind::ReviewHumanConfirmCandidate,
            NextActionPriority::Recommended,
            Some("cargo run --quiet --bin soma_experiment -- owner-impact-report --config examples/soma_owner_impact_report.toml".to_string()),
            Some("owner impact report text output".to_string()),
            true,
            vec![ReasonCode::OwnerImpactReportBuilt],
        ));
    }

    if candidate_panel.blocked_candidates > 0 || risk_panel.denied_count > 0 {
        recommended_actions.push(item(
            "review-risk-blocked-candidate",
            NextActionKind::ReviewRiskBlockedCandidate,
            NextActionPriority::Recommended,
            Some("cargo run --quiet --bin soma_experiment -- owner-review-queue --config examples/soma_owner_review_queue.toml".to_string()),
            Some("owner review queue text output".to_string()),
            true,
            vec![ReasonCode::RiskDenied],
        ));
    }

    if refresh_planner.missing_outputs > 0 {
        let config_path = config_path.unwrap_or("examples/soma_control_tower_v1_kis.toml");
        recommended_actions.push(item(
            "refresh-control-tower-v1",
            NextActionKind::RunDashboardRender,
            NextActionPriority::Required,
            Some(format!(
                "cargo run --quiet --bin soma_experiment -- control-tower-v1 --config {config_path}"
            )),
            Some(format!(
                "{output_root}/<control_tower_id>/dashboard_v1.html"
            )),
            true,
            vec![ReasonCode::DashboardReportMissing],
        ));
    }

    operator_actions.push(item(
        "dashboard-action-drafts",
        NextActionKind::RunDashboardSnapshot,
        NextActionPriority::Optional,
        Some(format!(
            "cargo run --quiet --bin soma_experiment -- dashboard-action-drafts --config {}",
            config_path.unwrap_or("examples/soma_dashboard_action_drafts.toml")
        )),
        Some(format!(
            "{output_root}/<control_tower_id>/owner_action_drafts/"
        )),
        true,
        vec![ReasonCode::OwnerInputApplied],
    ));
    operator_actions.push(item(
        "owner-impact-report",
        NextActionKind::RunOwnerImpactReport,
        NextActionPriority::Optional,
        Some("cargo run --quiet --bin soma_experiment -- owner-impact-report --config examples/soma_owner_impact_report.toml".to_string()),
        Some("owner impact report text output".to_string()),
        true,
        vec![ReasonCode::OwnerImpactReportBuilt],
    ));
    operator_actions.push(item(
        "core-performance",
        NextActionKind::RunCorePerformance,
        NextActionPriority::Optional,
        Some("cargo run --quiet --bin soma_experiment -- core-performance --config examples/soma_core_performance_diagnostics_only.toml".to_string()),
        Some("target/core_performance_diagnostics_only/".to_string()),
        true,
        vec![ReasonCode::CorePerformanceScorecardBuilt],
    ));
    operator_actions.push(item(
        "committee-benchmark",
        NextActionKind::RunCommitteeBenchmark,
        NextActionPriority::Optional,
        Some("cargo run --quiet --bin soma_experiment -- committee-benchmark --config examples/soma_committee_benchmark_fixture.toml".to_string()),
        Some("committee benchmark text output".to_string()),
        true,
        vec![ReasonCode::CommitteeValueAttributionReportBuilt],
    ));

    let primary_next_action = recommended_actions
        .iter()
        .find(|item| matches!(item.priority, NextActionPriority::Required))
        .cloned()
        .or_else(|| recommended_actions.first().cloned())
        .unwrap_or_else(|| {
            item(
                "no-action",
                NextActionKind::NoAction,
                NextActionPriority::Optional,
                config_path.map(|path| {
                    format!(
                        "cargo run --quiet --bin soma_experiment -- control-tower-v1 --config {path}"
                    )
                }),
                Some(format!("{output_root}/<control_tower_id>/dashboard_v1.html")),
                true,
                vec![ReasonCode::DeterministicPath],
            )
        });

    let mut panel = NextActionPanel {
        primary_next_action,
        recommended_actions,
        blocked_actions,
        operator_actions,
        reason_codes,
    };
    panel.stabilize();
    panel
}

fn item(
    action_id: &str,
    action_kind: NextActionKind,
    priority: NextActionPriority,
    command_suggestion: Option<String>,
    expected_output_artifact: Option<String>,
    safe_to_run: bool,
    reason_codes: Vec<ReasonCode>,
) -> NextActionItem {
    NextActionItem {
        action_id: action_id.to_string(),
        action_kind,
        priority,
        command_suggestion,
        expected_output_artifact,
        safe_to_run,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}
