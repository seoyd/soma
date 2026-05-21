use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{CoreCheckConfig, CoreReadinessStatus, ReasonCode};
use crate::data::{
    OfficialCollectionPlan, OfficialCollectionReport, OfficialCollectionRunner,
    ProviderAuthPreflightConfig, ProviderAuthPreflightReport, ProviderAuthPreflightRunner,
};
use crate::experiment::{
    AuthSetupGuide, CoreCheckedBenchmarkConfig, CoreCheckedBenchmarkRecommendation,
    CoreCheckedBenchmarkReport, CoreCheckedBenchmarkRunner, CoreCheckedBenchmarkStatus,
    OfficialEvidenceDelta, OfficialStorageDelta, PreviousCollectionComparison,
    VenueCoverageExpansionPlan, VenueCoverageExpansionReport, VenueCoverageStatus,
    build_auth_setup_guide, build_official_evidence_delta, build_official_storage_delta,
    build_previous_collection_comparison, build_venue_coverage_report,
    load_previous_collection_report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceExpansionConfig {
    pub expansion_id: String,
    #[serde(default)]
    pub auth_preflight_config: Option<ProviderAuthPreflightConfig>,
    pub venue_coverage_plan: VenueCoverageExpansionPlan,
    #[serde(default)]
    pub official_collection_plan_path: Option<String>,
    #[serde(default)]
    pub previous_collection_report_path: Option<String>,
    #[serde(default)]
    pub previous_core_benchmark_report_path: Option<String>,
    #[serde(default = "default_true")]
    pub run_auth_preflight: bool,
    #[serde(default)]
    pub run_collection: bool,
    #[serde(default = "default_true")]
    pub run_core_benchmark: bool,
    #[serde(default)]
    pub run_external_eval: bool,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub core_check_config: Option<CoreCheckConfig>,
    #[serde(default = "default_true")]
    pub require_core_check: bool,
    #[serde(default = "default_allowed_core_statuses")]
    pub allowed_core_statuses: Vec<CoreReadinessStatus>,
    #[serde(default = "default_true")]
    pub strict_schema_validation: bool,
    #[serde(default = "default_one")]
    pub min_total_ready_datasets: usize,
    #[serde(default = "default_twenty")]
    pub min_total_outcome_records: usize,
    #[serde(default = "default_one")]
    pub min_comparable_model_reports: usize,
    #[serde(default = "default_storage")]
    pub max_storage_bytes: usize,
    #[serde(default = "default_true")]
    pub continue_on_missing_auth: bool,
    #[serde(default = "default_true")]
    pub continue_on_provider_failure: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreviousBenchmarkSummary {
    pub comparable: bool,
    pub ready_datasets: usize,
    pub outcome_records: usize,
    pub status: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialEvidenceExpansionStatus {
    MissingAuth,
    MissingOfficialData,
    CryptoOnly,
    InsufficientOutcomes,
    BaselineOnlyEvaluated,
    ExternalModelEvaluated,
    ExternalTabularCandidate,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    MoreOfficialEvidence,
    HoldCurrentScope,
    CoreBlocked,
    BenchmarkBlocked,
    StorageBudgetBlocked,
    PreflightBlocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialEvidenceExpansionRecommendation {
    MoreOfficialEvidence,
    MissingAuth,
    ImproveDataFirst,
    ImproveSignalModelFirst,
    ImproveRiskGovernorFirst,
    ExternalTabularCandidate,
    BuildSequenceDatasetFirst,
    HoldCurrentScope,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceExpansionReport {
    pub expansion_id: String,
    #[serde(default)]
    pub auth_preflight_report: Option<ProviderAuthPreflightReport>,
    pub venue_coverage_report: VenueCoverageExpansionReport,
    #[serde(default)]
    pub collection_report: Option<OfficialCollectionReport>,
    #[serde(default)]
    pub core_benchmark_report: Option<CoreCheckedBenchmarkReport>,
    #[serde(default)]
    pub previous_benchmark_summary: Option<PreviousBenchmarkSummary>,
    #[serde(default)]
    pub previous_collection_comparison: Option<PreviousCollectionComparison>,
    pub evidence_delta: OfficialEvidenceDelta,
    pub storage_delta: OfficialStorageDelta,
    pub auth_setup_guides: Vec<AuthSetupGuide>,
    #[serde(default)]
    pub nested_benchmark_status: Option<String>,
    pub final_status: OfficialEvidenceExpansionStatus,
    pub final_recommendation: OfficialEvidenceExpansionRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceExpansionRunner;

impl Default for OfficialEvidenceExpansionConfig {
    fn default() -> Self {
        Self {
            expansion_id: "official_evidence_expansion".to_string(),
            auth_preflight_config: Some(ProviderAuthPreflightConfig::default()),
            venue_coverage_plan: VenueCoverageExpansionPlan::default(),
            official_collection_plan_path: None,
            previous_collection_report_path: None,
            previous_core_benchmark_report_path: None,
            run_auth_preflight: true,
            run_collection: false,
            run_core_benchmark: true,
            run_external_eval: false,
            output_root: default_output_root(),
            core_check_config: None,
            require_core_check: true,
            allowed_core_statuses: default_allowed_core_statuses(),
            strict_schema_validation: true,
            min_total_ready_datasets: 1,
            min_total_outcome_records: 20,
            min_comparable_model_reports: 1,
            max_storage_bytes: default_storage(),
            continue_on_missing_auth: true,
            continue_on_provider_failure: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl OfficialEvidenceExpansionConfig {
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
            self.previous_collection_report_path.as_deref(),
            self.previous_core_benchmark_report_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reasons.push(ReasonCode::RemotePathRejected);
            }
        }
        reasons.extend(self.venue_coverage_plan.validate_local_paths());
        dedupe_reasons(reasons)
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.expansion_id)
    }
}

impl OfficialEvidenceExpansionReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("expansion_id={}", self.expansion_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "coverage_status={:?}",
                self.venue_coverage_report.coverage_status
            ),
            format!(
                "auth_safe_to_collect={}",
                self.auth_preflight_report
                    .as_ref()
                    .map(|report| report.safe_to_collect)
                    .unwrap_or(true)
            ),
            self.venue_coverage_report.to_text(),
            self.evidence_delta.to_text(),
            self.storage_delta.to_text(),
            self.previous_collection_comparison
                .as_ref()
                .map(|comparison| comparison.to_text())
                .unwrap_or_default(),
            format!(
                "nested_benchmark_status={}",
                self.nested_benchmark_status.clone().unwrap_or_default()
            ),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn to_markdown(&self) -> String {
        [
            format!("# {}", self.expansion_id),
            format!("- final_status: `{:?}`", self.final_status),
            format!("- final_recommendation: `{:?}`", self.final_recommendation),
            format!(
                "- coverage_status: `{:?}`",
                self.venue_coverage_report.coverage_status
            ),
            format!(
                "- nested_benchmark_status: `{}`",
                self.nested_benchmark_status.clone().unwrap_or_default()
            ),
            format!("- blockers: {}", self.blockers.join(", ")),
            format!("- warnings: {}", self.warnings.join(", ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_expansion_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_expansion_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_expansion_report.md"),
            self.to_markdown(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl OfficialEvidenceExpansionRunner {
    pub fn run(
        &self,
        config: &OfficialEvidenceExpansionConfig,
    ) -> Result<OfficialEvidenceExpansionReport, String> {
        if config
            .validate_local_paths()
            .contains(&ReasonCode::RemotePathRejected)
        {
            return Err("official evidence expansion paths must be local".to_string());
        }

        let auth_preflight_report = if config.run_auth_preflight {
            config
                .auth_preflight_config
                .as_ref()
                .map(|preflight| ProviderAuthPreflightRunner::default().run(preflight))
        } else {
            None
        };

        let collection_report =
            load_or_run_collection_report(config, auth_preflight_report.as_ref())?;
        let (previous_collection_report, previous_collection_reason_codes) =
            load_previous_collection_report(config.previous_collection_report_path.as_deref())?;
        let previous_benchmark_report =
            load_previous_core_benchmark(config.previous_core_benchmark_report_path.as_deref())?;
        let (core_benchmark_report, benchmark_error) = if config.run_core_benchmark {
            run_core_benchmark(config, collection_report.as_ref())?
        } else {
            (None, None)
        };

        let venue_coverage_report = build_venue_coverage_report(
            &config.venue_coverage_plan,
            collection_report.as_ref(),
            auth_preflight_report.as_ref(),
        );
        let evidence_delta = build_official_evidence_delta(
            previous_benchmark_report.as_ref(),
            core_benchmark_report.as_ref(),
            &venue_coverage_report,
        );
        let storage_delta = build_official_storage_delta(
            previous_benchmark_report.as_ref(),
            core_benchmark_report.as_ref(),
            collection_report.as_ref(),
            config.max_storage_bytes,
        );
        let auth_setup_guides = vec![
            build_auth_setup_guide(crate::data::ProviderKind::KrxOpenApi),
            build_auth_setup_guide(crate::data::ProviderKind::AlphaVantage),
            build_auth_setup_guide(crate::data::ProviderKind::Alpaca),
        ];
        let previous_collection_comparison =
            if config.previous_collection_report_path.is_some() || collection_report.is_some() {
                Some(build_previous_collection_comparison(
                    previous_collection_report.as_ref(),
                    collection_report.as_ref(),
                    auth_preflight_report.as_ref(),
                    config.previous_collection_report_path.is_some(),
                    &previous_collection_reason_codes,
                ))
            } else {
                None
            };
        let previous_benchmark_summary =
            previous_benchmark_report
                .as_ref()
                .map(|report| PreviousBenchmarkSummary {
                    comparable: true,
                    ready_datasets: report
                        .dataset_selection
                        .as_ref()
                        .map(|selection| selection.selected_entries.len())
                        .unwrap_or_default(),
                    outcome_records: report
                        .dataset_bundle
                        .as_ref()
                        .map(|bundle| bundle.label_counts.values().sum())
                        .unwrap_or_default(),
                    status: format!("{:?}", report.final_status),
                });

        let (final_status, final_recommendation, blockers, mut warnings, mut reason_codes) =
            classify_official_evidence_expansion_state(
                config,
                auth_preflight_report.as_ref(),
                collection_report.as_ref(),
                &venue_coverage_report,
                core_benchmark_report.as_ref(),
                benchmark_error.as_deref(),
                &storage_delta,
            );
        warnings.extend(
            auth_preflight_report
                .as_ref()
                .map(|report| report.warnings.clone())
                .unwrap_or_default(),
        );
        warnings.extend(venue_coverage_report.warnings.clone());
        if let Some(error) = &benchmark_error {
            warnings.push(error.clone());
        }
        reason_codes.extend(previous_collection_reason_codes);
        reason_codes.push(ReasonCode::OfficialEvidenceExpansionRan);
        let nested_benchmark_status = benchmark_error
            .as_ref()
            .map(|_| "BenchmarkExecutionError".to_string())
            .or_else(|| report_nested_benchmark_status(core_benchmark_report.as_ref()));

        let report = OfficialEvidenceExpansionReport {
            expansion_id: config.expansion_id.clone(),
            auth_preflight_report,
            venue_coverage_report,
            collection_report,
            core_benchmark_report,
            previous_benchmark_summary,
            previous_collection_comparison,
            evidence_delta,
            storage_delta,
            auth_setup_guides,
            nested_benchmark_status,
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: dedupe_reasons(reason_codes),
        };
        report.write_to_dir(&config.output_dir())?;
        Ok(report)
    }
}

fn run_core_benchmark(
    config: &OfficialEvidenceExpansionConfig,
    collection_report: Option<&OfficialCollectionReport>,
) -> Result<(Option<CoreCheckedBenchmarkReport>, Option<String>), String> {
    let Some(collection_report) = collection_report else {
        return Ok((None, None));
    };
    let output_dir = config.output_dir();
    let collection_report_path = collection_report.write_to_dir(&output_dir)?;
    match CoreCheckedBenchmarkRunner::default().run(&CoreCheckedBenchmarkConfig {
        benchmark_id: format!("{}-core-benchmark", config.expansion_id),
        core_check_config: Some(config.core_check_config.clone().unwrap_or(CoreCheckConfig {
            check_id: format!("{}-core-check", config.expansion_id),
            output_root: output_dir.join("core_check").display().to_string(),
            ..CoreCheckConfig::default()
        })),
        require_core_ready: config.require_core_check,
        allowed_core_statuses: config.allowed_core_statuses.clone(),
        official_collection_report_path: Some(collection_report_path.display().to_string()),
        output_root: output_dir.display().to_string(),
        strict_schema_validation: config.strict_schema_validation,
        run_external_eval: config.run_external_eval,
        max_allowed_storage_bytes: config.max_storage_bytes,
        min_ready_official_datasets: config.min_total_ready_datasets,
        min_outcome_records: config.min_total_outcome_records,
        ..CoreCheckedBenchmarkConfig::default()
    }) {
        Ok(report) => Ok((Some(report), None)),
        Err(err) => Ok((None, Some(err.to_string()))),
    }
}

fn load_or_run_collection_report(
    config: &OfficialEvidenceExpansionConfig,
    auth_preflight_report: Option<&ProviderAuthPreflightReport>,
) -> Result<Option<OfficialCollectionReport>, String> {
    if config.run_collection {
        let plan_path = config
            .official_collection_plan_path
            .as_ref()
            .or(config.venue_coverage_plan.collection_plan_path.as_ref())
            .ok_or_else(|| {
                "official_collection_plan_path required when run_collection=true".to_string()
            })?;
        let mut plan = OfficialCollectionPlan::from_toml_path(Path::new(plan_path))?;
        plan.continue_on_missing_auth = config.continue_on_missing_auth;
        plan.continue_on_provider_failure = config.continue_on_provider_failure;
        if let Some(auth_report) = auth_preflight_report {
            let blocked = auth_report
                .statuses
                .iter()
                .filter(|status| {
                    matches!(
                        status.status,
                        crate::data::ProviderAuthStatusKind::MissingAuth
                            | crate::data::ProviderAuthStatusKind::MissingEndpointTemplate
                            | crate::data::ProviderAuthStatusKind::UnsafeSecretExposure
                    )
                })
                .map(|status| status.provider_kind)
                .collect::<Vec<_>>();
            if !config.continue_on_missing_auth && !blocked.is_empty() {
                return Ok(None);
            }
            plan.entries.retain(|entry| {
                !blocked.contains(&entry.provider_kind) || entry.fixture_path.is_some()
            });
        }
        return Ok(Some(OfficialCollectionRunner::default().run_plan(&plan)));
    }
    if let Some(path) = config
        .venue_coverage_plan
        .existing_collection_report_path
        .as_deref()
    {
        return Ok(Some(OfficialCollectionReport::from_json_path(Path::new(
            path,
        ))?));
    }
    Ok(None)
}

fn load_previous_core_benchmark(
    path: Option<&str>,
) -> Result<Option<CoreCheckedBenchmarkReport>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|err| err.to_string())
}

pub fn classify_official_evidence_expansion_state(
    config: &OfficialEvidenceExpansionConfig,
    auth_preflight_report: Option<&ProviderAuthPreflightReport>,
    collection_report: Option<&OfficialCollectionReport>,
    venue_coverage_report: &VenueCoverageExpansionReport,
    core_benchmark_report: Option<&CoreCheckedBenchmarkReport>,
    benchmark_error: Option<&str>,
    storage_delta: &OfficialStorageDelta,
) -> (
    OfficialEvidenceExpansionStatus,
    OfficialEvidenceExpansionRecommendation,
    Vec<String>,
    Vec<String>,
    Vec<ReasonCode>,
) {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut reason_codes = vec![ReasonCode::OfficialEvidenceExpansionRan];
    if storage_delta.budget_exceeded {
        blockers.push("storage budget exceeded".to_string());
        reason_codes.push(ReasonCode::BudgetExceeded);
        return (
            OfficialEvidenceExpansionStatus::StorageBudgetBlocked,
            OfficialEvidenceExpansionRecommendation::HoldCurrentScope,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }
    if auth_preflight_report.is_some_and(|report| {
        report.statuses.iter().any(|status| {
            matches!(
                status.status,
                crate::data::ProviderAuthStatusKind::UnsafeSecretExposure
            )
        })
    }) || collection_report.is_some_and(|report| {
        report.entry_reports.iter().any(|entry| {
            matches!(
                entry.status,
                crate::data::OfficialCollectionEntryStatus::FailedPreflight
            )
        })
    }) {
        blockers.push("preflight blocked evidence expansion".to_string());
        reason_codes.push(ReasonCode::PreflightFailed);
        return (
            OfficialEvidenceExpansionStatus::PreflightBlocked,
            OfficialEvidenceExpansionRecommendation::HoldCurrentScope,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }
    if let Some(error) = benchmark_error {
        blockers.push(format!("core-benchmark execution failed: {error}"));
        return (
            OfficialEvidenceExpansionStatus::BenchmarkBlocked,
            OfficialEvidenceExpansionRecommendation::HoldCurrentScope,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }
    if auth_preflight_report
        .is_some_and(|report| !report.safe_to_collect && !config.continue_on_missing_auth)
    {
        blockers.push("auth preflight blocked required provider collection".to_string());
        reason_codes.push(ReasonCode::MissingAuth);
        return (
            OfficialEvidenceExpansionStatus::MissingAuth,
            OfficialEvidenceExpansionRecommendation::MissingAuth,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }
    if matches!(
        venue_coverage_report.coverage_status,
        VenueCoverageStatus::NoOfficialData
    ) {
        blockers.push("no official-ready evidence available".to_string());
        return (
            OfficialEvidenceExpansionStatus::MissingOfficialData,
            OfficialEvidenceExpansionRecommendation::MoreOfficialEvidence,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }
    if matches!(
        venue_coverage_report.coverage_status,
        VenueCoverageStatus::MissingAuth
    ) {
        warnings.push("missing auth blocks some venue claims".to_string());
        reason_codes.push(ReasonCode::MissingAuth);
        return (
            OfficialEvidenceExpansionStatus::MissingAuth,
            OfficialEvidenceExpansionRecommendation::MissingAuth,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    }
    if matches!(
        venue_coverage_report.coverage_status,
        VenueCoverageStatus::CryptoOnly
    ) {
        warnings.push("official evidence remains crypto-only".to_string());
    }
    let Some(core_report) = core_benchmark_report else {
        return (
            if matches!(
                venue_coverage_report.coverage_status,
                VenueCoverageStatus::CryptoOnly
            ) {
                OfficialEvidenceExpansionStatus::CryptoOnly
            } else {
                OfficialEvidenceExpansionStatus::MoreOfficialEvidence
            },
            OfficialEvidenceExpansionRecommendation::MoreOfficialEvidence,
            blockers,
            warnings,
            dedupe_reasons(reason_codes),
        );
    };

    let mapped = match core_report.final_status {
        CoreCheckedBenchmarkStatus::MissingAuth => (
            OfficialEvidenceExpansionStatus::MissingAuth,
            OfficialEvidenceExpansionRecommendation::MissingAuth,
        ),
        CoreCheckedBenchmarkStatus::CoreBlocked => (
            OfficialEvidenceExpansionStatus::CoreBlocked,
            OfficialEvidenceExpansionRecommendation::HoldCurrentScope,
        ),
        CoreCheckedBenchmarkStatus::MissingOfficialData => (
            OfficialEvidenceExpansionStatus::MissingOfficialData,
            OfficialEvidenceExpansionRecommendation::MoreOfficialEvidence,
        ),
        CoreCheckedBenchmarkStatus::InsufficientOutcomes => (
            OfficialEvidenceExpansionStatus::InsufficientOutcomes,
            OfficialEvidenceExpansionRecommendation::MoreOfficialEvidence,
        ),
        CoreCheckedBenchmarkStatus::BaselineOnlyEvaluated => (
            if matches!(
                venue_coverage_report.coverage_status,
                VenueCoverageStatus::CryptoOnly
            ) {
                OfficialEvidenceExpansionStatus::CryptoOnly
            } else {
                OfficialEvidenceExpansionStatus::BaselineOnlyEvaluated
            },
            OfficialEvidenceExpansionRecommendation::HoldCurrentScope,
        ),
        CoreCheckedBenchmarkStatus::ExternalModelEvaluated => (
            OfficialEvidenceExpansionStatus::ExternalModelEvaluated,
            OfficialEvidenceExpansionRecommendation::HoldCurrentScope,
        ),
        CoreCheckedBenchmarkStatus::ExternalTabularCandidate => (
            OfficialEvidenceExpansionStatus::ExternalTabularCandidate,
            match core_report.next_recommendation {
                CoreCheckedBenchmarkRecommendation::BuildSequenceDatasetFirst => {
                    OfficialEvidenceExpansionRecommendation::BuildSequenceDatasetFirst
                }
                _ => OfficialEvidenceExpansionRecommendation::ExternalTabularCandidate,
            },
        ),
        CoreCheckedBenchmarkStatus::PoorCalibration
        | CoreCheckedBenchmarkStatus::WorseThanBaseline => (
            OfficialEvidenceExpansionStatus::ImproveSignalModelFirst,
            OfficialEvidenceExpansionRecommendation::ImproveSignalModelFirst,
        ),
        CoreCheckedBenchmarkStatus::PoorRiskBehavior => (
            OfficialEvidenceExpansionStatus::ImproveRiskGovernorFirst,
            OfficialEvidenceExpansionRecommendation::ImproveRiskGovernorFirst,
        ),
        CoreCheckedBenchmarkStatus::NeedMoreExperiments => (
            OfficialEvidenceExpansionStatus::MoreOfficialEvidence,
            OfficialEvidenceExpansionRecommendation::MoreOfficialEvidence,
        ),
    };
    (
        mapped.0,
        mapped.1,
        blockers,
        warnings,
        dedupe_reasons(reason_codes),
    )
}

fn report_nested_benchmark_status(
    core_benchmark_report: Option<&CoreCheckedBenchmarkReport>,
) -> Option<String> {
    core_benchmark_report.map(|report| format!("{:?}", report.final_status))
}

fn default_true() -> bool {
    true
}

fn default_one() -> usize {
    1
}

fn default_twenty() -> usize {
    20
}

fn default_output_root() -> String {
    "target/soma_official_evidence_expansion".to_string()
}

fn default_storage() -> usize {
    16 * 1024 * 1024
}

fn default_allowed_core_statuses() -> Vec<CoreReadinessStatus> {
    vec![
        CoreReadinessStatus::ReadyForMoreOfficialEvidence,
        CoreReadinessStatus::ReadyForExternalModelPrototype,
        CoreReadinessStatus::ReadyForSequenceDatasetBuild,
    ]
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
