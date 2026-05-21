use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::chair_diagnostics::{ChairDiagnosticsReport, build_chair_diagnostics};
use super::committee_decision::CommitteeInput;
use super::committee_evaluation::{
    CommitteeEvaluationScaffold, build_committee_evaluation_scaffold,
};
use super::committee_evidence_quality::{
    CommitteeEvidenceQualityReport, build_committee_evidence_quality_report,
};
use super::committee_replay::{
    CommitteeDebateReplay, CommitteeReplayConfig, CommitteeReplayReport,
};
use super::committee_risk_bridge::CommitteeRiskBridge;
use super::committee_scenario_loader::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioSet,
};
use super::committee_smoke::CommitteeSmokeTestConfig;
use super::persona_conflict_matrix::{PersonaConflictMatrix, build_persona_conflict_matrix};
use super::risk_bridge_diagnostics::{RiskBridgeDiagnosticsReport, build_risk_bridge_diagnostics};
use super::six_persona_readiness::{
    SixPersonaDesignReadinessConfig, SixPersonaDesignReadinessReport,
    evaluate_six_persona_design_readiness,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDiagnosticsConfig {
    pub diagnostic_id: String,
    #[serde(default)]
    pub scenario_load_config_path: Option<String>,
    #[serde(default)]
    pub replay_config_path: Option<String>,
    #[serde(default)]
    pub committee_smoke_config_path: Option<String>,
    pub output_root: String,
    #[serde(default)]
    pub six_persona_readiness_config: Option<SixPersonaDesignReadinessConfig>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDiagnosticsStatus {
    DiagnosticsHealthy,
    EvidenceTooWeak,
    ChairNeedsTuning,
    RiskNeedsReview,
    PersonaScoringNeedsTuning,
    GroupthinkRisk,
    TooMuchDisagreement,
    ResearchOnly,
    NeedMoreSamples,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeDiagnosticsRecommendation {
    KeepTrinity,
    ImproveEvidenceIngestionFirst,
    ImproveChairFirst,
    ImprovePersonaScoringFirst,
    ImproveRiskGovernorFirst,
    SixPersonaDesignReviewOnly,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDiagnosticsAggregate {
    pub replay_report: CommitteeReplayReport,
    pub chair_diagnostics: Vec<ChairDiagnosticsReport>,
    pub risk_diagnostics: Vec<RiskBridgeDiagnosticsReport>,
    pub conflict_matrix: PersonaConflictMatrix,
    pub evidence_quality_report: CommitteeEvidenceQualityReport,
    pub evaluation_scaffold: CommitteeEvaluationScaffold,
    pub final_status: CommitteeDiagnosticsStatus,
    pub recommendation: CommitteeDiagnosticsRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeDiagnosticsBundle {
    pub diagnostics: CommitteeDiagnosticsAggregate,
    pub six_persona_readiness: SixPersonaDesignReadinessReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeDiagnosticsRunner;

impl Default for CommitteeDiagnosticsConfig {
    fn default() -> Self {
        Self {
            diagnostic_id: "committee_diagnostics".to_string(),
            scenario_load_config_path: None,
            replay_config_path: None,
            committee_smoke_config_path: None,
            output_root: "target/soma_committee_diagnostics".to_string(),
            six_persona_readiness_config: None,
            reason_codes: vec![ReasonCode::CommitteeDiagnosticsBuilt],
        }
    }
}

impl CommitteeDiagnosticsConfig {
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

    pub fn validate_local_paths(&self) -> Result<(), String> {
        if self.output_root.contains("://")
            || self
                .scenario_load_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .replay_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .committee_smoke_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
        {
            return Err("committee diagnostics paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.diagnostic_id)
    }
}

impl CommitteeDiagnosticsAggregate {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("final_status={:?}", self.final_status),
            format!("recommendation={:?}", self.recommendation),
            self.replay_report.to_text(),
            self.evidence_quality_report.to_text(),
            self.conflict_matrix.to_text(),
        ]
        .join("\n")
    }
}

impl CommitteeDiagnosticsBundle {
    pub fn to_text(&self) -> String {
        [
            self.diagnostics.to_text(),
            self.six_persona_readiness.to_text(),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("committee_diagnostics_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_diagnostics_bundle.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl CommitteeDiagnosticsRunner {
    pub fn run(
        &self,
        config: &CommitteeDiagnosticsConfig,
    ) -> Result<CommitteeDiagnosticsBundle, String> {
        config.validate_local_paths()?;
        let (scenario_set, replay_report) = load_replay_and_scenarios(config)?;
        let bridge = CommitteeRiskBridge::default();
        let chair_diagnostics = replay_report
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
                        reason_codes: vec![ReasonCode::CommitteeDiagnosticsBuilt],
                    },
                    &record.chair_decision_record,
                )
            })
            .collect::<Vec<_>>();
        let risk_diagnostics = replay_report
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
        let conflict_matrix = build_persona_conflict_matrix(&replay_report);
        let evidence_quality_report = build_committee_evidence_quality_report(&scenario_set);
        let outcomes = replay_report
            .records
            .iter()
            .map(|record| record.risk_bridge_outcome.clone())
            .collect::<Vec<_>>();
        let evaluation_scaffold = build_committee_evaluation_scaffold(&outcomes);
        let final_status = if evidence_quality_report.quality_status
            == super::committee_evidence_quality::CommitteeEvidenceQualityStatus::ResearchOnlyEvidence
        {
            CommitteeDiagnosticsStatus::ResearchOnly
        } else if matches!(
            evidence_quality_report.quality_status,
            super::committee_evidence_quality::CommitteeEvidenceQualityStatus::FixtureOnlyEvidence
                | super::committee_evidence_quality::CommitteeEvidenceQualityStatus::InsufficientEvidence
                | super::committee_evidence_quality::CommitteeEvidenceQualityStatus::LowQualityEvidence
        ) {
            CommitteeDiagnosticsStatus::EvidenceTooWeak
        } else if replay_report.record_count < 3 {
            CommitteeDiagnosticsStatus::NeedMoreSamples
        } else if conflict_matrix.conflict_status == super::persona_conflict_matrix::PersonaConflictStatus::TooAligned
        {
            CommitteeDiagnosticsStatus::GroupthinkRisk
        } else if conflict_matrix.conflict_status
            == super::persona_conflict_matrix::PersonaConflictStatus::TooConflicted
        {
            CommitteeDiagnosticsStatus::TooMuchDisagreement
        } else if risk_diagnostics.iter().all(|report| report.veto_applied) {
            CommitteeDiagnosticsStatus::RiskNeedsReview
        } else if chair_diagnostics.iter().any(|report| {
            matches!(
                report.diagnostic_status,
                super::chair_diagnostics::ChairDiagnosticStatus::OverFiltered
                    | super::chair_diagnostics::ChairDiagnosticStatus::TooFewSpeakers
            )
        }) {
            CommitteeDiagnosticsStatus::ChairNeedsTuning
        } else if evaluation_scaffold.recommendation
            == super::committee_evaluation::CommitteeEvaluationRecommendation::ImprovePersonaThresholds
        {
            CommitteeDiagnosticsStatus::PersonaScoringNeedsTuning
        } else {
            CommitteeDiagnosticsStatus::DiagnosticsHealthy
        };
        let recommendation = match final_status {
            CommitteeDiagnosticsStatus::DiagnosticsHealthy => {
                if evidence_quality_report.enough_for_design_review {
                    CommitteeDiagnosticsRecommendation::SixPersonaDesignReviewOnly
                } else {
                    CommitteeDiagnosticsRecommendation::KeepTrinity
                }
            }
            CommitteeDiagnosticsStatus::EvidenceTooWeak
            | CommitteeDiagnosticsStatus::ResearchOnly => {
                CommitteeDiagnosticsRecommendation::ImproveEvidenceIngestionFirst
            }
            CommitteeDiagnosticsStatus::ChairNeedsTuning
            | CommitteeDiagnosticsStatus::GroupthinkRisk => {
                CommitteeDiagnosticsRecommendation::ImproveChairFirst
            }
            CommitteeDiagnosticsStatus::RiskNeedsReview => {
                CommitteeDiagnosticsRecommendation::ImproveRiskGovernorFirst
            }
            CommitteeDiagnosticsStatus::PersonaScoringNeedsTuning
            | CommitteeDiagnosticsStatus::TooMuchDisagreement => {
                CommitteeDiagnosticsRecommendation::ImprovePersonaScoringFirst
            }
            CommitteeDiagnosticsStatus::NeedMoreSamples => {
                CommitteeDiagnosticsRecommendation::NeedMoreEvidence
            }
        };
        let diagnostics = CommitteeDiagnosticsAggregate {
            replay_report,
            chair_diagnostics,
            risk_diagnostics,
            conflict_matrix,
            evidence_quality_report,
            evaluation_scaffold,
            final_status,
            recommendation,
            reason_codes: vec![ReasonCode::CommitteeDiagnosticsBuilt],
        };
        let six_persona_readiness = evaluate_six_persona_design_readiness(
            &diagnostics,
            &config
                .six_persona_readiness_config
                .clone()
                .unwrap_or_default(),
        );
        Ok(CommitteeDiagnosticsBundle {
            diagnostics,
            six_persona_readiness,
        })
    }
}

fn load_replay_and_scenarios(
    config: &CommitteeDiagnosticsConfig,
) -> Result<(CommitteeScenarioSet, CommitteeReplayReport), String> {
    if let Some(path) = &config.replay_config_path {
        let replay_config = CommitteeReplayConfig::from_toml_path(Path::new(path))?;
        let replay_report = CommitteeDebateReplay::default().run(&replay_config)?;
        let scenario_set = if let Some(path) = replay_config.scenario_set_path {
            CommitteeScenarioSet::from_json_path(Path::new(&path))?
        } else if let Some(path) = replay_config.committee_smoke_config_path {
            let smoke = CommitteeSmokeTestConfig::from_toml_path(Path::new(&path))?;
            CommitteeScenarioLoader::default().load(
                &CommitteeScenarioLoadConfig::from_committee_smoke_config(&smoke),
            )?
        } else {
            return Err("committee replay config missing scenario source".to_string());
        };
        Ok((scenario_set, replay_report))
    } else if let Some(path) = &config.scenario_load_config_path {
        let load_config = CommitteeScenarioLoadConfig::from_toml_path(Path::new(path))?;
        let scenario_set = CommitteeScenarioLoader::default().load(&load_config)?;
        let replay_report = CommitteeDebateReplay::default().run(&CommitteeReplayConfig {
            replay_id: format!("{}-replay", config.diagnostic_id),
            scenario_set_path: Some(
                scenario_set
                    .write_to_dir(&config.output_dir())?
                    .display()
                    .to_string(),
            ),
            output_root: config.output_root.clone(),
            reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
            ..CommitteeReplayConfig::default()
        })?;
        Ok((scenario_set, replay_report))
    } else if let Some(path) = &config.committee_smoke_config_path {
        let smoke = CommitteeSmokeTestConfig::from_toml_path(Path::new(path))?;
        let scenario_set = CommitteeScenarioLoader::default().load(
            &CommitteeScenarioLoadConfig::from_committee_smoke_config(&smoke),
        )?;
        let replay_report = CommitteeDebateReplay::default().run(&CommitteeReplayConfig {
            replay_id: format!("{}-replay", config.diagnostic_id),
            scenario_set_path: Some(
                scenario_set
                    .write_to_dir(&config.output_dir())?
                    .display()
                    .to_string(),
            ),
            output_root: config.output_root.clone(),
            reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
            ..CommitteeReplayConfig::default()
        })?;
        Ok((scenario_set, replay_report))
    } else {
        Err(
            "committee diagnostics requires replay, scenario-load, or committee-smoke config"
                .to_string(),
        )
    }
}
