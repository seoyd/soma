use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::krx_activation_storage::KRXActivationStorageReport;
use super::krx_auth_readiness::KRXAuthReadinessReport;
use super::krx_canonical_validation::KRXCanonicalValidationReport;
use super::krx_downstream_rerun::KRXDownstreamRerunSummary;
use super::krx_evidence_job::KRXEvidenceJobPlan;
use super::krx_official_activation::KRXOfficialEvidenceActivationReport;
use super::krx_operator_actions::KRXOperatorAction;
use super::krx_symbol_whitelist::KRXSymbolWhitelist;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXOfficialEvidenceActivationBundle {
    pub auth_readiness_report: KRXAuthReadinessReport,
    pub symbol_whitelist: KRXSymbolWhitelist,
    pub job_plan: KRXEvidenceJobPlan,
    pub canonical_validation_reports: Vec<KRXCanonicalValidationReport>,
    pub operator_actions: Vec<KRXOperatorAction>,
    pub downstream_rerun_summary: KRXDownstreamRerunSummary,
    pub activation_report: KRXOfficialEvidenceActivationReport,
    pub storage_report: KRXActivationStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXOfficialEvidenceActivationBundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        self.auth_readiness_report.write_to_dir(output_dir)?;
        self.symbol_whitelist.write_to_dir(output_dir)?;
        self.job_plan.write_to_dir(output_dir)?;
        write_validation_reports(&self.canonical_validation_reports, output_dir)?;
        write_operator_actions(&self.operator_actions, output_dir)?;
        fs::write(
            output_dir.join("krx_downstream_rerun_summary.txt"),
            self.downstream_rerun_summary.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_official_evidence_activation_report.txt"),
            self.activation_report.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_official_evidence_activation_report.json"),
            self.activation_report.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        self.storage_report.write_to_dir(output_dir)?;
        fs::write(
            output_dir.join("krx_official_activation_summary.txt"),
            &self.final_summary,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_official_activation_bundle.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn finalize_reason_codes(&mut self) {
        self.reason_codes = stable_reason_codes(&self.reason_codes);
    }
}

fn write_validation_reports(
    reports: &[KRXCanonicalValidationReport],
    output_dir: &Path,
) -> Result<(), String> {
    let text = reports
        .iter()
        .map(KRXCanonicalValidationReport::to_text)
        .collect::<Vec<_>>()
        .join("\n---\n");
    fs::write(output_dir.join("krx_canonical_validation.txt"), text)
        .map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("krx_canonical_validation.json"),
        serde_json::to_string_pretty(reports).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn write_operator_actions(actions: &[KRXOperatorAction], output_dir: &Path) -> Result<(), String> {
    let text = actions
        .iter()
        .map(KRXOperatorAction::to_text)
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(output_dir.join("krx_operator_actions.txt"), text).map_err(|err| err.to_string())?;
    fs::write(
        output_dir.join("krx_operator_actions.json"),
        serde_json::to_string_pretty(actions).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}
