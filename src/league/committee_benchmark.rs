use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{CoreCheckConfig, CoreCheckRunner, ReasonCode};

use super::chair_diagnostics::build_chair_diagnostics;
use super::committee_actionability::{
    CommitteeActionabilityReport, build_committee_actionability_report,
};
use super::committee_attribution::{
    CommitteeAttributionReport, build_committee_attribution_report,
};
use super::committee_benchmark_bundle::{
    CommitteeBenchmarkBundle, CommitteeBenchmarkDiagnosticsSummary,
};
use super::committee_benchmark_readiness::{
    CommitteeBenchmarkNextRecommendation, CommitteeBenchmarkReadinessReport,
    build_committee_benchmark_readiness_report,
};
use super::committee_decision::CommitteeInput;
use super::committee_decision_quality::build_committee_decision_quality_report;
use super::committee_materialization::{
    CommitteeMaterializationConfig, CommitteeScenarioMaterializerV2,
};
use super::committee_replay::{CommitteeDebateReplay, CommitteeReplayReport};
use super::committee_risk_bridge::CommitteeRiskBridge;
use super::committee_scenario_loader::CommitteeScenarioSet;
use super::committee_v1::CommitteeV1RunConfig;
use super::committee_v1_bundle::{ChairDiagnosticsSummary, RiskDiagnosticsSummary};
use super::committee_vs_baseline::{
    CommitteeVsBaselineComparison, CommitteeVsBaselineStatus,
    build_committee_vs_baseline_comparison,
};
use super::risk_bridge_diagnostics::build_risk_bridge_diagnostics;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeBenchmarkConfig {
    pub benchmark_id: String,
    #[serde(default)]
    pub materialization_config_path: Option<String>,
    #[serde(default)]
    pub scenario_set_path: Option<String>,
    #[serde(default)]
    pub committee_v1_config_path: Option<String>,
    pub output_root: String,
    #[serde(default = "default_max_decisions")]
    pub max_decisions: usize,
    #[serde(default = "default_true")]
    pub require_core_check: bool,
    #[serde(default = "default_true")]
    pub run_replay: bool,
    #[serde(default = "default_true")]
    pub run_chair_diagnostics: bool,
    #[serde(default = "default_true")]
    pub run_risk_diagnostics: bool,
    #[serde(default = "default_true")]
    pub run_vs_baseline_comparison: bool,
    #[serde(default = "default_true")]
    pub run_actionability_report: bool,
    #[serde(default = "default_true")]
    pub run_attribution_report: bool,
    #[serde(default = "default_true")]
    pub run_readiness_gate: bool,
    #[serde(default = "default_true")]
    pub allow_fixture: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_min_official_rows")]
    pub min_official_rows: usize,
    #[serde(default = "default_min_total_rows")]
    pub min_total_rows: usize,
    #[serde(default = "default_min_outcome_references")]
    pub min_outcome_references: usize,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeBenchmarkFinalStatus {
    CommitteeBenchmarkReady,
    NeedMoreOfficialScenarios,
    NeedBetterMaterialization,
    ResearchOnlyBenchmark,
    FixtureOnlyBenchmark,
    CryptoOnlyBenchmark,
    RiskBlockedDominant,
    ChairNeedsTuning,
    PersonaScoringNeedsTuning,
    ImproveRiskGovernorFirst,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeBenchmarkReport {
    pub benchmark_id: String,
    pub scenario_summary: String,
    pub replay_report: CommitteeReplayReport,
    pub chair_diagnostics_summary: ChairDiagnosticsSummary,
    pub risk_diagnostics_summary: RiskDiagnosticsSummary,
    pub decision_quality_report: super::committee_decision_quality::CommitteeDecisionQualityReport,
    #[serde(default)]
    pub vs_baseline_report: Option<CommitteeVsBaselineComparison>,
    pub actionability_report: CommitteeActionabilityReport,
    pub attribution_report: CommitteeAttributionReport,
    pub benchmark_readiness_report: CommitteeBenchmarkReadinessReport,
    pub final_status: CommitteeBenchmarkFinalStatus,
    pub final_recommendation: CommitteeBenchmarkNextRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeBenchmarkRunner;

impl Default for CommitteeBenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_id: "committee_benchmark".to_string(),
            materialization_config_path: None,
            scenario_set_path: None,
            committee_v1_config_path: None,
            output_root: "target/soma_committee_benchmark".to_string(),
            max_decisions: default_max_decisions(),
            require_core_check: true,
            run_replay: true,
            run_chair_diagnostics: true,
            run_risk_diagnostics: true,
            run_vs_baseline_comparison: true,
            run_actionability_report: true,
            run_attribution_report: true,
            run_readiness_gate: true,
            allow_fixture: true,
            allow_yfinance_research: true,
            allow_crypto_only: true,
            min_official_rows: default_min_official_rows(),
            min_total_rows: default_min_total_rows(),
            min_outcome_references: default_min_outcome_references(),
            reason_codes: vec![ReasonCode::CommitteeBenchmarkBuilt],
        }
    }
}

impl CommitteeBenchmarkConfig {
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
        if self.output_root.contains("://")
            || self
                .materialization_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .scenario_set_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .committee_v1_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
        {
            return Err("committee benchmark paths must be local".to_string());
        }
        if self.max_decisions == 0 || self.max_decisions > default_max_decisions() {
            return Err("committee benchmark max_decisions must be between 1 and 50".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.benchmark_id)
    }
}

impl CommitteeBenchmarkRunner {
    pub fn run(
        &self,
        config: &CommitteeBenchmarkConfig,
    ) -> Result<CommitteeBenchmarkBundle, String> {
        config.validate()?;
        let materialization_config = if let Some(path) = &config.materialization_config_path {
            Some(CommitteeMaterializationConfig::from_toml_path(Path::new(
                path,
            ))?)
        } else {
            None
        };
        let scenario_set = load_scenario_set(config, materialization_config.as_ref())?;
        if config.require_core_check && config.run_replay {
            let _ = CoreCheckRunner::default().run(&CoreCheckConfig::default())?;
        }
        let replay_report = CommitteeDebateReplay::default().run_for_scenario_set(
            &format!("{}-replay", config.benchmark_id),
            &scenario_set,
            config.max_decisions,
        )?;
        let bridge = CommitteeRiskBridge::default();
        let chair_reports = replay_report
            .records
            .iter()
            .map(|record| {
                build_chair_diagnostics(
                    &CommitteeInput {
                        scoring_input: record.scenario_row.to_scoring_input(),
                        persona_votes: record.persona_votes.clone(),
                        target_horizon: record.scenario_row.target_horizon,
                        source_kind: record.scenario_row.evidence_source_kind,
                        regime: record.scenario_row.regime,
                        reason_codes: vec![ReasonCode::CommitteeBenchmarkBuilt],
                    },
                    &record.chair_decision_record,
                )
            })
            .collect::<Vec<_>>();
        let risk_reports = replay_report
            .records
            .iter()
            .map(|record| {
                build_risk_bridge_diagnostics(
                    &bridge,
                    &record.scenario_row.to_market_snapshot(),
                    &record.scenario_row.to_scoring_input(),
                    &record.chair_decision_record,
                    &record.risk_bridge_outcome,
                )
            })
            .collect::<Vec<_>>();
        let chair_diagnostics_summary = ChairDiagnosticsSummary::from_reports(chair_reports);
        let risk_diagnostics_summary = RiskDiagnosticsSummary::from_reports(risk_reports);
        let conflict_matrix =
            super::persona_conflict_matrix::build_persona_conflict_matrix(&replay_report);
        let evidence_quality_report =
            super::committee_evidence_quality::build_committee_evidence_quality_report(
                &scenario_set,
            );
        let decision_quality_report = build_committee_decision_quality_report(
            &replay_report,
            &chair_diagnostics_summary.reports,
            &risk_diagnostics_summary.reports,
            &conflict_matrix,
            &evidence_quality_report,
        );
        let vs_baseline_report = config
            .run_vs_baseline_comparison
            .then(|| build_committee_vs_baseline_comparison(&scenario_set, &replay_report));
        let actionability_report =
            build_committee_actionability_report(&scenario_set, &replay_report);
        let attribution_report = build_committee_attribution_report(&replay_report);
        let readiness_report = build_committee_benchmark_readiness_report(
            &scenario_set,
            materialization_config.as_ref(),
            &decision_quality_report,
            &actionability_report,
            &attribution_report,
            config.min_official_rows,
            config.min_total_rows,
            config.min_outcome_references,
        );
        let final_status = map_final_status(
            &readiness_report,
            vs_baseline_report.as_ref(),
            &actionability_report,
        );
        let benchmark_report = CommitteeBenchmarkReport {
            benchmark_id: config.benchmark_id.clone(),
            scenario_summary: scenario_set.source_summary.clone(),
            replay_report: replay_report.clone(),
            chair_diagnostics_summary: chair_diagnostics_summary.clone(),
            risk_diagnostics_summary: risk_diagnostics_summary.clone(),
            decision_quality_report: decision_quality_report.clone(),
            vs_baseline_report: vs_baseline_report.clone(),
            actionability_report: actionability_report.clone(),
            attribution_report: attribution_report.clone(),
            benchmark_readiness_report: readiness_report.clone(),
            final_status,
            final_recommendation: readiness_report.next_recommendation,
            blockers: readiness_report.blockers.clone(),
            warnings: readiness_report.warnings.clone(),
            reason_codes: vec![ReasonCode::CommitteeBenchmarkBuilt],
        };
        let diagnostics_summary = CommitteeBenchmarkDiagnosticsSummary {
            chair: chair_diagnostics_summary,
            risk: risk_diagnostics_summary,
            decision_quality: decision_quality_report,
        };
        let mut bundle = CommitteeBenchmarkBundle {
            benchmark_report,
            materialized_scenario_set: scenario_set,
            replay_report,
            diagnostics_summary,
            vs_baseline_report,
            actionability_report,
            attribution_report,
            readiness_report,
            audit_summary: String::new(),
            storage_summary: Some(format!("output_dir={}", config.output_dir().display())),
            final_summary: String::new(),
            reason_codes: vec![ReasonCode::CommitteeBenchmarkBundleBuilt],
        };
        bundle.final_summary = bundle.benchmark_report.to_text();
        bundle.audit_summary = bundle.build_audit_summary();
        Ok(bundle)
    }
}

impl CommitteeBenchmarkReport {
    pub fn to_text(&self) -> String {
        [
            format!("benchmark_id={}", self.benchmark_id),
            format!("scenario_summary={}", self.scenario_summary),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("blockers={}", self.blockers.join("|")),
            format!("warnings={}", self.warnings.join("|")),
            self.replay_report.to_text(),
            self.benchmark_readiness_report.to_text(),
        ]
        .join("\n")
    }
}

fn load_scenario_set(
    config: &CommitteeBenchmarkConfig,
    materialization_config: Option<&CommitteeMaterializationConfig>,
) -> Result<CommitteeScenarioSet, String> {
    if let Some(path) = &config.scenario_set_path {
        CommitteeScenarioSet::from_json_path(Path::new(path))
    } else if let Some(config) = materialization_config {
        CommitteeScenarioMaterializerV2::default().materialize(config)
    } else if let Some(path) = &config.committee_v1_config_path {
        let v1 = super::committee_v1_runner::CommitteeV1Runner::default()
            .run(&CommitteeV1RunConfig::from_toml_path(Path::new(path))?)?;
        v1.scenario_set
            .ok_or_else(|| "committee v1 config did not produce scenario set".to_string())
    } else {
        Err("committee benchmark requires materialization_config_path, scenario_set_path, or committee_v1_config_path".to_string())
    }
}

fn map_final_status(
    readiness_report: &CommitteeBenchmarkReadinessReport,
    vs_baseline_report: Option<&CommitteeVsBaselineComparison>,
    actionability_report: &CommitteeActionabilityReport,
) -> CommitteeBenchmarkFinalStatus {
    match readiness_report.status {
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::ReadyForCommitteeBenchmark => {
            CommitteeBenchmarkFinalStatus::CommitteeBenchmarkReady
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::ReadyForMoreOfficialEvidence => {
            CommitteeBenchmarkFinalStatus::NeedMoreOfficialScenarios
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::ReadyForChairTuning
        | super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyGroupthink => {
            CommitteeBenchmarkFinalStatus::ChairNeedsTuning
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::ReadyForPersonaScoringTuning => {
            CommitteeBenchmarkFinalStatus::PersonaScoringNeedsTuning
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::ReadyForRiskReview
        | super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyRiskBlockedDominant => {
            CommitteeBenchmarkFinalStatus::ImproveRiskGovernorFirst
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyFixtureOnly => {
            CommitteeBenchmarkFinalStatus::FixtureOnlyBenchmark
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyResearchOnly => {
            CommitteeBenchmarkFinalStatus::ResearchOnlyBenchmark
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyCryptoOnly => {
            CommitteeBenchmarkFinalStatus::CryptoOnlyBenchmark
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyMaterializationWeak => {
            CommitteeBenchmarkFinalStatus::NeedBetterMaterialization
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::ReadyForSixPersonaDesignReviewOnly => {
            CommitteeBenchmarkFinalStatus::NeedMoreOfficialScenarios
        }
        super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyInsufficientRows
        | super::committee_benchmark_readiness::CommitteeBenchmarkReadinessStatus::NotReadyInsufficientOutcomes => {
            if actionability_report.actionability_status == super::committee_actionability::CommitteeActionabilityStatus::MostlyRiskDenied
                || vs_baseline_report
                    .is_some_and(|report| report.comparison_status == CommitteeVsBaselineStatus::NoOutcomeReference)
            {
                CommitteeBenchmarkFinalStatus::NeedMoreEvidence
            } else {
                CommitteeBenchmarkFinalStatus::NeedMoreOfficialScenarios
            }
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_max_decisions() -> usize {
    50
}

fn default_min_official_rows() -> usize {
    5
}

fn default_min_total_rows() -> usize {
    5
}

fn default_min_outcome_references() -> usize {
    3
}
