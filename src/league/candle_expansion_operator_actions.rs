use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::ProviderKind;

use super::candle_acquisition_job::{
    CandleAcquisitionJob, CandleAcquisitionJobKind, CandleAcquisitionPlan,
};
use super::official_candle_gap_map::{OfficialCandleCoverageGapMap, OfficialCandleGapKind};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandleExpansionActionPriority {
    Required,
    Recommended,
    #[default]
    Optional,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandleExpansionOperatorAction {
    pub action_id: String,
    pub priority: CandleExpansionActionPriority,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    pub description: String,
    #[serde(default)]
    pub env_var_names: Vec<String>,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_output_artifact: Option<String>,
    pub safe_to_run: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CandleExpansionOperatorAction {
    pub fn sort_key(
        &self,
    ) -> (
        CandleExpansionActionPriority,
        String,
        Option<ProviderKind>,
        Option<String>,
        Option<String>,
    ) {
        (
            self.priority,
            self.action_id.clone(),
            self.provider_kind,
            self.symbol.clone(),
            self.timeframe.clone(),
        )
    }

    pub fn to_text(&self) -> String {
        format!(
            "action_id={};priority={:?};provider_kind={};market={};symbol={};timeframe={};env_var_names={};safe_to_run={};command_suggestion={};expected_output_artifact={};description={}",
            self.action_id,
            self.priority,
            self.provider_kind
                .map(|provider| format!("{provider:?}"))
                .unwrap_or_default(),
            self.market.clone().unwrap_or_default(),
            self.symbol.clone().unwrap_or_default(),
            self.timeframe.clone().unwrap_or_default(),
            self.env_var_names.join("|"),
            self.safe_to_run,
            self.command_suggestion.clone().unwrap_or_default(),
            self.expected_output_artifact.clone().unwrap_or_default(),
            self.description,
        )
    }
}

pub fn build_candle_expansion_operator_actions(
    gap_map: &OfficialCandleCoverageGapMap,
    jobs: &[CandleAcquisitionJob],
) -> Vec<CandleExpansionOperatorAction> {
    let mut actions = Vec::new();

    if gap_map.cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingOfficialCandleSeries)
            || cell
                .gap_kinds
                .contains(&OfficialCandleGapKind::MissingCandleSeries)
    }) {
        actions.push(action(
            "provide-official-canonical-csv",
            CandleExpansionActionPriority::Required,
            None,
            None,
            None,
            None,
            "Provide a local official canonical CSV for the missing bounded candle gap.",
            vec![],
            None,
            Some("canonical csv".to_string()),
            false,
            vec![ReasonCode::MissingOfficialData],
        ));
    }
    if gap_map.cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingProvenance)
    }) {
        actions.push(action(
            "provide-official-provenance",
            CandleExpansionActionPriority::Required,
            None,
            None,
            None,
            None,
            "Provide official provenance metadata locally before claiming official readiness.",
            vec![],
            None,
            Some("provenance json".to_string()),
            false,
            vec![ReasonCode::MissingOfficialProvenance],
        ));
    }
    if gap_map.cells.iter().any(|cell| {
        cell.gap_kinds
            .contains(&OfficialCandleGapKind::MissingPreflight)
    }) {
        actions.push(action(
            "run-data-preflight",
            CandleExpansionActionPriority::Required,
            None,
            None,
            None,
            None,
            "Run local data preflight or provide a local preflight report; research-only and local-only.",
            vec![],
            Some(
                "cargo run --bin soma_experiment -- data-preflight --input <local.csv> --out <local-dir> --symbol <symbol> --timeframe <timeframe>".to_string(),
            ),
            Some("preflight json".to_string()),
            true,
            vec![ReasonCode::MissingOfficialPreflight, ReasonCode::PreflightFailed],
        ));
    }
    if jobs
        .iter()
        .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedMissingApproval)
    {
        actions.push(action(
            "wait-for-krx-approval",
            CandleExpansionActionPriority::Required,
            Some(ProviderKind::KrxOpenApi),
            Some("KoreanEquity".to_string()),
            None,
            None,
            "Wait for KRX approval before bounded official Korean equity collection.",
            vec![],
            None,
            None,
            false,
            vec![ReasonCode::MissingApproval, ReasonCode::KrxApprovalPending],
        ));
    }
    if jobs.iter().any(|job| {
        job.job_kind == CandleAcquisitionJobKind::SkippedMissingAuth
            && job.provider_kind == Some(ProviderKind::KrxOpenApi)
    }) {
        actions.push(action(
            "set-krx-api-key",
            CandleExpansionActionPriority::Required,
            Some(ProviderKind::KrxOpenApi),
            Some("KoreanEquity".to_string()),
            None,
            None,
            "Set KRX_API_KEY locally; never print or persist the secret value.",
            vec!["KRX_API_KEY".to_string()],
            Some(
                "cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml".to_string(),
            ),
            None,
            true,
            vec![ReasonCode::MissingAuth],
        ));
    }
    if jobs
        .iter()
        .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedMissingEndpointTemplate)
    {
        actions.push(action(
            "set-krx-endpoint-template",
            CandleExpansionActionPriority::Required,
            Some(ProviderKind::KrxOpenApi),
            Some("KoreanEquity".to_string()),
            None,
            None,
            "Set KRX_ENDPOINT_TEMPLATE locally for bounded market-data-only collection.",
            vec!["KRX_ENDPOINT_TEMPLATE".to_string()],
            Some(
                "cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml".to_string(),
            ),
            None,
            true,
            vec![ReasonCode::MissingEndpointTemplate],
        ));
    }
    if jobs.iter().any(|job| {
        job.job_kind == CandleAcquisitionJobKind::SkippedMissingAuth
            && job.provider_kind == Some(ProviderKind::DataGoKrFscStockPrice)
    }) {
        actions.push(action(
            "set-data-go-kr-service-key",
            CandleExpansionActionPriority::Required,
            Some(ProviderKind::DataGoKrFscStockPrice),
            Some("KoreanEquity".to_string()),
            None,
            None,
            "Set DATA_GO_KR_SERVICE_KEY locally for bounded market-data-only collection.",
            vec!["DATA_GO_KR_SERVICE_KEY".to_string()],
            None,
            None,
            true,
            vec![ReasonCode::MissingAuth],
        ));
    }
    if jobs.iter().any(|job| {
        job.job_kind == CandleAcquisitionJobKind::SkippedMissingAuth
            && job.provider_kind == Some(ProviderKind::AlphaVantage)
    }) {
        actions.push(action(
            "set-alphavantage-api-key",
            CandleExpansionActionPriority::Required,
            Some(ProviderKind::AlphaVantage),
            Some("USEquity".to_string()),
            None,
            None,
            "Set ALPHAVANTAGE_API_KEY locally; AlphaVantage remains compact EOD research-only here.",
            vec!["ALPHAVANTAGE_API_KEY".to_string()],
            None,
            None,
            true,
            vec![ReasonCode::MissingAuth],
        ));
    }
    if jobs.iter().any(|job| {
        job.job_kind == CandleAcquisitionJobKind::SkippedMissingAuth
            && job.provider_kind == Some(ProviderKind::Alpaca)
    }) {
        actions.push(action(
            "set-alpaca-keys",
            CandleExpansionActionPriority::Required,
            Some(ProviderKind::Alpaca),
            Some("USEquity".to_string()),
            None,
            None,
            "Set ALPACA_API_KEY and ALPACA_SECRET_KEY locally for bounded historical bars collection only.",
            vec!["ALPACA_API_KEY".to_string(), "ALPACA_SECRET_KEY".to_string()],
            None,
            None,
            true,
            vec![ReasonCode::MissingAuth],
        ));
    }
    if jobs
        .iter()
        .any(|job| job.is_collection_job() && job.is_runnable())
    {
        actions.push(action(
            "run-official-acquire",
            CandleExpansionActionPriority::Recommended,
            None,
            None,
            None,
            None,
            "Run bounded official acquisition or collection only after reviewing auth, approval, and scope.",
            vec![],
            Some(
                "cargo run --bin soma_experiment -- candle-expand --config <local-plan.toml>".to_string(),
            ),
            Some("bounded collection outputs".to_string()),
            true,
            vec![ReasonCode::OfficialEvidenceAcquisitionRan],
        ));
        actions.push(action(
            "run-collect-candles",
            CandleExpansionActionPriority::Optional,
            None,
            None,
            None,
            None,
            "If explicitly enabled, keep collection bounded, market-data-only, and research-only.",
            vec![],
            Some(
                "cargo run --bin soma_experiment -- collect-candles --provider <provider> --symbol <symbol> --timeframe <timeframe> --out <local-dir>".to_string(),
            ),
            Some("canonical csv".to_string()),
            true,
            vec![ReasonCode::OfficialEvidenceAcquisitionRan],
        ));
    }
    if jobs
        .iter()
        .any(|job| job.is_import_job() && job.is_runnable())
    {
        actions.push(action(
            "run-candle-pack",
            CandleExpansionActionPriority::Recommended,
            None,
            None,
            None,
            None,
            "Rebuild the official candle pack from local imported or reused canonical candles.",
            vec![],
            Some("cargo run --bin soma_experiment -- candle-pack --config <pack.toml>".to_string()),
            Some("official_candle_coverage_pack.json".to_string()),
            true,
            vec![ReasonCode::OfficialCandleCoverageBuilt],
        ));
    }
    if !gap_map.cells.is_empty() {
        actions.push(action(
            "run-candle-coverage-close",
            CandleExpansionActionPriority::Recommended,
            None,
            None,
            None,
            None,
            "Rerun candle coverage closure after bounded import or collection to re-evaluate the bottleneck.",
            vec![],
            Some(
                "cargo run --bin soma_experiment -- candle-coverage-close --config <closure.toml>".to_string(),
            ),
            Some("candle_expansion_closure.txt".to_string()),
            true,
            vec![ReasonCode::OfficialCandleCoverageBuilt],
        ));
        actions.push(action(
            "run-comparable-backfill",
            CandleExpansionActionPriority::Recommended,
            None,
            None,
            None,
            None,
            "Rerun comparable evidence backfill after candle coverage expands; never fabricate outcomes.",
            vec![],
            Some(
                "cargo run --bin soma_experiment -- comparable-backfill --config <backfill.toml>".to_string(),
            ),
            Some("comparable_backfill_report.json".to_string()),
            true,
            vec![ReasonCode::OfficialCandleCoverageBuilt],
        ));
    }
    if jobs
        .iter()
        .any(|job| job.job_kind == CandleAcquisitionJobKind::SkippedBudgetExceeded)
    {
        actions.push(action(
            "reduce-scope",
            CandleExpansionActionPriority::Required,
            None,
            None,
            None,
            None,
            "Reduce symbol, timeframe, or row scope to stay within the bounded research-only storage budget.",
            vec![],
            None,
            None,
            true,
            vec![ReasonCode::BudgetExceeded, ReasonCode::CollectionBudgetExceeded],
        ));
    }

    actions.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
    actions.dedup_by(|left, right| {
        left.action_id == right.action_id && left.provider_kind == right.provider_kind
    });
    actions
}

pub fn rebuild_plan_actions(
    gap_map: &OfficialCandleCoverageGapMap,
    plan: &CandleAcquisitionPlan,
) -> CandleAcquisitionPlan {
    let mut cloned = plan.clone();
    cloned.operator_actions = build_candle_expansion_operator_actions(gap_map, &cloned.jobs);
    cloned
}

fn action(
    action_id: &str,
    priority: CandleExpansionActionPriority,
    provider_kind: Option<ProviderKind>,
    market: Option<String>,
    symbol: Option<String>,
    timeframe: Option<String>,
    description: &str,
    env_var_names: Vec<String>,
    command_suggestion: Option<String>,
    expected_output_artifact: Option<String>,
    safe_to_run: bool,
    reason_codes: Vec<ReasonCode>,
) -> CandleExpansionOperatorAction {
    CandleExpansionOperatorAction {
        action_id: action_id.to_string(),
        priority,
        provider_kind,
        market,
        symbol,
        timeframe,
        description: description.to_string(),
        env_var_names,
        command_suggestion,
        expected_output_artifact,
        safe_to_run,
        reason_codes: stable_reason_codes(
            &reason_codes
                .into_iter()
                .chain([ReasonCode::OperatorActionPlanBuilt])
                .collect::<Vec<_>>(),
        ),
    }
}
