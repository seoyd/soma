use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string};

use super::committee_benchmark::CommitteeBenchmarkReport;
use super::committee_outcome_linked_comparison::CommitteeOutcomeLinkedComparison;
use super::committee_outcome_linker::OutcomeLinkedCommitteeScenarioPack;
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::official_committee_readiness::OfficialCommitteeEvidenceReadinessReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeOfficialBenchmarkBundle {
    pub official_scenario_pack: OfficialCommitteeScenarioPack,
    pub outcome_linked_pack: OutcomeLinkedCommitteeScenarioPack,
    pub committee_benchmark_report: CommitteeBenchmarkReport,
    pub outcome_linked_comparison: CommitteeOutcomeLinkedComparison,
    pub official_readiness_report: OfficialCommitteeEvidenceReadinessReport,
    pub audit_summary: String,
    pub storage_summary: String,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl CommitteeOfficialBenchmarkBundle {
    pub fn to_text(&self) -> String {
        [
            self.official_scenario_pack.to_text(),
            self.outcome_linked_pack.to_text(),
            self.committee_benchmark_report.to_text(),
            self.outcome_linked_comparison.to_text(),
            self.official_readiness_report.to_text(),
            format!("audit_summary={}", self.audit_summary),
            format!("storage_summary={}", self.storage_summary),
            format!("final_summary={}", self.final_summary),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_scenario_pack.txt"),
            self.official_scenario_pack.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_scenario_pack.json"),
            self.official_scenario_pack.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("outcome_link_summary.txt"),
            self.outcome_linked_pack.link_summary.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_benchmark_report.txt"),
            self.committee_benchmark_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("outcome_linked_comparison.txt"),
            self.outcome_linked_comparison.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_readiness.txt"),
            self.official_readiness_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_committee_benchmark_summary.txt"),
            self.final_summary.clone(),
        )
        .map_err(|err| err.to_string())?;
        let bundle_path = output_dir.join("committee_official_benchmark_bundle.json");
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
            self.official_scenario_pack.pack_id, self.storage_summary, self.final_summary
        ))
    }
}
