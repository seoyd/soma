use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_official_benchmark::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkReport,
    CommitteeOfficialBenchmarkRunner,
};
use super::official_candle_coverage::{
    OfficialCandleCoverageReport, OfficialCandleCoverageRunner, materialize_official_candle_series,
};
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::official_reference_replication::{
    OfficialReferenceReplicationArtifacts, OfficialReferenceReplicationReport,
    OfficialReferenceReplicationRunner,
};
use super::official_replication_bundle::OfficialEvidenceReplicationBundle;
use super::official_replication_inventory::{
    OfficialReplicationArtifactInventory, OfficialReplicationInventoryResolver,
};
use super::official_replication_operator_actions::{
    OfficialReplicationActionPriority, OfficialReplicationOperatorActionPlan,
    OfficialReplicationOperatorActionPlanner,
};
use super::official_row_injection::{
    OfficialRowInjectionPolicy, OfficialRowInjectionResult, OfficialRowInjector,
};
use super::official_sufficiency_replication::{
    OfficialSufficiencyReplicationBuilder, OfficialSufficiencyReplicationReport,
    OfficialSufficiencyReplicationStatus,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OfficialEvidenceReplicationConfig {
    pub replication_id: String,
    #[serde(default)]
    pub provider_readiness_report_paths: Vec<String>,
    #[serde(default)]
    pub provider_reality_report_paths: Vec<String>,
    #[serde(default)]
    pub official_collection_report_paths: Vec<String>,
    #[serde(default)]
    pub official_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub official_preflight_report_paths: Vec<String>,
    #[serde(default)]
    pub official_provenance_paths: Vec<String>,
    #[serde(default)]
    pub evidence_lane_report_paths: Vec<String>,
    #[serde(default)]
    pub official_committee_pack_paths: Vec<String>,
    #[serde(default)]
    pub generated_reference_pack_paths: Vec<String>,
    #[serde(default)]
    pub previous_sufficiency_closure_paths: Vec<String>,
    #[serde(default)]
    pub previous_outcome_coverage_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default = "default_true")]
    pub require_non_crypto_official: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_true")]
    pub require_preflight: bool,
    #[serde(default = "default_true")]
    pub require_local_candles: bool,
    #[serde(default = "default_true")]
    pub require_outcome_links: bool,
    #[serde(default)]
    pub allow_crypto_only: bool,
    #[serde(default)]
    pub allow_yfinance_research: bool,
    #[serde(default)]
    pub allow_fixture: bool,
    #[serde(default)]
    pub allow_controlled_fixture: bool,
    #[serde(default)]
    pub allow_summary_derived_rows: bool,
    #[serde(default = "default_true")]
    pub run_pack_generation: bool,
    #[serde(default = "default_true")]
    pub run_reference_generation: bool,
    #[serde(default = "default_true")]
    pub run_sufficiency_closure: bool,
    #[serde(default = "default_true")]
    pub run_official_committee_benchmark: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceReplicationFinalStatus {
    OfficialReplicationReady,
    OfficialBenchmarkReady,
    ControlledOnly,
    CryptoOnly,
    MissingOfficialAuth,
    MissingOfficialData,
    MissingOfficialCandles,
    MissingOfficialProvenance,
    MissingOfficialPreflight,
    MissingOutcomeLinks,
    MissingCounterfactualDepth,
    NeedMoreOfficialEvidence,
    NeedMoreEvidence,
    #[default]
    Unknown,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceReplicationRecommendation {
    RunProviderReadiness,
    SetKrxAuth,
    SetKrxEndpointTemplate,
    SetAlphaVantageAuth,
    RunOfficialCollection,
    ProvideOfficialCanonicalCsv,
    ProvideOfficialCandleSeries,
    BuildOfficialReferences,
    RunOfficialCommitteeBenchmark,
    MoreOfficialCommitteeEvidence,
    KeepTrinity,
    NeedMoreEvidence,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceReplicationReport {
    pub replication_id: String,
    pub artifact_inventory: OfficialReplicationArtifactInventory,
    pub row_injection_result: OfficialRowInjectionResult,
    pub official_candle_coverage_report: OfficialCandleCoverageReport,
    #[serde(default)]
    pub official_reference_replication_report: Option<OfficialReferenceReplicationReport>,
    pub official_sufficiency_replication_report: OfficialSufficiencyReplicationReport,
    #[serde(default)]
    pub official_committee_benchmark_report: Option<CommitteeOfficialBenchmarkReport>,
    pub operator_action_plan: OfficialReplicationOperatorActionPlan,
    pub storage_summary: String,
    pub final_status: OfficialEvidenceReplicationFinalStatus,
    pub final_recommendation: OfficialEvidenceReplicationRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialEvidenceReplicationRunner;

impl Default for OfficialEvidenceReplicationConfig {
    fn default() -> Self {
        Self {
            replication_id: "official_evidence_replication".to_string(),
            provider_readiness_report_paths: Vec::new(),
            provider_reality_report_paths: Vec::new(),
            official_collection_report_paths: Vec::new(),
            official_canonical_csv_paths: Vec::new(),
            official_preflight_report_paths: Vec::new(),
            official_provenance_paths: Vec::new(),
            evidence_lane_report_paths: Vec::new(),
            official_committee_pack_paths: Vec::new(),
            generated_reference_pack_paths: Vec::new(),
            previous_sufficiency_closure_paths: Vec::new(),
            previous_outcome_coverage_paths: Vec::new(),
            output_root: default_output_root(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            require_non_crypto_official: true,
            require_provenance: true,
            require_preflight: true,
            require_local_candles: true,
            require_outcome_links: true,
            allow_crypto_only: false,
            allow_yfinance_research: false,
            allow_fixture: false,
            allow_controlled_fixture: false,
            allow_summary_derived_rows: false,
            run_pack_generation: true,
            run_reference_generation: true,
            run_sufficiency_closure: true,
            run_official_committee_benchmark: true,
            reason_codes: vec![ReasonCode::OfficialEvidenceReplicationConfigValidated],
        }
    }
}

impl OfficialEvidenceReplicationConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        if self.replication_id.trim().is_empty() {
            return Err("official replication id must not be empty".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("official replication max_rows must be between 1 and 500".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err("official replication max_symbols must be between 1 and 5".to_string());
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err("official replication max_bytes must be between 1 and 5000000".to_string());
        }
        if self
            .all_artifact_paths()
            .iter()
            .any(|path| path.contains("://"))
            || self.output_root.contains("://")
        {
            return Err("official replication paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.replication_id)
    }

    pub fn all_artifact_paths(&self) -> Vec<String> {
        self.provider_readiness_report_paths
            .iter()
            .chain(self.provider_reality_report_paths.iter())
            .chain(self.official_collection_report_paths.iter())
            .chain(self.official_canonical_csv_paths.iter())
            .chain(self.official_preflight_report_paths.iter())
            .chain(self.official_provenance_paths.iter())
            .chain(self.evidence_lane_report_paths.iter())
            .chain(self.official_committee_pack_paths.iter())
            .chain(self.generated_reference_pack_paths.iter())
            .chain(self.previous_sufficiency_closure_paths.iter())
            .chain(self.previous_outcome_coverage_paths.iter())
            .cloned()
            .collect()
    }

    pub fn row_injection_policy(&self) -> OfficialRowInjectionPolicy {
        OfficialRowInjectionPolicy {
            require_preflight_ready: self.require_preflight,
            require_provenance_official: self.require_provenance,
            max_rows_per_symbol: self.max_rows,
            reason_codes: vec![ReasonCode::OfficialRowInjectionBuilt],
            ..OfficialRowInjectionPolicy::default()
        }
    }
}

impl OfficialEvidenceReplicationRunner {
    pub fn run(
        &self,
        config: &OfficialEvidenceReplicationConfig,
    ) -> Result<OfficialEvidenceReplicationReport, String> {
        let bundle = self.run_bundle(config)?;
        Ok(bundle.replication_report)
    }

    pub fn run_bundle(
        &self,
        config: &OfficialEvidenceReplicationConfig,
    ) -> Result<OfficialEvidenceReplicationBundle, String> {
        config.validate()?;
        let inventory = OfficialReplicationInventoryResolver::default().resolve(config);
        let row_injection = OfficialRowInjector::default().inject(
            config,
            &inventory,
            &config.row_injection_policy(),
        )?;
        let candle_series_paths = if config.require_local_candles {
            materialize_official_candle_series(config, &inventory)?
        } else {
            Vec::new()
        };
        let candle_coverage = OfficialCandleCoverageRunner::default()
            .run(&row_injection.injected_rows, &candle_series_paths)?;
        let reference_artifacts = if config.run_reference_generation {
            Some(OfficialReferenceReplicationRunner::default().run(
                config,
                &row_injection,
                &candle_series_paths,
                &candle_coverage,
            )?)
        } else {
            None
        };
        let sufficiency = OfficialSufficiencyReplicationBuilder::default().build(
            config,
            &row_injection,
            reference_artifacts.as_ref(),
        );
        let operator_actions = OfficialReplicationOperatorActionPlanner::default().build(
            config,
            &inventory,
            Some(&candle_coverage),
            Some(&sufficiency),
        );
        let benchmark_report =
            if config.run_official_committee_benchmark && sufficiency.passed_for_official {
                run_official_benchmark(config, reference_artifacts.as_ref())?
            } else {
                None
            };
        let (final_status, final_recommendation) = determine_final_status(
            config,
            &inventory,
            &row_injection,
            &candle_coverage,
            reference_artifacts.as_ref(),
            &sufficiency,
            benchmark_report.as_ref(),
            &operator_actions,
        );
        let mut blockers = operator_actions.blockers.clone();
        if matches!(
            final_status,
            OfficialEvidenceReplicationFinalStatus::MissingOutcomeLinks
        ) {
            blockers.push(
                "outcome links remain insufficient for conservative official closure".to_string(),
            );
        }
        if matches!(
            final_status,
            OfficialEvidenceReplicationFinalStatus::MissingCounterfactualDepth
        ) {
            blockers.push(
                "counterfactual depth remains insufficient for conservative official closure"
                    .to_string(),
            );
        }
        let warnings = vec![
            "research_only_warning=official evidence replication remains research-only and paper-only"
                .to_string(),
            "controlled closure never implies live trading, profitability, or real-money readiness"
                .to_string(),
        ];
        let report = OfficialEvidenceReplicationReport {
            replication_id: config.replication_id.clone(),
            artifact_inventory: inventory.clone(),
            row_injection_result: row_injection.clone(),
            official_candle_coverage_report: candle_coverage.clone(),
            official_reference_replication_report: reference_artifacts
                .as_ref()
                .map(|artifacts| artifacts.report.clone()),
            official_sufficiency_replication_report: sufficiency.clone(),
            official_committee_benchmark_report: benchmark_report.clone(),
            operator_action_plan: operator_actions.clone(),
            storage_summary: format!(
                "output_dir={};candle_series_paths={}",
                config.output_dir().display(),
                candle_series_paths.join("|")
            ),
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialEvidenceReplicationBuilt,
                ReasonCode::OfficialEvidenceReplicationConfigValidated,
            ]),
        };
        let injected_pack = if config.run_pack_generation && !row_injection.injected_rows.is_empty()
        {
            Some(build_injected_pack(config, &row_injection))
        } else {
            None
        };
        let bundle = OfficialEvidenceReplicationBundle::from_parts(
            report,
            inventory,
            injected_pack,
            candle_coverage,
            reference_artifacts
                .as_ref()
                .map(|artifacts| artifacts.report.clone()),
            sufficiency,
            benchmark_report,
            operator_actions,
        );
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }

    pub fn inventory(
        &self,
        config: &OfficialEvidenceReplicationConfig,
    ) -> Result<OfficialReplicationArtifactInventory, String> {
        config.validate()?;
        Ok(OfficialReplicationInventoryResolver::default().resolve(config))
    }

    pub fn row_injection(
        &self,
        config: &OfficialEvidenceReplicationConfig,
    ) -> Result<OfficialRowInjectionResult, String> {
        config.validate()?;
        let inventory = OfficialReplicationInventoryResolver::default().resolve(config);
        OfficialRowInjector::default().inject(config, &inventory, &config.row_injection_policy())
    }
}

impl OfficialEvidenceReplicationReport {
    pub fn to_text(&self) -> String {
        [
            format!("replication_id={}", self.replication_id),
            format!("storage_summary={}", self.storage_summary),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
            self.artifact_inventory.to_text(),
            self.row_injection_result.to_text(),
            self.official_candle_coverage_report.to_text(),
            self.official_reference_replication_report
                .as_ref()
                .map(OfficialReferenceReplicationReport::to_text)
                .unwrap_or_else(|| "official_reference_replication=none".to_string()),
            self.official_sufficiency_replication_report.to_text(),
            self.operator_action_plan.to_text(),
            self.official_committee_benchmark_report
                .as_ref()
                .map(CommitteeOfficialBenchmarkReport::to_text)
                .unwrap_or_else(|| "official_committee_benchmark=none".to_string()),
        ]
        .join("\n")
    }
}

fn run_official_benchmark(
    config: &OfficialEvidenceReplicationConfig,
    reference_artifacts: Option<&OfficialReferenceReplicationArtifacts>,
) -> Result<Option<CommitteeOfficialBenchmarkReport>, String> {
    let Some(artifacts) = reference_artifacts else {
        return Ok(None);
    };
    let Some(linked_pack) = artifacts.linked_pack.as_ref() else {
        return Ok(None);
    };
    let benchmark_dir = config.output_dir().join("benchmark_inputs");
    fs::create_dir_all(&benchmark_dir).map_err(|err| err.to_string())?;
    let linked_pack_path = benchmark_dir.join("outcome_linked_pack.json");
    fs::write(
        &linked_pack_path,
        linked_pack
            .to_json_string()
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let benchmark_config = CommitteeOfficialBenchmarkConfig {
        benchmark_id: format!("{}-official-benchmark", config.replication_id),
        outcome_linked_pack_path: Some(linked_pack_path.display().to_string()),
        output_root: config.output_dir().display().to_string(),
        run_materialization: false,
        run_outcome_linking: false,
        require_core_check: false,
        reason_codes: vec![ReasonCode::CommitteeOfficialBenchmarkBuilt],
        ..CommitteeOfficialBenchmarkConfig::default()
    };
    CommitteeOfficialBenchmarkRunner::default()
        .run(&benchmark_config)
        .map(Some)
}

fn determine_final_status(
    config: &OfficialEvidenceReplicationConfig,
    inventory: &OfficialReplicationArtifactInventory,
    row_injection: &OfficialRowInjectionResult,
    candle_coverage: &OfficialCandleCoverageReport,
    reference_artifacts: Option<&OfficialReferenceReplicationArtifacts>,
    sufficiency: &OfficialSufficiencyReplicationReport,
    benchmark_report: Option<&CommitteeOfficialBenchmarkReport>,
    operator_actions: &OfficialReplicationOperatorActionPlan,
) -> (
    OfficialEvidenceReplicationFinalStatus,
    OfficialEvidenceReplicationRecommendation,
) {
    if benchmark_report.is_some() {
        return (
            OfficialEvidenceReplicationFinalStatus::OfficialBenchmarkReady,
            OfficialEvidenceReplicationRecommendation::RunOfficialCommitteeBenchmark,
        );
    }
    let required_action_ids = operator_actions
        .actions
        .iter()
        .filter(|action| action.priority == OfficialReplicationActionPriority::Required)
        .map(|action| action.action_id.as_str())
        .collect::<Vec<_>>();
    if required_action_ids.iter().any(|id| {
        matches!(
            *id,
            "SetKrxApiKey"
                | "SetKrxEndpointTemplate"
                | "SetAlphaVantageApiKey"
                | "WaitForKrxApproval"
        )
    }) && inventory.non_crypto_official_artifact_count == 0
    {
        let recommendation = if required_action_ids.contains(&"SetKrxEndpointTemplate") {
            OfficialEvidenceReplicationRecommendation::SetKrxEndpointTemplate
        } else if required_action_ids.contains(&"SetAlphaVantageApiKey") {
            OfficialEvidenceReplicationRecommendation::SetAlphaVantageAuth
        } else {
            OfficialEvidenceReplicationRecommendation::SetKrxAuth
        };
        return (
            OfficialEvidenceReplicationFinalStatus::MissingOfficialAuth,
            recommendation,
        );
    }
    if config.require_provenance && inventory.missing_provenance_count > 0 {
        return (
            OfficialEvidenceReplicationFinalStatus::MissingOfficialProvenance,
            OfficialEvidenceReplicationRecommendation::RunOfficialCollection,
        );
    }
    if config.require_preflight && inventory.missing_preflight_count > 0 {
        return (
            OfficialEvidenceReplicationFinalStatus::MissingOfficialPreflight,
            OfficialEvidenceReplicationRecommendation::RunOfficialCollection,
        );
    }
    if row_injection.non_crypto_official_row_count == 0
        && inventory.non_crypto_official_artifact_count == 0
    {
        return (
            OfficialEvidenceReplicationFinalStatus::MissingOfficialData,
            OfficialEvidenceReplicationRecommendation::RunOfficialCollection,
        );
    }
    if config.require_local_candles
        && matches!(
            candle_coverage.coverage_status,
            super::official_candle_coverage::OfficialCandleCoverageStatus::MissingOfficialCandles
                | super::official_candle_coverage::OfficialCandleCoverageStatus::MissingFutureWindow
                | super::official_candle_coverage::OfficialCandleCoverageStatus::InsufficientCoverage
        )
    {
        return (
            OfficialEvidenceReplicationFinalStatus::MissingOfficialCandles,
            OfficialEvidenceReplicationRecommendation::ProvideOfficialCandleSeries,
        );
    }
    if matches!(
        sufficiency.final_status,
        OfficialSufficiencyReplicationStatus::MissingOutcomeLinks
    ) {
        return (
            OfficialEvidenceReplicationFinalStatus::MissingOutcomeLinks,
            OfficialEvidenceReplicationRecommendation::BuildOfficialReferences,
        );
    }
    if matches!(
        sufficiency.final_status,
        OfficialSufficiencyReplicationStatus::MissingCounterfactuals
    ) {
        return (
            OfficialEvidenceReplicationFinalStatus::MissingCounterfactualDepth,
            OfficialEvidenceReplicationRecommendation::BuildOfficialReferences,
        );
    }
    if sufficiency.passed_for_official {
        return (
            OfficialEvidenceReplicationFinalStatus::OfficialReplicationReady,
            OfficialEvidenceReplicationRecommendation::KeepTrinity,
        );
    }
    if row_injection.non_crypto_official_row_count == 0 && row_injection.crypto_only_row_count > 0 {
        return (
            OfficialEvidenceReplicationFinalStatus::CryptoOnly,
            OfficialEvidenceReplicationRecommendation::MoreOfficialCommitteeEvidence,
        );
    }
    if row_injection.official_row_count == 0 && !row_injection.injected_rows.is_empty() {
        return (
            OfficialEvidenceReplicationFinalStatus::ControlledOnly,
            OfficialEvidenceReplicationRecommendation::NeedMoreEvidence,
        );
    }
    if reference_artifacts
        .as_ref()
        .is_some_and(|artifacts| artifacts.report.official_ready_reference_count == 0)
    {
        return (
            OfficialEvidenceReplicationFinalStatus::NeedMoreOfficialEvidence,
            OfficialEvidenceReplicationRecommendation::BuildOfficialReferences,
        );
    }
    (
        OfficialEvidenceReplicationFinalStatus::NeedMoreEvidence,
        OfficialEvidenceReplicationRecommendation::NeedMoreEvidence,
    )
}

fn build_injected_pack(
    config: &OfficialEvidenceReplicationConfig,
    row_injection: &OfficialRowInjectionResult,
) -> OfficialCommitteeScenarioPack {
    let official_row_count = row_injection.official_row_count;
    let crypto_only_row_count = row_injection.crypto_only_row_count;
    let yfinance_row_count = row_injection
        .injected_rows
        .iter()
        .filter(|row| row.evidence_source_kind == crate::data::EvidenceSourceKind::YFinanceResearch)
        .count();
    let fixture_row_count = row_injection
        .injected_rows
        .iter()
        .filter(|row| {
            matches!(
                row.evidence_source_kind,
                crate::data::EvidenceSourceKind::TestFixture
                    | crate::data::EvidenceSourceKind::SyntheticFixture
            )
        })
        .count();
    let row_level_count = row_injection
        .injected_rows
        .iter()
        .filter(|row| {
            row.materialization_level
                == super::committee_scenario_loader::CommitteeScenarioMaterializationLevel::RowLevel
        })
        .count();
    let summary_derived_count = row_injection
        .injected_rows
        .len()
        .saturating_sub(row_level_count);
    let outcome_linked_count = row_injection
        .injected_rows
        .iter()
        .filter(|row| row.outcome_reference.is_some())
        .count();
    let baseline_reference_count = row_injection
        .injected_rows
        .iter()
        .filter(|row| row.baseline_signal_summary.is_some())
        .count();
    let external_reference_count = row_injection
        .injected_rows
        .iter()
        .filter(|row| row.external_prediction_summary.is_some())
        .count();
    OfficialCommitteeScenarioPack {
        pack_id: format!("{}-official-pack", config.replication_id),
        rows: row_injection.injected_rows.clone(),
        source_summary: row_injection
            .injected_rows
            .iter()
            .fold(
                std::collections::BTreeMap::<String, usize>::new(),
                |mut acc, row| {
                    *acc.entry(format!("{:?}", row.evidence_source_kind))
                        .or_insert(0) += 1;
                    acc
                },
            )
            .into_iter()
            .map(|(kind, count)| format!("{kind}={count}"))
            .collect::<Vec<_>>()
            .join("|"),
        official_row_count,
        crypto_only_row_count,
        yfinance_row_count,
        fixture_row_count,
        row_level_count,
        summary_derived_count,
        outcome_linked_count,
        baseline_reference_count,
        external_reference_count,
        no_trade_counterfactual_count: row_injection
            .injected_rows
            .iter()
            .filter(|row| row.no_trade_counterfactual.is_some())
            .count(),
        risk_denial_counterfactual_count: row_injection
            .injected_rows
            .iter()
            .filter(|row| row.risk_denial_counterfactual.is_some())
            .count(),
        storage_bytes: serde_json::to_vec(&row_injection.injected_rows)
            .map(|bytes| bytes.len())
            .unwrap_or_default(),
        reason_codes: stable_reason_codes(&[
            ReasonCode::OfficialEvidenceReplicationBuilt,
            ReasonCode::OfficialCommitteePackBuilt,
        ]),
    }
}

fn default_true() -> bool {
    true
}

fn default_output_root() -> String {
    "target/soma_official_replication".to_string()
}

fn default_max_rows() -> usize {
    500
}

fn default_max_symbols() -> usize {
    5
}

fn default_max_bytes() -> usize {
    5_000_000
}
