use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::model::{
    Mamba3FinCandidateReport, Mamba3FinGapAnalysisReport, ModelEscalationGateConfig,
    ModelEscalationGateResult, SequenceDatasetConfig, SequenceDatasetSpec,
    build_mamba3fin_candidate_report, build_mamba3fin_gap_analysis,
};

use super::ai_benchmark::OfficialAiBenchmarkReport;
use super::official_consistency::{OfficialConsistencyConfig, OfficialConsistencyReport};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MambaReadinessConfig {
    pub readiness_id: String,
    pub official_consistency: OfficialConsistencyConfig,
    pub sequence_dataset_config: SequenceDatasetConfig,
    #[serde(default)]
    pub escalation_gate_config: ModelEscalationGateConfig,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl MambaReadinessConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        if self.output_root.contains("://")
            || !self.official_consistency.validate_local_paths().is_empty()
        {
            vec![ReasonCode::LocalPathRejected]
        } else {
            Vec::new()
        }
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.readiness_id)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MambaReadinessBenchmarkReport {
    pub official_consistency_report: OfficialConsistencyReport,
    #[serde(default)]
    pub ai_signal_usefulness_report: Option<super::AiSignalUsefulnessReport>,
    pub mamba3fin_gap_analysis: Mamba3FinGapAnalysisReport,
    pub sequence_dataset_spec: SequenceDatasetSpec,
    pub candidate_report: Mamba3FinCandidateReport,
    pub escalation_gate_result: ModelEscalationGateResult,
    pub final_recommendation: String,
    pub reason_codes: Vec<ReasonCode>,
}

impl MambaReadinessBenchmarkReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            self.official_consistency_report.to_text(),
            self.mamba3fin_gap_analysis.to_text(),
            format!(
                "sequence_dataset_windows={}",
                self.sequence_dataset_spec.estimated_windows
            ),
            format!(
                "sequence_dataset_bytes={}",
                self.sequence_dataset_spec.estimated_bytes
            ),
            self.candidate_report.to_text(),
            self.escalation_gate_result.to_text(),
            format!("final_recommendation={}", self.final_recommendation),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("mamba_readiness_benchmark_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("mamba_readiness_benchmark_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MambaReadinessRunner;

impl MambaReadinessRunner {
    pub fn run(
        &self,
        config: &MambaReadinessConfig,
    ) -> Result<MambaReadinessBenchmarkReport, String> {
        if !config.validate_local_paths().is_empty() {
            return Err("mamba readiness config contains remote URL-like path".to_string());
        }
        let benchmark_reports = config.official_consistency.load_benchmark_reports()?;
        let official_consistency_report =
            OfficialConsistencyReport::build(&config.official_consistency, &benchmark_reports);
        let primary_benchmark = select_primary_benchmark_report(&benchmark_reports);
        let sequence_dataset_spec =
            build_sequence_dataset_spec(primary_benchmark, &config.sequence_dataset_config)?;
        let mamba3fin_gap_analysis = build_mamba3fin_gap_analysis(
            primary_benchmark,
            Some(official_consistency_report.consistency_status),
            Some(&sequence_dataset_spec),
        );
        let candidate_report = build_mamba3fin_candidate_report(
            &mamba3fin_gap_analysis,
            official_consistency_report.consistency_status,
            primary_benchmark.map(|report| report.usefulness_report.status),
            Some(&sequence_dataset_spec),
        );
        let escalation_gate_result = ModelEscalationGateResult::evaluate(
            &config.escalation_gate_config,
            official_consistency_report.consistency_status,
            primary_benchmark.map(|report| report.usefulness_report.status),
            primary_benchmark
                .map(|report| report.usefulness_report.total_outcome_records)
                .unwrap_or(0),
            primary_benchmark
                .map(|report| {
                    report
                        .usefulness_report
                        .calibration_summary
                        .avg_expected_calibration_error
                })
                .unwrap_or(0.0),
            primary_benchmark
                .map(|report| report.usefulness_report.risk_governor_summary.stable)
                .unwrap_or(false),
            primary_benchmark
                .map(|report| !report.storage_audit.budget_exceeded)
                .unwrap_or(false),
            Some(&sequence_dataset_spec),
            &candidate_report,
        );
        let report = MambaReadinessBenchmarkReport {
            official_consistency_report,
            ai_signal_usefulness_report: primary_benchmark
                .map(|report| report.usefulness_report.clone()),
            mamba3fin_gap_analysis,
            sequence_dataset_spec,
            candidate_report,
            final_recommendation: format!("{:?}", escalation_gate_result.decision),
            escalation_gate_result,
            reason_codes: vec![ReasonCode::MambaReadinessBuilt],
        };
        report.write_to_dir(&config.output_dir())?;
        Ok(report)
    }
}

fn select_primary_benchmark_report(
    reports: &[OfficialAiBenchmarkReport],
) -> Option<&OfficialAiBenchmarkReport> {
    reports.iter().max_by_key(|report| {
        (
            report.usefulness_report.official_dataset_count,
            report.usefulness_report.total_outcome_records,
        )
    })
}

fn build_sequence_dataset_spec(
    report: Option<&OfficialAiBenchmarkReport>,
    config: &SequenceDatasetConfig,
) -> Result<SequenceDatasetSpec, String> {
    let dataset_csv = report
        .and_then(|report| {
            report
                .dataset_reports
                .iter()
                .find_map(|dataset| dataset.dataset_export_dir.as_ref())
        })
        .map(|path| Path::new(path).join("dataset.csv"));
    if let Some(path) = dataset_csv {
        return SequenceDatasetSpec::from_dataset_csv_path(&path, config);
    }
    Ok(SequenceDatasetSpec {
        config: config.clone(),
        estimated_windows: 0,
        estimated_bytes: 0,
        no_lookahead_safe: false,
        storage_budget_ok: false,
        reason_codes: vec![ReasonCode::SequenceDatasetSpecBuilt],
    })
}

fn default_output_root() -> String {
    "target/soma_mamba_readiness".to_string()
}
