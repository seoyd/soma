use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};

use super::chair_calibration::ChairCalibrationReport;
use super::chair_diagnostics::ChairDiagnosticsReport;
use super::committee_decision_quality::CommitteeDecisionQualityReport;
use super::committee_evidence_quality::CommitteeEvidenceQualityReport;
use super::committee_replay::CommitteeReplayReport;
use super::committee_scenario_loader::CommitteeScenarioSet;
use super::committee_v1_readiness::{CommitteeV1NextRecommendation, CommitteeV1ReadinessReport};
use super::persona_conflict_matrix::PersonaConflictMatrix;
use super::risk_bridge_diagnostics::RiskBridgeDiagnosticsReport;
use super::risk_calibration::RiskCalibrationReport;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitteeV1FinalStatus {
    CommitteeV1ResearchReady,
    CommitteeV1NeedsEvidence,
    CommitteeV1NeedsChairTuning,
    CommitteeV1NeedsRiskReview,
    CommitteeV1ResearchOnly,
    CommitteeV1FixtureOnly,
    CommitteeV1Blocked,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChairDiagnosticsSummary {
    pub report_count: usize,
    pub status_counts: BTreeMap<String, usize>,
    pub warnings: Vec<String>,
    pub reports: Vec<ChairDiagnosticsReport>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskDiagnosticsSummary {
    pub report_count: usize,
    pub status_counts: BTreeMap<String, usize>,
    pub veto_count: usize,
    pub warnings: Vec<String>,
    pub reports: Vec<RiskBridgeDiagnosticsReport>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeV1ReportBundle {
    pub run_id: String,
    #[serde(default)]
    pub scenario_set: Option<CommitteeScenarioSet>,
    pub replay_report: CommitteeReplayReport,
    pub chair_diagnostics_summary: ChairDiagnosticsSummary,
    pub risk_diagnostics_summary: RiskDiagnosticsSummary,
    pub conflict_matrix: PersonaConflictMatrix,
    pub evidence_quality_report: CommitteeEvidenceQualityReport,
    pub decision_quality_report: CommitteeDecisionQualityReport,
    pub chair_calibration_report: ChairCalibrationReport,
    pub risk_calibration_report: RiskCalibrationReport,
    pub v1_readiness_report: CommitteeV1ReadinessReport,
    pub audit_summary: String,
    #[serde(default)]
    pub storage_summary: Option<String>,
    pub final_status: CommitteeV1FinalStatus,
    pub final_recommendation: CommitteeV1NextRecommendation,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl ChairDiagnosticsSummary {
    pub fn from_reports(reports: Vec<ChairDiagnosticsReport>) -> Self {
        let mut status_counts = BTreeMap::new();
        let mut warnings = Vec::new();
        for report in &reports {
            *status_counts
                .entry(format!("{:?}", report.diagnostic_status))
                .or_insert(0) += 1;
            if report.cluster_penalty_applied {
                warnings.push("cluster penalty applied in replay".to_string());
            }
            if report.contrarian_included {
                warnings.push("contrarian protection triggered in replay".to_string());
            }
        }
        warnings.sort();
        warnings.dedup();
        Self {
            report_count: reports.len(),
            status_counts,
            warnings,
            reports,
            reason_codes: vec![ReasonCode::CommitteeV1BundleBuilt],
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("report_count={}", self.report_count),
            format!("warnings={}", self.warnings.join("|")),
        ];
        for (status, count) in &self.status_counts {
            lines.push(format!("chair_status={status};count={count}"));
        }
        lines
            .into_iter()
            .chain(self.reports.iter().map(|report| report.to_text()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl RiskDiagnosticsSummary {
    pub fn from_reports(reports: Vec<RiskBridgeDiagnosticsReport>) -> Self {
        let mut status_counts = BTreeMap::new();
        let mut warnings = Vec::new();
        let veto_count = reports.iter().filter(|report| report.veto_applied).count();
        for report in &reports {
            *status_counts
                .entry(format!("{:?}", report.diagnostic_status))
                .or_insert(0) += 1;
            if report.veto_applied {
                warnings.push("risk governor vetoed at least one committee candidate".to_string());
            }
        }
        warnings.sort();
        warnings.dedup();
        Self {
            report_count: reports.len(),
            status_counts,
            veto_count,
            warnings,
            reports,
            reason_codes: vec![ReasonCode::CommitteeV1BundleBuilt],
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("report_count={}", self.report_count),
            format!("veto_count={}", self.veto_count),
            format!("warnings={}", self.warnings.join("|")),
        ];
        for (status, count) in &self.status_counts {
            lines.push(format!("risk_status={status};count={count}"));
        }
        lines
            .into_iter()
            .chain(self.reports.iter().map(|report| report.to_text()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl CommitteeV1ReportBundle {
    pub fn build_audit_summary(&self) -> String {
        stable_hash_string(&format!(
            "{}|{}|{:?}|{:?}|{}",
            self.run_id,
            self.replay_report.deterministic_fingerprint,
            self.final_status,
            self.final_recommendation,
            self.warnings.join("|")
        ))
    }

    pub fn to_text(&self) -> String {
        [
            format!("run_id={}", self.run_id),
            format!("final_status={:?}", self.final_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!("audit_summary={}", self.audit_summary),
            format!("warnings={}", self.warnings.join("|")),
            self.replay_report.to_text(),
            self.evidence_quality_report.to_text(),
            self.decision_quality_report.to_text(),
            self.chair_calibration_report.to_text(),
            self.risk_calibration_report.to_text(),
            self.v1_readiness_report.to_text(),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        if let Some(scenario_set) = &self.scenario_set {
            fs::write(output_dir.join("scenario_set.txt"), scenario_set.to_text())
                .map_err(|err| err.to_string())?;
            fs::write(
                output_dir.join("scenario_set.json"),
                scenario_set.to_json_string()?,
            )
            .map_err(|err| err.to_string())?;
        }
        fs::write(
            output_dir.join("replay_report.txt"),
            self.replay_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("chair_diagnostics.txt"),
            self.chair_diagnostics_summary.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("risk_diagnostics.txt"),
            self.risk_diagnostics_summary.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("conflict_matrix.txt"),
            self.conflict_matrix.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("evidence_quality.txt"),
            self.evidence_quality_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("decision_quality.txt"),
            self.decision_quality_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("chair_calibration.txt"),
            self.chair_calibration_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("risk_calibration.txt"),
            self.risk_calibration_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_v1_readiness.txt"),
            self.v1_readiness_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(output_dir.join("committee_v1_summary.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        let bundle_path = output_dir.join("committee_v1_bundle.json");
        fs::write(
            &bundle_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(bundle_path)
    }
}
