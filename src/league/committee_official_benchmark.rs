use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{CoreCheckConfig, CoreCheckRunner, ReasonCode};

use super::committee_actionability::CommitteeActionabilityReport;
use super::committee_attribution::CommitteeAttributionReport;
use super::committee_benchmark::{
    CommitteeBenchmarkConfig, CommitteeBenchmarkFinalStatus, CommitteeBenchmarkReport,
    CommitteeBenchmarkRunner,
};
use super::committee_outcome_linked_comparison::{
    CommitteeOutcomeLinkedComparison, CommitteeOutcomeLinkedComparisonStatus,
    build_committee_outcome_linked_comparison,
};
use super::committee_outcome_linker::{
    CommitteeOutcomeLinker, CommitteeOutcomeLinkerConfig, OutcomeLinkedCommitteeScenarioPack,
};
use super::official_committee_benchmark_bundle::CommitteeOfficialBenchmarkBundle;
use super::official_committee_pack::{
    OfficialCommitteeScenarioPack, OfficialCommitteeScenarioPackBuilder,
    OfficialCommitteeScenarioPackConfig,
};
use super::official_committee_readiness::{
    OfficialCommitteeEvidenceReadinessReport, OfficialCommitteeEvidenceReadinessStatus,
    build_official_committee_evidence_readiness_report,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOfficialBenchmarkConfig {
    pub benchmark_id: String,
    #[serde(default)]
    pub scenario_pack_config_path: Option<String>,
    #[serde(default)]
    pub outcome_linker_config_path: Option<String>,
    #[serde(default)]
    pub outcome_linked_pack_path: Option<String>,
    #[serde(default)]
    pub committee_benchmark_config_path: Option<String>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub require_core_check: bool,
    #[serde(default = "default_true")]
    pub require_outcome_linked_rows: bool,
    #[serde(default = "default_true")]
    pub run_materialization: bool,
    #[serde(default = "default_true")]
    pub run_outcome_linking: bool,
    #[serde(default = "default_true")]
    pub run_committee_benchmark: bool,
    #[serde(default = "default_true")]
    pub run_vs_baseline: bool,
    #[serde(default = "default_true")]
    pub run_actionability: bool,
    #[serde(default = "default_true")]
    pub run_attribution: bool,
    #[serde(default = "default_true")]
    pub run_readiness: bool,
    #[serde(default = "default_min_official_rows")]
    pub min_official_rows: usize,
    #[serde(default = "default_min_outcome_linked_rows")]
    pub min_outcome_linked_rows: usize,
    #[serde(default = "default_min_baseline_linked_rows")]
    pub min_baseline_linked_rows: usize,
    #[serde(default = "default_min_counterfactuals")]
    pub min_no_trade_counterfactuals: usize,
    #[serde(default = "default_min_counterfactuals")]
    pub min_risk_denial_counterfactuals: usize,
    #[serde(default = "default_max_summary_derived_ratio")]
    pub max_summary_derived_ratio: f64,
    #[serde(default = "default_max_research_only_ratio")]
    pub max_research_only_ratio: f64,
    #[serde(default = "default_max_fixture_ratio")]
    pub max_fixture_ratio: f64,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeOfficialBenchmarkFinalStatus {
    OfficialCommitteeBenchmarkReady,
    NeedMoreOfficialRows,
    NeedMoreOutcomeLinks,
    NeedBetterBaselineReferences,
    NeedMoreNoTradeCounterfactuals,
    NeedMoreRiskDeniedCounterfactuals,
    ResearchOnly,
    FixtureOnly,
    CryptoOnly,
    MaterializationWeak,
    CoreBlocked,
    RiskBlockedDominant,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    ImproveRiskGovernorFirst,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOfficialBenchmarkReport {
    pub benchmark_id: String,
    pub scenario_pack_summary: String,
    pub outcome_link_summary: String,
    pub committee_benchmark_report: CommitteeBenchmarkReport,
    pub outcome_linked_vs_baseline_report: CommitteeOutcomeLinkedComparison,
    pub actionability_report: CommitteeActionabilityReport,
    pub attribution_report: CommitteeAttributionReport,
    pub official_evidence_readiness_report: OfficialCommitteeEvidenceReadinessReport,
    pub final_status: CommitteeOfficialBenchmarkFinalStatus,
    pub final_recommendation: String,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeOfficialBenchmarkRunner;

struct OfficialRunArtifacts {
    pack: OfficialCommitteeScenarioPack,
    linked_pack: OutcomeLinkedCommitteeScenarioPack,
    benchmark_report: CommitteeBenchmarkReport,
    comparison: CommitteeOutcomeLinkedComparison,
    readiness: OfficialCommitteeEvidenceReadinessReport,
    report: CommitteeOfficialBenchmarkReport,
}

impl Default for CommitteeOfficialBenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_id: "committee_official_benchmark".to_string(),
            scenario_pack_config_path: None,
            outcome_linker_config_path: None,
            outcome_linked_pack_path: None,
            committee_benchmark_config_path: None,
            output_root: "target/soma_committee_official_benchmark".to_string(),
            require_core_check: true,
            require_outcome_linked_rows: true,
            run_materialization: true,
            run_outcome_linking: true,
            run_committee_benchmark: true,
            run_vs_baseline: true,
            run_actionability: true,
            run_attribution: true,
            run_readiness: true,
            min_official_rows: default_min_official_rows(),
            min_outcome_linked_rows: default_min_outcome_linked_rows(),
            min_baseline_linked_rows: default_min_baseline_linked_rows(),
            min_no_trade_counterfactuals: default_min_counterfactuals(),
            min_risk_denial_counterfactuals: default_min_counterfactuals(),
            max_summary_derived_ratio: default_max_summary_derived_ratio(),
            max_research_only_ratio: default_max_research_only_ratio(),
            max_fixture_ratio: default_max_fixture_ratio(),
            reason_codes: vec![ReasonCode::CommitteeOfficialBenchmarkBuilt],
        }
    }
}

impl CommitteeOfficialBenchmarkConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut path_iter = self
            .scenario_pack_config_path
            .iter()
            .cloned()
            .chain(self.outcome_linker_config_path.iter().cloned())
            .chain(self.outcome_linked_pack_path.iter().cloned())
            .chain(self.committee_benchmark_config_path.iter().cloned())
            .chain(std::iter::once(self.output_root.clone()));
        if path_iter.any(|path| path.contains("://")) {
            return Err("committee official benchmark paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.benchmark_id)
    }
}

impl CommitteeOfficialBenchmarkRunner {
    pub fn run(
        &self,
        config: &CommitteeOfficialBenchmarkConfig,
    ) -> Result<CommitteeOfficialBenchmarkReport, String> {
        self.execute(config).map(|artifacts| artifacts.report)
    }

    pub fn run_bundle(
        &self,
        config: &CommitteeOfficialBenchmarkConfig,
    ) -> Result<CommitteeOfficialBenchmarkBundle, String> {
        let artifacts = self.execute(config)?;
        let storage_summary = format!("output_dir={}", config.output_dir().display());
        let mut bundle = CommitteeOfficialBenchmarkBundle {
            official_scenario_pack: artifacts.pack,
            outcome_linked_pack: artifacts.linked_pack,
            committee_benchmark_report: artifacts.benchmark_report,
            outcome_linked_comparison: artifacts.comparison,
            official_readiness_report: artifacts.readiness,
            audit_summary: String::new(),
            storage_summary,
            final_summary: artifacts.report.to_text(),
            reason_codes: vec![ReasonCode::CommitteeOfficialBenchmarkBundleBuilt],
        };
        bundle.audit_summary = bundle.build_audit_summary();
        Ok(bundle)
    }

    fn execute(
        &self,
        config: &CommitteeOfficialBenchmarkConfig,
    ) -> Result<OfficialRunArtifacts, String> {
        config.validate()?;
        if config.require_core_check {
            CoreCheckRunner::default()
                .run(&CoreCheckConfig::default())
                .map_err(|err| format!("core check failed: {err}"))?;
        }
        let pack = self.load_pack(config)?;
        let linked_pack = self.load_linked_pack(config, &pack)?;
        let benchmark_bundle = self.run_committee_benchmark(config, &linked_pack)?;
        let comparison = build_committee_outcome_linked_comparison(
            &linked_pack,
            &benchmark_bundle.replay_report,
            config.min_outcome_linked_rows,
        );
        let readiness = build_official_committee_evidence_readiness_report(
            &pack,
            Some(&linked_pack),
            config.min_official_rows,
            config.min_outcome_linked_rows,
            config.min_baseline_linked_rows,
            config.min_no_trade_counterfactuals,
            config.min_risk_denial_counterfactuals,
            config.max_summary_derived_ratio,
            config.max_research_only_ratio,
            config.max_fixture_ratio,
        );
        let blockers = collect_blockers(config, &linked_pack, &readiness, &comparison);
        let warnings = collect_warnings(&pack, &linked_pack, &comparison);
        let final_status = map_final_status(
            config,
            &benchmark_bundle.benchmark_report,
            &readiness,
            &comparison,
        );
        let final_recommendation = recommendation_for_status(final_status).to_string();
        let report = CommitteeOfficialBenchmarkReport {
            benchmark_id: config.benchmark_id.clone(),
            scenario_pack_summary: pack.to_text(),
            outcome_link_summary: linked_pack.link_summary.to_text(),
            committee_benchmark_report: benchmark_bundle.benchmark_report.clone(),
            outcome_linked_vs_baseline_report: comparison.clone(),
            actionability_report: benchmark_bundle.actionability_report.clone(),
            attribution_report: benchmark_bundle.attribution_report.clone(),
            official_evidence_readiness_report: readiness.clone(),
            final_status,
            final_recommendation,
            blockers,
            warnings,
            reason_codes: config
                .reason_codes
                .iter()
                .cloned()
                .chain([ReasonCode::CommitteeOfficialBenchmarkBuilt])
                .collect(),
        };
        Ok(OfficialRunArtifacts {
            pack,
            linked_pack,
            benchmark_report: benchmark_bundle.benchmark_report,
            comparison,
            readiness,
            report,
        })
    }

    fn load_pack(
        &self,
        config: &CommitteeOfficialBenchmarkConfig,
    ) -> Result<OfficialCommitteeScenarioPack, String> {
        if let Some(path) = &config.outcome_linked_pack_path {
            if !config.run_materialization {
                return OutcomeLinkedCommitteeScenarioPack::from_json_path(Path::new(path))
                    .map(|linked_pack| linked_pack.pack);
            }
        }
        let pack_config = if let Some(path) = &config.scenario_pack_config_path {
            OfficialCommitteeScenarioPackConfig::from_toml_path(Path::new(path))?
        } else {
            OfficialCommitteeScenarioPackConfig {
                pack_id: format!("{}-pack", config.benchmark_id),
                output_root: config.output_root.clone(),
                ..OfficialCommitteeScenarioPackConfig::default()
            }
        };
        OfficialCommitteeScenarioPackBuilder::default().build(&pack_config)
    }

    fn load_linked_pack(
        &self,
        config: &CommitteeOfficialBenchmarkConfig,
        pack: &OfficialCommitteeScenarioPack,
    ) -> Result<OutcomeLinkedCommitteeScenarioPack, String> {
        if let Some(path) = &config.outcome_linked_pack_path {
            return OutcomeLinkedCommitteeScenarioPack::from_json_path(Path::new(path));
        }
        let linker_config = if let Some(path) = &config.outcome_linker_config_path {
            CommitteeOutcomeLinkerConfig::from_toml_path(Path::new(path))?
        } else {
            CommitteeOutcomeLinkerConfig {
                linker_id: format!("{}-linker", config.benchmark_id),
                output_root: config.output_root.clone(),
                ..CommitteeOutcomeLinkerConfig::default()
            }
        };
        CommitteeOutcomeLinker::default().link(pack, &linker_config)
    }

    fn run_committee_benchmark(
        &self,
        config: &CommitteeOfficialBenchmarkConfig,
        linked_pack: &OutcomeLinkedCommitteeScenarioPack,
    ) -> Result<super::committee_benchmark_bundle::CommitteeBenchmarkBundle, String> {
        let scenario_set =
            linked_pack.to_benchmark_scenario_set(&format!("{}-scenario-set", config.benchmark_id));
        let scenario_set_dir = config.output_dir().join("linked_scenario_set");
        let scenario_set_path = scenario_set.write_to_dir(&scenario_set_dir)?;
        let mut benchmark_config = if let Some(path) = &config.committee_benchmark_config_path {
            CommitteeBenchmarkConfig::from_toml_path(Path::new(path))?
        } else {
            CommitteeBenchmarkConfig::default()
        };
        benchmark_config.benchmark_id = config.benchmark_id.clone();
        benchmark_config.output_root = config.output_root.clone();
        benchmark_config.scenario_set_path = Some(scenario_set_path.to_string_lossy().to_string());
        benchmark_config.materialization_config_path = None;
        benchmark_config.committee_v1_config_path = None;
        benchmark_config.require_core_check = false;
        benchmark_config.run_vs_baseline_comparison = false;
        benchmark_config.run_actionability_report = config.run_actionability;
        benchmark_config.run_attribution_report = config.run_attribution;
        CommitteeBenchmarkRunner::default().run(&benchmark_config)
    }
}

impl CommitteeOfficialBenchmarkReport {
    pub fn to_text(&self) -> String {
        [
            format!("benchmark_id={}", self.benchmark_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={}", self.final_recommendation),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
            self.official_evidence_readiness_report.to_text(),
            self.outcome_linked_vs_baseline_report.to_text(),
        ]
        .join("\n")
    }
}

fn collect_blockers(
    config: &CommitteeOfficialBenchmarkConfig,
    linked_pack: &OutcomeLinkedCommitteeScenarioPack,
    readiness: &OfficialCommitteeEvidenceReadinessReport,
    comparison: &CommitteeOutcomeLinkedComparison,
) -> Vec<String> {
    let mut blockers = Vec::new();
    if readiness.official_row_count < config.min_official_rows {
        blockers.push("not enough official rows".to_string());
    }
    if config.require_outcome_linked_rows
        && linked_pack.outcome_linked_count < config.min_outcome_linked_rows
    {
        blockers.push("not enough outcome-linked rows".to_string());
    }
    if linked_pack.baseline_linked_count < config.min_baseline_linked_rows {
        blockers.push("not enough baseline references".to_string());
    }
    if linked_pack.no_trade_counterfactual_count < config.min_no_trade_counterfactuals {
        blockers.push("not enough no-trade counterfactuals".to_string());
    }
    if linked_pack.risk_denial_counterfactual_count < config.min_risk_denial_counterfactuals {
        blockers.push("not enough risk-denied counterfactuals".to_string());
    }
    if !readiness.no_lookahead_safe {
        blockers.push("no-lookahead violations block readiness".to_string());
    }
    if comparison.comparison_status == CommitteeOutcomeLinkedComparisonStatus::NoOutcomeReferences {
        blockers.push("no outcome references available for comparison".to_string());
    }
    blockers
}

fn collect_warnings(
    pack: &OfficialCommitteeScenarioPack,
    linked_pack: &OutcomeLinkedCommitteeScenarioPack,
    comparison: &CommitteeOutcomeLinkedComparison,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if pack.yfinance_row_count > 0 {
        warnings.push("research-only rows remain excluded from official readiness".to_string());
    }
    if pack.fixture_row_count > 0 {
        warnings.push("fixture rows remain diagnostic only".to_string());
    }
    if linked_pack.external_linked_count == 0 {
        warnings.push("no external prediction links were retained".to_string());
    }
    if matches!(
        comparison.comparison_status,
        CommitteeOutcomeLinkedComparisonStatus::DiagnosticOnly
            | CommitteeOutcomeLinkedComparisonStatus::NotEnoughOutcomeLinks
    ) {
        warnings.push("outcome-linked comparison remains diagnostic".to_string());
    }
    warnings
}

fn map_final_status(
    config: &CommitteeOfficialBenchmarkConfig,
    benchmark_report: &CommitteeBenchmarkReport,
    readiness: &OfficialCommitteeEvidenceReadinessReport,
    comparison: &CommitteeOutcomeLinkedComparison,
) -> CommitteeOfficialBenchmarkFinalStatus {
    if readiness.fixture_ratio >= 0.999 && benchmark_report.replay_report.record_count > 0 {
        CommitteeOfficialBenchmarkFinalStatus::FixtureOnly
    } else if readiness.research_only_ratio >= 0.999
        && readiness.official_row_count == 0
        && benchmark_report.replay_report.record_count > 0
    {
        CommitteeOfficialBenchmarkFinalStatus::ResearchOnly
    } else if readiness.crypto_only_ratio >= 0.999
        && (readiness.official_row_count > 0 || benchmark_report.replay_report.record_count > 0)
    {
        CommitteeOfficialBenchmarkFinalStatus::CryptoOnly
    } else if !readiness.no_lookahead_safe {
        CommitteeOfficialBenchmarkFinalStatus::CoreBlocked
    } else if readiness.readiness_status
        == OfficialCommitteeEvidenceReadinessStatus::NotReadySummaryDerivedDominant
    {
        CommitteeOfficialBenchmarkFinalStatus::MaterializationWeak
    } else if readiness.official_row_count < config.min_official_rows {
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOfficialRows
    } else if config.require_outcome_linked_rows
        && readiness.outcome_linked_row_count < config.min_outcome_linked_rows
    {
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOutcomeLinks
    } else if readiness.baseline_linked_row_count < config.min_baseline_linked_rows {
        CommitteeOfficialBenchmarkFinalStatus::NeedBetterBaselineReferences
    } else if readiness.no_trade_counterfactual_count < config.min_no_trade_counterfactuals {
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreNoTradeCounterfactuals
    } else if readiness.risk_denial_counterfactual_count < config.min_risk_denial_counterfactuals {
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreRiskDeniedCounterfactuals
    } else {
        match benchmark_report.final_status {
            CommitteeBenchmarkFinalStatus::RiskBlockedDominant => {
                CommitteeOfficialBenchmarkFinalStatus::RiskBlockedDominant
            }
            CommitteeBenchmarkFinalStatus::ChairNeedsTuning => {
                CommitteeOfficialBenchmarkFinalStatus::ImproveChairFirst
            }
            CommitteeBenchmarkFinalStatus::PersonaScoringNeedsTuning => {
                CommitteeOfficialBenchmarkFinalStatus::ImprovePersonaScoringFirst
            }
            CommitteeBenchmarkFinalStatus::ImproveRiskGovernorFirst => {
                CommitteeOfficialBenchmarkFinalStatus::ImproveRiskGovernorFirst
            }
            _ if readiness.enough_for_committee_benchmark
                && comparison.comparison_status
                    == CommitteeOutcomeLinkedComparisonStatus::Comparable =>
            {
                CommitteeOfficialBenchmarkFinalStatus::OfficialCommitteeBenchmarkReady
            }
            _ => CommitteeOfficialBenchmarkFinalStatus::NeedMoreEvidence,
        }
    }
}

fn recommendation_for_status(status: CommitteeOfficialBenchmarkFinalStatus) -> &'static str {
    match status {
        CommitteeOfficialBenchmarkFinalStatus::OfficialCommitteeBenchmarkReady => {
            "CommitteeV1BenchmarkReady"
        }
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOfficialRows => {
            "MoreOfficialCommitteeEvidence"
        }
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreOutcomeLinks => "ImproveOutcomeLinkingFirst",
        CommitteeOfficialBenchmarkFinalStatus::NeedBetterBaselineReferences => {
            "NeedBetterBaselineReferences"
        }
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreNoTradeCounterfactuals => {
            "NeedMoreNoTradeCounterfactuals"
        }
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreRiskDeniedCounterfactuals => {
            "NeedMoreRiskDeniedCounterfactuals"
        }
        CommitteeOfficialBenchmarkFinalStatus::ResearchOnly => "NeedMoreEvidence",
        CommitteeOfficialBenchmarkFinalStatus::FixtureOnly => "NeedMoreEvidence",
        CommitteeOfficialBenchmarkFinalStatus::CryptoOnly => "KeepTrinity",
        CommitteeOfficialBenchmarkFinalStatus::MaterializationWeak => {
            "ImproveScenarioMaterializationFirst"
        }
        CommitteeOfficialBenchmarkFinalStatus::CoreBlocked => "NeedMoreEvidence",
        CommitteeOfficialBenchmarkFinalStatus::RiskBlockedDominant => "ImproveRiskGovernorFirst",
        CommitteeOfficialBenchmarkFinalStatus::ImproveChairFirst => "ImproveChairFirst",
        CommitteeOfficialBenchmarkFinalStatus::ImprovePersonaScoringFirst => {
            "ImprovePersonaScoringFirst"
        }
        CommitteeOfficialBenchmarkFinalStatus::ImproveRiskGovernorFirst => {
            "ImproveRiskGovernorFirst"
        }
        CommitteeOfficialBenchmarkFinalStatus::NeedMoreEvidence => "NeedMoreEvidence",
    }
}

fn default_true() -> bool {
    true
}

fn default_min_official_rows() -> usize {
    5
}

fn default_min_outcome_linked_rows() -> usize {
    3
}

fn default_min_baseline_linked_rows() -> usize {
    3
}

fn default_min_counterfactuals() -> usize {
    1
}

fn default_max_summary_derived_ratio() -> f64 {
    0.40
}

fn default_max_research_only_ratio() -> f64 {
    0.10
}

fn default_max_fixture_ratio() -> f64 {
    0.05
}
