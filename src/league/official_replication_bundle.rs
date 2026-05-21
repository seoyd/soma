use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::committee_official_benchmark::CommitteeOfficialBenchmarkReport;
use super::official_candle_coverage::OfficialCandleCoverageReport;
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::official_evidence_replication::OfficialEvidenceReplicationReport;
use super::official_reference_replication::OfficialReferenceReplicationReport;
use super::official_replication_inventory::OfficialReplicationArtifactInventory;
use super::official_replication_operator_actions::OfficialReplicationOperatorActionPlan;
use super::official_sufficiency_replication::OfficialSufficiencyReplicationReport;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceReplicationBundle {
    pub replication_report: OfficialEvidenceReplicationReport,
    pub artifact_inventory: OfficialReplicationArtifactInventory,
    #[serde(default)]
    pub injected_scenario_pack: Option<OfficialCommitteeScenarioPack>,
    pub official_candle_coverage: OfficialCandleCoverageReport,
    #[serde(default)]
    pub reference_replication: Option<OfficialReferenceReplicationReport>,
    pub sufficiency_replication: OfficialSufficiencyReplicationReport,
    #[serde(default)]
    pub official_benchmark: Option<CommitteeOfficialBenchmarkReport>,
    pub operator_actions: OfficialReplicationOperatorActionPlan,
    pub final_summary: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialEvidenceReplicationBundle {
    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("artifact_inventory.txt"),
            self.artifact_inventory.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("row_injection.txt"),
            self.replication_report.row_injection_result.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_candle_coverage.txt"),
            self.official_candle_coverage.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_reference_replication.txt"),
            self.reference_replication
                .as_ref()
                .map(OfficialReferenceReplicationReport::to_text)
                .unwrap_or_else(|| "official_reference_replication=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_sufficiency_replication.txt"),
            self.sufficiency_replication.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_committee_benchmark_summary.txt"),
            self.official_benchmark
                .as_ref()
                .map(CommitteeOfficialBenchmarkReport::to_text)
                .unwrap_or_else(|| "official_committee_benchmark=none".to_string()),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("operator_actions.txt"),
            self.operator_actions.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_replication_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        if let Some(pack) = &self.injected_scenario_pack {
            let _ = pack.write_to_dir(output_dir)?;
        }
        let json_path = output_dir.join("official_replication_bundle.json");
        fs::write(
            &json_path,
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }

    pub fn from_parts(
        replication_report: OfficialEvidenceReplicationReport,
        artifact_inventory: OfficialReplicationArtifactInventory,
        injected_scenario_pack: Option<OfficialCommitteeScenarioPack>,
        official_candle_coverage: OfficialCandleCoverageReport,
        reference_replication: Option<OfficialReferenceReplicationReport>,
        sufficiency_replication: OfficialSufficiencyReplicationReport,
        official_benchmark: Option<CommitteeOfficialBenchmarkReport>,
        operator_actions: OfficialReplicationOperatorActionPlan,
    ) -> Self {
        let final_summary = replication_report.to_text();
        Self {
            replication_report,
            artifact_inventory,
            injected_scenario_pack,
            official_candle_coverage,
            reference_replication,
            sufficiency_replication,
            official_benchmark,
            operator_actions,
            final_summary,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialReplicationBundleBuilt,
                ReasonCode::OfficialEvidenceReplicationBuilt,
            ]),
        }
    }
}
