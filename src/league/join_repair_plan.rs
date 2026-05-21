use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::official_candle_join_audit::OfficialCandleJoinAuditConfig;
use super::row_candle_candidate::{
    RowCandleCandidateReport, RowCandleCandidateStatus, buckets_by_row_id,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JoinRepairActionKind {
    AddSymbolAlias,
    AddTimeframeAlias,
    AddTimestampPolicy,
    EnableSessionDailyAlignment,
    IncreaseTimestampTolerance,
    ProvidePreflightReport,
    ProvideProvenance,
    ProvideLongerCandleWindow,
    ProvideMatchingCanonicalCsv,
    RegenerateScenarioRows,
    RerunCandlePack,
    RerunComparableBackfill,
    RerunReferenceGeneration,
    RerunCorePerformance,
    #[default]
    NoSafeRepairAvailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JoinRepairAction {
    pub action_id: String,
    pub action_kind: JoinRepairActionKind,
    #[serde(default)]
    pub row_id: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    #[serde(default)]
    pub required_artifact: Option<String>,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    pub safe_to_apply_automatically: bool,
    pub requires_operator_review: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JoinRepairPlanStatus {
    RepairAvailable,
    OperatorActionRequired,
    #[default]
    NoSafeRepairAvailable,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JoinRepairPlan {
    pub plan_id: String,
    pub actions: Vec<JoinRepairAction>,
    pub auto_safe_actions: usize,
    pub operator_review_actions: usize,
    pub no_safe_repair_count: usize,
    pub plan_status: JoinRepairPlanStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_join_repair_plan(
    config: &OfficialCandleJoinAuditConfig,
    candidate_report: &RowCandleCandidateReport,
) -> JoinRepairPlan {
    let buckets = buckets_by_row_id(candidate_report);
    let mut actions = buckets
        .values()
        .flat_map(|bucket| build_actions_for_bucket(config, bucket))
        .collect::<Vec<_>>();
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    let auto_safe_actions = actions
        .iter()
        .filter(|action| action.safe_to_apply_automatically)
        .count();
    let operator_review_actions = actions
        .iter()
        .filter(|action| action.requires_operator_review)
        .count();
    let no_safe_repair_count = actions
        .iter()
        .filter(|action| action.action_kind == JoinRepairActionKind::NoSafeRepairAvailable)
        .count();
    let plan_status = if candidate_report.candidates_by_row.iter().all(|bucket| {
        matches!(
            bucket.status,
            RowCandleCandidateStatus::DiagnosticOnly | RowCandleCandidateStatus::SourceIneligible
        )
    }) {
        JoinRepairPlanStatus::DiagnosticOnly
    } else if auto_safe_actions > 0 {
        JoinRepairPlanStatus::RepairAvailable
    } else if operator_review_actions > 0 {
        JoinRepairPlanStatus::OperatorActionRequired
    } else {
        JoinRepairPlanStatus::NoSafeRepairAvailable
    };
    JoinRepairPlan {
        plan_id: format!("{}-repair-plan", config.audit_id),
        actions,
        auto_safe_actions,
        operator_review_actions,
        no_safe_repair_count,
        plan_status,
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialCandleCoverageBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

impl JoinRepairPlan {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("auto_safe_actions={}", self.auto_safe_actions),
            format!("operator_review_actions={}", self.operator_review_actions),
            format!("no_safe_repair_count={}", self.no_safe_repair_count),
            format!("plan_status={:?}", self.plan_status),
        ];
        lines.extend(self.actions.iter().map(|action| {
            format!(
                "action_id={};action_kind={:?};row_id={};symbol={};timeframe={};required_artifact={};safe_to_apply_automatically={};requires_operator_review={};command_suggestion={}",
                action.action_id,
                action.action_kind,
                action.row_id.clone().unwrap_or_default(),
                action.symbol.clone().unwrap_or_default(),
                action.timeframe.clone().unwrap_or_default(),
                action.required_artifact.clone().unwrap_or_default(),
                action.safe_to_apply_automatically,
                action.requires_operator_review,
                action.command_suggestion.clone().unwrap_or_default(),
            )
        }));
        lines.join("\n")
    }
}

fn build_actions_for_bucket(
    config: &OfficialCandleJoinAuditConfig,
    bucket: &super::row_candle_candidate::RowCandleCandidateBucket,
) -> Vec<JoinRepairAction> {
    let action = match bucket.status {
        RowCandleCandidateStatus::SymbolMismatch => build_action(
            config,
            bucket,
            JoinRepairActionKind::AddSymbolAlias,
            config.symbol_alias_map_path.clone(),
            !config.allow_explicit_symbol_alias && config.symbol_alias_map_path.is_some(),
            "candle-join-repair-plan --config",
        ),
        RowCandleCandidateStatus::TimeframeMismatch => build_action(
            config,
            bucket,
            JoinRepairActionKind::AddTimeframeAlias,
            config.timeframe_alias_map_path.clone(),
            !config.allow_explicit_timeframe_alias && config.timeframe_alias_map_path.is_some(),
            "candle-join-repair-plan --config",
        ),
        RowCandleCandidateStatus::TimestampOutsideRange => build_action(
            config,
            bucket,
            JoinRepairActionKind::AddTimestampPolicy,
            config.timestamp_policy_map_path.clone(),
            !config.allow_explicit_timestamp_policy_map
                && config.timestamp_policy_map_path.is_some(),
            "candle-join-repair-plan --config",
        ),
        RowCandleCandidateStatus::MissingPreflight => build_action(
            config,
            bucket,
            JoinRepairActionKind::ProvidePreflightReport,
            None,
            false,
            "provide local preflight report and rerun candle-join-audit",
        ),
        RowCandleCandidateStatus::MissingProvenance => build_action(
            config,
            bucket,
            JoinRepairActionKind::ProvideProvenance,
            None,
            false,
            "provide local provenance sidecar and rerun candle-join-audit",
        ),
        RowCandleCandidateStatus::MissingFutureWindow => build_action(
            config,
            bucket,
            JoinRepairActionKind::ProvideLongerCandleWindow,
            bucket.selected_candle_series_id.clone(),
            false,
            "provide longer local candle coverage window and rerun candle-join-audit",
        ),
        RowCandleCandidateStatus::NoCandidate => build_action(
            config,
            bucket,
            JoinRepairActionKind::ProvideMatchingCanonicalCsv,
            None,
            false,
            "provide matching canonical csv or regenerate scenario rows",
        ),
        RowCandleCandidateStatus::SourceIneligible | RowCandleCandidateStatus::DiagnosticOnly => {
            build_action(
                config,
                bucket,
                JoinRepairActionKind::NoSafeRepairAvailable,
                None,
                false,
                "source class is intentionally diagnostic-only; do not promote it",
            )
        }
        _ => build_action(
            config,
            bucket,
            JoinRepairActionKind::RerunComparableBackfill,
            None,
            true,
            "comparable-backfill --config <local-config>",
        ),
    };
    vec![action]
}

fn build_action(
    config: &OfficialCandleJoinAuditConfig,
    bucket: &super::row_candle_candidate::RowCandleCandidateBucket,
    action_kind: JoinRepairActionKind,
    required_artifact: Option<String>,
    safe_to_apply_automatically: bool,
    command_suggestion: &str,
) -> JoinRepairAction {
    let requires_operator_review = !safe_to_apply_automatically
        || matches!(
            action_kind,
            JoinRepairActionKind::ProvidePreflightReport
                | JoinRepairActionKind::ProvideProvenance
                | JoinRepairActionKind::ProvideLongerCandleWindow
                | JoinRepairActionKind::ProvideMatchingCanonicalCsv
                | JoinRepairActionKind::RegenerateScenarioRows
                | JoinRepairActionKind::NoSafeRepairAvailable
        );
    JoinRepairAction {
        action_id: format!(
            "{}-{:?}-{}",
            config.audit_id,
            action_kind,
            bucket.row_id.to_ascii_lowercase()
        ),
        action_kind,
        row_id: Some(bucket.row_id.clone()),
        symbol: Some(bucket.normalized_key.raw_symbol.clone()),
        timeframe: Some(bucket.normalized_key.timeframe.clone()),
        required_artifact,
        command_suggestion: Some(command_suggestion.to_string()),
        safe_to_apply_automatically,
        requires_operator_review,
        reason_codes: stable_reason_codes(&bucket.reason_codes),
    }
}
