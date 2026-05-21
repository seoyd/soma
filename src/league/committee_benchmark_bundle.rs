use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};

use super::committee_actionability::CommitteeActionabilityReport;
use super::committee_attribution::CommitteeAttributionReport;
use super::committee_benchmark::CommitteeBenchmarkReport;
use super::committee_benchmark_readiness::CommitteeBenchmarkReadinessReport;
use super::committee_decision_quality::CommitteeDecisionQualityReport;
use super::committee_replay::CommitteeReplayReport;
use super::committee_scenario_loader::CommitteeScenarioSet;
use super::committee_v1_bundle::{ChairDiagnosticsSummary, RiskDiagnosticsSummary};
use super::committee_vs_baseline::CommitteeVsBaselineComparison;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeBenchmarkDiagnosticsSummary {
    pub chair: ChairDiagnosticsSummary,
    pub risk: RiskDiagnosticsSummary,
    pub decision_quality: CommitteeDecisionQualityReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeBenchmarkBundle {
    pub benchmark_report: CommitteeBenchmarkReport,
    pub materialized_scenario_set: CommitteeScenarioSet,
    pub replay_report: CommitteeReplayReport,
    pub diagnostics_summary: CommitteeBenchmarkDiagnosticsSummary,
    #[serde(default)]
    pub vs_baseline_report: Option<CommitteeVsBaselineComparison>,
    pub actionability_report: CommitteeActionabilityReport,
    pub attribution_report: CommitteeAttributionReport,
    pub readiness_report: CommitteeBenchmarkReadinessReport,
    pub audit_summary: String,
    #[serde(default)]
    pub storage_summary: Option<String>,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeBenchmarkBundle {
    pub fn to_text(&self) -> String {
        [
            self.benchmark_report.to_text(),
            self.diagnostics_summary.chair.to_text(),
            self.diagnostics_summary.risk.to_text(),
            self.diagnostics_summary.decision_quality.to_text(),
            self.actionability_report.to_text(),
            self.attribution_report.to_text(),
            self.readiness_report.to_text(),
            self.vs_baseline_report
                .as_ref()
                .map(|report| report.to_text())
                .unwrap_or_else(|| "vs_baseline=none".to_string()),
            format!("audit_summary={}", self.audit_summary),
            format!("final_summary={}", self.final_summary),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("materialized_scenarios.txt"),
            self.materialized_scenario_set.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("materialized_scenarios.json"),
            self.materialized_scenario_set.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("replay_report.txt"),
            self.replay_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("diagnostics_summary.txt"),
            [
                self.diagnostics_summary.chair.to_text(),
                self.diagnostics_summary.risk.to_text(),
                self.diagnostics_summary.decision_quality.to_text(),
            ]
            .join("\n"),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("decision_quality.txt"),
            self.diagnostics_summary.decision_quality.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("actionability.txt"),
            self.actionability_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("attribution.txt"),
            self.attribution_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("readiness.txt"),
            self.readiness_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("vs_baseline.txt"),
            self.vs_baseline_report
                .as_ref()
                .map(|report| report.to_text())
                .unwrap_or_else(|| "vs_baseline=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_benchmark_summary.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        let bundle_path = output_dir.join("committee_benchmark_bundle.json");
        fs::write(
            &bundle_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(bundle_path)
    }

    pub fn build_audit_summary(&self) -> String {
        stable_hash_string(&format!(
            "{}|{}|{}",
            self.benchmark_report.benchmark_id,
            self.replay_report.deterministic_fingerprint,
            self.final_summary
        ))
    }
}
