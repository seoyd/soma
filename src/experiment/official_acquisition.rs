use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::backtest::Timeframe;
use crate::core::ReasonCode;
use crate::data::{
    AssetClass, AuthConfig, CollectionOutputSize, CollectionSizePolicy, CompressionPolicy,
    MarketVenue, OfficialCollectionEntry, OfficialCollectionPlan, OfficialCollectionReport,
    OfficialCollectionRunner, ProviderAuthPreflightConfig, ProviderAuthPreflightReport,
    ProviderAuthPreflightRunner, ProviderAuthStatusKind, ProviderKind, RawArchivePolicy,
    RetentionPolicy, StorageBudget,
};
use crate::experiment::{
    OfficialEvidenceExpansionConfig, OfficialEvidenceExpansionRecommendation,
    OfficialEvidenceExpansionReport, OfficialEvidenceExpansionRunner,
    OfficialEvidenceExpansionStatus, OperatorActionPlan, PreviousCollectionComparison,
    build_operator_action_plan, build_previous_collection_comparison,
    load_previous_collection_report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAcquisitionStorageCheck {
    pub requested_max_symbols: usize,
    pub requested_max_rows: usize,
    pub requested_max_requests: usize,
    pub requested_max_bytes: usize,
    pub estimated_bytes: usize,
    pub budget_ok: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceAcquisitionPlan {
    pub plan_id: String,
    pub auth_preflight_config: ProviderAuthPreflightConfig,
    #[serde(default)]
    pub official_collection_plan_path: Option<String>,
    #[serde(default)]
    pub fallback_crypto_only_plan: Option<String>,
    #[serde(default)]
    pub previous_collection_report_path: Option<String>,
    #[serde(default)]
    pub expansion_config: Option<OfficialEvidenceExpansionConfig>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_collection: bool,
    #[serde(default = "default_true")]
    pub run_upbit_if_public_available: bool,
    #[serde(default = "default_true")]
    pub run_krx_if_auth_ready: bool,
    #[serde(default = "default_true")]
    pub run_alpha_if_auth_ready: bool,
    #[serde(default = "default_true")]
    pub skip_missing_auth: bool,
    #[serde(default = "default_three")]
    pub max_symbols: usize,
    #[serde(default = "default_five_hundred")]
    pub max_rows_per_symbol: usize,
    #[serde(default = "default_ten")]
    pub max_requests: usize,
    #[serde(default = "default_storage")]
    pub max_total_bytes: usize,
    #[serde(default)]
    pub allow_full_history: bool,
    #[serde(default)]
    pub allow_all_symbols: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialEvidenceAcquisitionRecommendation {
    SetAlphaVantageAuth,
    SetKrxAuth,
    SetKrxEndpointTemplate,
    RunCryptoOnlyEvidence,
    RunMultiVenueEvidence,
    MoreOfficialEvidence,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    HoldCurrentScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceAcquisitionReport {
    pub plan_id: String,
    pub auth_preflight_report: ProviderAuthPreflightReport,
    #[serde(default)]
    pub generated_collection_plan: Option<OfficialCollectionPlan>,
    pub storage_check: EvidenceAcquisitionStorageCheck,
    #[serde(default)]
    pub collection_report: Option<OfficialCollectionReport>,
    #[serde(default)]
    pub previous_collection_comparison: Option<PreviousCollectionComparison>,
    #[serde(default)]
    pub expansion_report: Option<OfficialEvidenceExpansionReport>,
    pub operator_action_plan: OperatorActionPlan,
    pub final_status: OfficialEvidenceExpansionStatus,
    pub final_recommendation: OfficialEvidenceAcquisitionRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceAcquisitionRunner;

impl Default for OfficialEvidenceAcquisitionPlan {
    fn default() -> Self {
        Self {
            plan_id: "official_evidence_acquisition".to_string(),
            auth_preflight_config: ProviderAuthPreflightConfig::default(),
            official_collection_plan_path: None,
            fallback_crypto_only_plan: None,
            previous_collection_report_path: None,
            expansion_config: Some(OfficialEvidenceExpansionConfig {
                run_collection: false,
                ..OfficialEvidenceExpansionConfig::default()
            }),
            output_root: default_output_root(),
            run_collection: true,
            run_upbit_if_public_available: true,
            run_krx_if_auth_ready: true,
            run_alpha_if_auth_ready: true,
            skip_missing_auth: true,
            max_symbols: default_three(),
            max_rows_per_symbol: default_five_hundred(),
            max_requests: default_ten(),
            max_total_bytes: default_storage(),
            allow_full_history: false,
            allow_all_symbols: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialEvidenceAcquisitionPlan {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            self.official_collection_plan_path.as_deref(),
            self.fallback_crypto_only_plan.as_deref(),
            self.previous_collection_report_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reasons.push(ReasonCode::RemotePathRejected);
            }
        }
        if self.expansion_config.as_ref().is_some_and(|config| {
            config
                .validate_local_paths()
                .contains(&ReasonCode::RemotePathRejected)
        }) {
            reasons.push(ReasonCode::RemotePathRejected);
        }
        dedupe_reasons(reasons)
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.plan_id)
    }
}

impl OfficialEvidenceAcquisitionReport {
    pub fn to_text(&self) -> String {
        [
            format!("plan_id={}", self.plan_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            self.auth_preflight_report.to_text(),
            self.storage_check.to_text(),
            self.previous_collection_comparison
                .as_ref()
                .map(|report| report.to_text())
                .unwrap_or_default(),
            self.operator_action_plan.to_text(),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn to_markdown(&self) -> String {
        [
            format!("# {}", self.plan_id),
            format!("- final_status: `{:?}`", self.final_status),
            format!("- final_recommendation: `{:?}`", self.final_recommendation),
            format!("- blockers: {}", self.blockers.join(", ")),
            format!("- warnings: {}", self.warnings.join(", ")),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_acquisition_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_acquisition_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_acquisition_report.md"),
            self.to_markdown(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl EvidenceAcquisitionStorageCheck {
    pub fn to_text(&self) -> String {
        [
            format!("requested_max_symbols={}", self.requested_max_symbols),
            format!("requested_max_rows={}", self.requested_max_rows),
            format!("requested_max_requests={}", self.requested_max_requests),
            format!("requested_max_bytes={}", self.requested_max_bytes),
            format!("estimated_bytes={}", self.estimated_bytes),
            format!("budget_ok={}", self.budget_ok),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }
}

impl OfficialEvidenceAcquisitionRunner {
    pub fn run(
        &self,
        plan: &OfficialEvidenceAcquisitionPlan,
    ) -> Result<OfficialEvidenceAcquisitionReport, String> {
        if plan
            .validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
        {
            return Err("official-acquire config path must be local".to_string());
        }

        let auth_preflight_report =
            ProviderAuthPreflightRunner::default().run(&plan.auth_preflight_config);
        let storage_check = build_evidence_acquisition_storage_check(plan);
        let ready_providers = ready_providers(plan, &auth_preflight_report);
        let generated_collection_plan = if storage_check.budget_ok {
            build_generated_collection_plan(plan, &ready_providers)?
        } else {
            None
        };
        let collection_report = if plan.run_collection && storage_check.budget_ok {
            generated_collection_plan
                .as_ref()
                .map(|generated| OfficialCollectionRunner::default().run_plan(generated))
        } else {
            None
        };
        let (previous_collection_report, load_reason_codes) =
            load_previous_collection_report(plan.previous_collection_report_path.as_deref())?;
        let previous_collection_comparison =
            if plan.previous_collection_report_path.is_some() || collection_report.is_some() {
                Some(build_previous_collection_comparison(
                    previous_collection_report.as_ref(),
                    collection_report.as_ref(),
                    Some(&auth_preflight_report),
                    plan.previous_collection_report_path.is_some(),
                    &load_reason_codes,
                ))
            } else {
                None
            };
        let expansion_report = run_expansion_if_ready(plan, collection_report.as_ref())?;
        let crypto_ready = ready_providers.contains(&ProviderKind::Upbit);
        let multi_venue_ready = ready_providers.contains(&ProviderKind::KrxOpenApi)
            || ready_providers.contains(&ProviderKind::AlphaVantage);
        let operator_action_plan = build_operator_action_plan(
            &auth_preflight_report,
            expansion_report.as_ref(),
            generated_collection_plan.is_some(),
            crypto_ready,
            multi_venue_ready,
        );
        let (final_status, final_recommendation, blockers, mut warnings, mut reason_codes) =
            map_acquisition_outcome(
                &auth_preflight_report,
                &storage_check,
                &ready_providers,
                collection_report.as_ref(),
                expansion_report.as_ref(),
            );
        warnings.extend(storage_check.warnings.clone());
        warnings.extend(operator_action_plan.warnings.clone());
        reason_codes.extend(load_reason_codes);
        reason_codes.push(ReasonCode::OfficialEvidenceAcquisitionRan);

        let report = OfficialEvidenceAcquisitionReport {
            plan_id: plan.plan_id.clone(),
            auth_preflight_report,
            generated_collection_plan,
            storage_check,
            collection_report,
            previous_collection_comparison,
            expansion_report,
            operator_action_plan,
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: dedupe_reasons(reason_codes),
        };
        report.write_to_dir(&plan.output_dir())?;
        Ok(report)
    }
}

pub fn build_evidence_acquisition_storage_check(
    plan: &OfficialEvidenceAcquisitionPlan,
) -> EvidenceAcquisitionStorageCheck {
    let estimated_bytes = plan
        .max_symbols
        .saturating_mul(plan.max_rows_per_symbol)
        .saturating_mul(64);
    let mut warnings = Vec::new();
    let mut reason_codes = vec![ReasonCode::EvidenceAcquisitionStorageCheckBuilt];
    let mut budget_ok = true;

    if plan.allow_all_symbols {
        warnings.push("all-symbol collection remains denied".to_string());
        reason_codes.push(ReasonCode::DeniedByDefault);
        budget_ok = false;
    }
    if plan.allow_full_history {
        warnings.push("full-history collection remains denied".to_string());
        reason_codes.push(ReasonCode::FullHistoryDenied);
        budget_ok = false;
    }
    if plan.max_symbols > default_three() {
        warnings.push("requested symbol scope exceeds bounded Sprint 26 limits".to_string());
        reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        budget_ok = false;
    }
    if plan.max_rows_per_symbol > default_five_hundred() {
        warnings.push("requested rows per symbol exceed Sprint 26 limit".to_string());
        reason_codes.push(ReasonCode::RowLimitApplied);
        budget_ok = false;
    }
    if plan.max_requests > default_ten() {
        warnings.push("requested request count exceeds Sprint 26 limit".to_string());
        reason_codes.push(ReasonCode::CollectionBudgetExceeded);
        budget_ok = false;
    }
    if plan.max_total_bytes > default_storage() || estimated_bytes > plan.max_total_bytes {
        warnings.push("requested storage exceeds bounded budget".to_string());
        reason_codes.push(ReasonCode::BudgetExceeded);
        budget_ok = false;
    }

    EvidenceAcquisitionStorageCheck {
        requested_max_symbols: plan.max_symbols,
        requested_max_rows: plan.max_rows_per_symbol,
        requested_max_requests: plan.max_requests,
        requested_max_bytes: plan.max_total_bytes,
        estimated_bytes,
        budget_ok,
        warnings,
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn build_generated_collection_plan(
    plan: &OfficialEvidenceAcquisitionPlan,
    ready_providers: &[ProviderKind],
) -> Result<Option<OfficialCollectionPlan>, String> {
    if ready_providers.is_empty() {
        return Ok(None);
    }

    let only_crypto = ready_providers
        .iter()
        .all(|provider| matches!(provider, ProviderKind::Upbit | ProviderKind::Binance));
    let mut source_plan = if only_crypto {
        if let Some(path) = plan.fallback_crypto_only_plan.as_deref() {
            OfficialCollectionPlan::from_toml_path(Path::new(path))?
        } else if let Some(path) = plan.official_collection_plan_path.as_deref() {
            OfficialCollectionPlan::from_toml_path(Path::new(path))?
        } else {
            default_collection_plan()
        }
    } else if let Some(path) = plan.official_collection_plan_path.as_deref() {
        OfficialCollectionPlan::from_toml_path(Path::new(path))?
    } else {
        default_collection_plan()
    };

    source_plan
        .entries
        .retain(|entry| ready_providers.contains(&entry.provider_kind));
    source_plan.entries.truncate(plan.max_symbols);
    for entry in &mut source_plan.entries {
        entry.max_rows = Some(
            entry
                .max_rows
                .unwrap_or(plan.max_rows_per_symbol)
                .min(plan.max_rows_per_symbol),
        );
        entry.max_requests = Some(
            entry
                .max_requests
                .unwrap_or(plan.max_requests)
                .min(plan.max_requests),
        );
        entry.enabled = true;
    }
    if source_plan.entries.is_empty() {
        return Ok(None);
    }
    source_plan.plan_id = format!("{}-generated-collection", plan.plan_id);
    source_plan.output_root = plan.output_dir().join("collection").display().to_string();
    source_plan.max_total_rows = plan.max_symbols.saturating_mul(plan.max_rows_per_symbol);
    source_plan.max_total_requests = plan.max_requests;
    source_plan.max_total_bytes = plan.max_total_bytes;
    source_plan
        .default_collection_size_policy
        .max_symbols_per_run = plan.max_symbols;
    source_plan
        .default_collection_size_policy
        .max_rows_per_symbol = plan.max_rows_per_symbol;
    source_plan
        .default_collection_size_policy
        .max_total_rows_per_run = plan.max_symbols.saturating_mul(plan.max_rows_per_symbol);
    source_plan
        .default_collection_size_policy
        .max_requests_per_run = plan.max_requests;
    source_plan
        .default_collection_size_policy
        .allow_full_history = false;
    source_plan
        .default_collection_size_policy
        .default_outputsize = CollectionOutputSize::Compact;
    source_plan
        .default_collection_size_policy
        .raw_archive_policy = RawArchivePolicy::CompactJson;
    source_plan.default_retention_policy = RetentionPolicy::DeleteRawAfterCanonicalAndManifest;
    source_plan.default_compression_policy = CompressionPolicy::default();
    source_plan.storage_budget = StorageBudget {
        max_total_bytes: plan.max_total_bytes,
        max_raw_bytes: plan.max_total_bytes / 2,
        max_canonical_bytes: plan.max_total_bytes / 3,
        max_manifest_bytes: plan.max_total_bytes / 10,
        max_file_count: 64,
        reason_codes: vec![ReasonCode::DeterministicPath],
    };
    source_plan.continue_on_missing_auth = plan.skip_missing_auth;
    source_plan.continue_on_provider_failure = true;
    Ok(Some(source_plan))
}

fn run_expansion_if_ready(
    plan: &OfficialEvidenceAcquisitionPlan,
    collection_report: Option<&OfficialCollectionReport>,
) -> Result<Option<OfficialEvidenceExpansionReport>, String> {
    let Some(collection_report) = collection_report else {
        return Ok(None);
    };
    let Some(expansion_config) = plan.expansion_config.as_ref() else {
        return Ok(None);
    };
    let collection_dir = plan.output_dir().join("current_collection");
    let collection_report_path = collection_report.write_to_dir(&collection_dir)?;
    let mut config = expansion_config.clone();
    config.run_auth_preflight = false;
    config.run_collection = false;
    config.output_root = plan.output_dir().join("expansion").display().to_string();
    config.venue_coverage_plan.existing_collection_report_path =
        Some(collection_report_path.display().to_string());
    if config.previous_collection_report_path.is_none() {
        config.previous_collection_report_path = plan.previous_collection_report_path.clone();
    }
    OfficialEvidenceExpansionRunner::default()
        .run(&config)
        .map(Some)
        .map_err(|err| err.to_string())
}

fn ready_providers(
    plan: &OfficialEvidenceAcquisitionPlan,
    auth_preflight_report: &ProviderAuthPreflightReport,
) -> Vec<ProviderKind> {
    let mut providers = Vec::new();
    if plan.run_upbit_if_public_available {
        providers.push(ProviderKind::Upbit);
    }
    if plan.run_krx_if_auth_ready && provider_ready(auth_preflight_report, ProviderKind::KrxOpenApi)
    {
        providers.push(ProviderKind::KrxOpenApi);
    }
    if plan.run_alpha_if_auth_ready
        && provider_ready(auth_preflight_report, ProviderKind::AlphaVantage)
    {
        providers.push(ProviderKind::AlphaVantage);
    }
    providers
}

fn provider_ready(report: &ProviderAuthPreflightReport, provider_kind: ProviderKind) -> bool {
    report
        .statuses
        .iter()
        .find(|status| status.provider_kind == provider_kind)
        .is_some_and(|status| {
            matches!(
                status.status,
                ProviderAuthStatusKind::Ready | ProviderAuthStatusKind::NotRequired
            )
        })
}

fn map_acquisition_outcome(
    auth_preflight_report: &ProviderAuthPreflightReport,
    storage_check: &EvidenceAcquisitionStorageCheck,
    ready_providers: &[ProviderKind],
    collection_report: Option<&OfficialCollectionReport>,
    expansion_report: Option<&OfficialEvidenceExpansionReport>,
) -> (
    OfficialEvidenceExpansionStatus,
    OfficialEvidenceAcquisitionRecommendation,
    Vec<String>,
    Vec<String>,
    Vec<ReasonCode>,
) {
    let mut blockers = Vec::new();
    let warnings = Vec::new();
    let mut reason_codes = vec![ReasonCode::OfficialEvidenceAcquisitionRan];

    if !storage_check.budget_ok {
        blockers.push("bounded storage check blocked collection scope".to_string());
        reason_codes.push(ReasonCode::BudgetExceeded);
        return (
            OfficialEvidenceExpansionStatus::StorageBudgetBlocked,
            OfficialEvidenceAcquisitionRecommendation::HoldCurrentScope,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }

    if let Some(expansion_report) = expansion_report {
        return (
            expansion_report.final_status,
            map_expansion_recommendation(expansion_report.final_recommendation),
            expansion_report.blockers.clone(),
            expansion_report.warnings.clone(),
            dedupe_reasons(reason_codes),
        );
    }

    let missing_krx_endpoint = auth_preflight_report
        .missing_endpoint_providers
        .iter()
        .any(|provider| provider == "krx");
    let missing_krx_auth = auth_preflight_report
        .missing_auth_providers
        .iter()
        .any(|provider| provider == "krx");
    let missing_alpha_auth = auth_preflight_report
        .missing_auth_providers
        .iter()
        .any(|provider| provider == "alphavantage");
    let has_upbit = ready_providers.contains(&ProviderKind::Upbit);
    let has_equity = ready_providers.contains(&ProviderKind::KrxOpenApi)
        || ready_providers.contains(&ProviderKind::AlphaVantage);

    if collection_report.is_none()
        || collection_report.is_some_and(|report| report.ready_entries_count == 0)
    {
        if has_upbit && !has_equity {
            return (
                OfficialEvidenceExpansionStatus::CryptoOnly,
                OfficialEvidenceAcquisitionRecommendation::RunCryptoOnlyEvidence,
                blockers,
                warnings,
                dedupe_reasons(reason_codes),
            );
        }
        if has_upbit && has_equity {
            return (
                OfficialEvidenceExpansionStatus::MoreOfficialEvidence,
                OfficialEvidenceAcquisitionRecommendation::RunMultiVenueEvidence,
                blockers,
                warnings,
                dedupe_reasons(reason_codes),
            );
        }
        blockers.push("no ready provider collection could be executed".to_string());
        if missing_krx_endpoint {
            return (
                OfficialEvidenceExpansionStatus::MissingAuth,
                OfficialEvidenceAcquisitionRecommendation::SetKrxEndpointTemplate,
                blockers,
                warnings,
                dedupe_reasons(reason_codes),
            );
        }
        if missing_krx_auth {
            return (
                OfficialEvidenceExpansionStatus::MissingAuth,
                OfficialEvidenceAcquisitionRecommendation::SetKrxAuth,
                blockers,
                warnings,
                dedupe_reasons(reason_codes),
            );
        }
        if missing_alpha_auth {
            return (
                OfficialEvidenceExpansionStatus::MissingAuth,
                OfficialEvidenceAcquisitionRecommendation::SetAlphaVantageAuth,
                blockers,
                warnings,
                dedupe_reasons(reason_codes),
            );
        }
        return (
            OfficialEvidenceExpansionStatus::MissingOfficialData,
            OfficialEvidenceAcquisitionRecommendation::MoreOfficialEvidence,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }

    if collection_report.is_some_and(|report| {
        report
            .entry_reports
            .iter()
            .all(|entry| entry.provider_kind == ProviderKind::Upbit || !entry.ready_for_evidence)
    }) {
        return (
            OfficialEvidenceExpansionStatus::CryptoOnly,
            OfficialEvidenceAcquisitionRecommendation::RunCryptoOnlyEvidence,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }

    (
        OfficialEvidenceExpansionStatus::MoreOfficialEvidence,
        OfficialEvidenceAcquisitionRecommendation::RunMultiVenueEvidence,
        blockers,
        warnings,
        dedupe_reasons(reason_codes),
    )
}

fn map_expansion_recommendation(
    recommendation: OfficialEvidenceExpansionRecommendation,
) -> OfficialEvidenceAcquisitionRecommendation {
    match recommendation {
        OfficialEvidenceExpansionRecommendation::ImproveDataFirst
        | OfficialEvidenceExpansionRecommendation::MoreOfficialEvidence => {
            OfficialEvidenceAcquisitionRecommendation::MoreOfficialEvidence
        }
        OfficialEvidenceExpansionRecommendation::ImproveSignalModelFirst => {
            OfficialEvidenceAcquisitionRecommendation::ImproveSignalModelFirst
        }
        OfficialEvidenceExpansionRecommendation::ImproveRiskGovernorFirst => {
            OfficialEvidenceAcquisitionRecommendation::ImproveRiskGovernorFirst
        }
        OfficialEvidenceExpansionRecommendation::ExternalTabularCandidate
        | OfficialEvidenceExpansionRecommendation::BuildSequenceDatasetFirst
        | OfficialEvidenceExpansionRecommendation::HoldCurrentScope
        | OfficialEvidenceExpansionRecommendation::MissingAuth => {
            OfficialEvidenceAcquisitionRecommendation::HoldCurrentScope
        }
    }
}

fn default_collection_plan() -> OfficialCollectionPlan {
    OfficialCollectionPlan {
        plan_id: "default_official_evidence_acquisition".to_string(),
        output_root: "target/soma_official_collection".to_string(),
        max_total_bytes: default_storage(),
        max_total_rows: default_three() * default_five_hundred(),
        max_total_requests: default_ten(),
        default_collection_size_policy: CollectionSizePolicy {
            max_symbols_per_run: default_three(),
            max_rows_per_symbol: default_five_hundred(),
            max_total_rows_per_run: default_three() * default_five_hundred(),
            max_raw_bytes_per_run: default_storage() / 2,
            max_canonical_bytes_per_run: default_storage() / 3,
            max_requests_per_run: default_ten(),
            max_days_per_run: 365,
            default_outputsize: CollectionOutputSize::Compact,
            raw_archive_policy: RawArchivePolicy::CompactJson,
            retention_policy: RetentionPolicy::DeleteRawAfterCanonicalAndManifest,
            allow_full_history: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        default_compression_policy: CompressionPolicy::default(),
        default_retention_policy: RetentionPolicy::DeleteRawAfterCanonicalAndManifest,
        storage_budget: StorageBudget {
            max_total_bytes: default_storage(),
            max_raw_bytes: default_storage() / 2,
            max_canonical_bytes: default_storage() / 3,
            max_manifest_bytes: default_storage() / 10,
            max_file_count: 64,
            reason_codes: vec![ReasonCode::DeterministicPath],
        },
        entries: vec![
            OfficialCollectionEntry {
                entry_id: "upbit-btc".to_string(),
                provider_kind: ProviderKind::Upbit,
                symbol: "KRW-BTC".to_string(),
                normalized_symbol: None,
                venue: Some(MarketVenue::Upbit),
                asset_class: AssetClass::Crypto,
                timeframe: Timeframe::OneMinute,
                start: None,
                end: None,
                max_rows: Some(default_five_hundred()),
                max_requests: Some(4),
                outputsize: Some(CollectionOutputSize::Compact),
                auth_config_ref: None,
                endpoint_template: None,
                fixture_path: None,
                enabled: true,
                tags: vec!["crypto".to_string(), "public".to_string()],
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            OfficialCollectionEntry {
                entry_id: "krx-005930".to_string(),
                provider_kind: ProviderKind::KrxOpenApi,
                symbol: "005930".to_string(),
                normalized_symbol: None,
                venue: Some(MarketVenue::KOSPI),
                asset_class: AssetClass::Equity,
                timeframe: Timeframe::OneDay,
                start: None,
                end: None,
                max_rows: Some(200),
                max_requests: Some(2),
                outputsize: Some(CollectionOutputSize::Compact),
                auth_config_ref: Some(AuthConfig {
                    provider_kind: ProviderKind::KrxOpenApi,
                    api_key_env_var: Some("KRX_API_KEY".to_string()),
                    api_secret_env_var: None,
                    auth_header_name: Some("Authorization".to_string()),
                    query_param_name: None,
                    allow_missing_for_mock: false,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }),
                endpoint_template: Some(
                    "https://krx.example.local/daily?symbol={symbol}&date={date}".to_string(),
                ),
                fixture_path: None,
                enabled: true,
                tags: vec!["krx".to_string(), "official".to_string()],
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
            OfficialCollectionEntry {
                entry_id: "us-aapl".to_string(),
                provider_kind: ProviderKind::AlphaVantage,
                symbol: "AAPL".to_string(),
                normalized_symbol: None,
                venue: Some(MarketVenue::NASDAQ),
                asset_class: AssetClass::Equity,
                timeframe: Timeframe::OneDay,
                start: None,
                end: None,
                max_rows: Some(100),
                max_requests: Some(2),
                outputsize: Some(CollectionOutputSize::Compact),
                auth_config_ref: Some(AuthConfig {
                    provider_kind: ProviderKind::AlphaVantage,
                    api_key_env_var: Some("ALPHAVANTAGE_API_KEY".to_string()),
                    api_secret_env_var: None,
                    auth_header_name: None,
                    query_param_name: Some("apikey".to_string()),
                    allow_missing_for_mock: false,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                }),
                endpoint_template: None,
                fixture_path: None,
                enabled: true,
                tags: vec!["us".to_string(), "official".to_string()],
                reason_codes: vec![ReasonCode::DeterministicPath],
            },
        ],
        continue_on_missing_auth: true,
        continue_on_provider_failure: true,
        reason_codes: vec![ReasonCode::DeterministicPath],
    }
}

fn dedupe_reasons(reason_codes: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for reason in reason_codes {
        if !deduped.contains(&reason) {
            deduped.push(reason);
        }
    }
    deduped
}

fn default_true() -> bool {
    true
}

fn default_three() -> usize {
    3
}

fn default_five_hundred() -> usize {
    500
}

fn default_ten() -> usize {
    10
}

fn default_storage() -> usize {
    16 * 1024 * 1024
}

fn default_output_root() -> String {
    "target/soma_official_acquisition".to_string()
}
