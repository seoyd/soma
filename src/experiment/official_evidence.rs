use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{OfficialCollectionEntryStatus, OfficialCollectionReport};

use super::{
    AblationRunner, AblationStudyConfig, BatchExperimentRunner, ExperimentMatrixConfig,
    RealEvidenceClosureConfig, RealEvidenceClosureRunner, RealEvidenceRecommendation,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceRunConfig {
    #[serde(default)]
    pub collection_report_path: Option<String>,
    #[serde(default)]
    pub generated_rerun_configs: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_real_evidence: bool,
    #[serde(default = "default_true")]
    pub run_batch: bool,
    #[serde(default = "default_true")]
    pub run_ablation: bool,
    #[serde(default)]
    pub require_ready_entries: bool,
    #[serde(default = "default_one")]
    pub min_ready_entries: usize,
    #[serde(default = "default_twenty")]
    pub min_outcome_records: usize,
    #[serde(default = "default_two")]
    pub min_comparable_variants: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialEvidenceRunConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialEvidenceRecommendation {
    NeedMoreExperiments,
    MissingAuth,
    ImproveDataFirst,
    ImproveRiskGovernorFirst,
    ImproveSignalModelFirst,
    HoldCurrentScope,
    ReadyForSixPersonaDesignReview,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceRunReport {
    pub ready_entries: Vec<String>,
    pub real_evidence_status: String,
    pub outcome_records: usize,
    pub comparable_variants: usize,
    pub readiness_before: String,
    pub readiness_after: String,
    pub recommendation: OfficialEvidenceRecommendation,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialEvidenceRunReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        [
            format!("ready_entries={}", self.ready_entries.join("|")),
            format!("real_evidence_status={}", self.real_evidence_status),
            format!("outcome_records={}", self.outcome_records),
            format!("comparable_variants={}", self.comparable_variants),
            format!("readiness_before={}", self.readiness_before),
            format!("readiness_after={}", self.readiness_after),
            format!("recommendation={:?}", self.recommendation),
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_run_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("official_evidence_run_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OfficialEvidenceRunner {
    pub real_evidence_runner: RealEvidenceClosureRunner,
    pub batch_runner: BatchExperimentRunner,
    pub ablation_runner: AblationRunner,
}

impl OfficialEvidenceRunner {
    pub fn run(&self, config: &OfficialEvidenceRunConfig) -> OfficialEvidenceRunReport {
        let collection_report = config
            .collection_report_path
            .as_deref()
            .and_then(|path| OfficialCollectionReport::from_json_path(Path::new(path)).ok());

        let mut ready_entry_ids = Vec::new();
        let mut ready_dirs = Vec::new();
        let mut missing_auth = false;
        if let Some(report) = &collection_report {
            for entry in &report.entry_reports {
                if entry.status == OfficialCollectionEntryStatus::SkippedMissingAuth {
                    missing_auth = true;
                }
                if entry.ready_for_evidence {
                    ready_entry_ids.push(entry.entry_id.clone());
                    if let Some(path) = &entry.canonical_csv_path {
                        if let Some(dir) = Path::new(path).parent().and_then(Path::parent) {
                            ready_dirs.push(dir.to_path_buf());
                        }
                    }
                }
            }
        }
        for path in &config.generated_rerun_configs {
            let path = PathBuf::from(path);
            if let Some(dir) = path.parent() {
                ready_dirs.push(dir.to_path_buf());
                ready_entry_ids.push(
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("generated")
                        .to_string(),
                );
            }
        }
        ready_dirs.sort();
        ready_dirs.dedup();
        ready_entry_ids.sort();
        ready_entry_ids.dedup();

        let mut outcome_records = 0usize;
        let mut comparable_variants = 0usize;
        let mut blockers = Vec::new();
        let mut warnings = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        let readiness_before = collection_report
            .as_ref()
            .map(|report| format!("ready_entries={}", report.ready_entries_count))
            .unwrap_or_else(|| "ready_entries=0".to_string());
        let mut strongest_recommendation = OfficialEvidenceRecommendation::HoldCurrentScope;

        if config.require_ready_entries && ready_dirs.len() < config.min_ready_entries {
            blockers.push(format!(
                "ready official entries {} < required {}",
                ready_dirs.len(),
                config.min_ready_entries
            ));
        }

        for dir in &ready_dirs {
            if config.run_real_evidence {
                let real_config_path = dir.join("generated_real_evidence_closure.toml");
                if real_config_path.exists() {
                    if let Ok(real_config) =
                        RealEvidenceClosureConfig::from_toml_path(&real_config_path)
                    {
                        let report = self.real_evidence_runner.run(&real_config);
                        outcome_records = outcome_records.saturating_add(
                            report.source_evidence_summary.readiness_eligible_outcomes,
                        );
                        comparable_variants = comparable_variants.saturating_add(
                            report.source_evidence_summary.readiness_eligible_variants,
                        );
                        strongest_recommendation = merge_recommendation(
                            strongest_recommendation,
                            report.final_recommendation,
                        );
                    }
                }
            }
            if config.run_batch {
                let batch_config_path = dir.join("generated_batch_matrix.toml");
                if batch_config_path.exists() {
                    if let Ok(batch_config) =
                        ExperimentMatrixConfig::from_toml_path(&batch_config_path)
                    {
                        let _ = self.batch_runner.run_matrix(&batch_config);
                    }
                }
            }
            if config.run_ablation {
                let ablation_config_path = dir.join("generated_ablation_study.toml");
                if ablation_config_path.exists() {
                    if let Ok(ablation_config) =
                        AblationStudyConfig::from_toml_path(&ablation_config_path)
                    {
                        let _ = self.ablation_runner.run_study(&ablation_config);
                    }
                }
            }
        }

        if ready_dirs.is_empty() {
            warnings.push("no ready official entries available for evidence execution".to_string());
        }
        if outcome_records < config.min_outcome_records {
            blockers.push(format!(
                "official outcome records {} < required {}",
                outcome_records, config.min_outcome_records
            ));
        }
        if comparable_variants < config.min_comparable_variants {
            blockers.push(format!(
                "official comparable variants {} < required {}",
                comparable_variants, config.min_comparable_variants
            ));
        }

        let recommendation = if missing_auth && ready_dirs.len() < config.min_ready_entries {
            reason_codes.push(ReasonCode::OfficialEvidenceMissingAuth);
            OfficialEvidenceRecommendation::MissingAuth
        } else if !blockers.is_empty() {
            reason_codes.push(ReasonCode::OfficialEvidenceNeedMoreExperiments);
            strongest_recommendation = if matches!(
                strongest_recommendation,
                OfficialEvidenceRecommendation::HoldCurrentScope
                    | OfficialEvidenceRecommendation::ReadyForSixPersonaDesignReview
            ) {
                OfficialEvidenceRecommendation::NeedMoreExperiments
            } else {
                strongest_recommendation
            };
            strongest_recommendation
        } else {
            strongest_recommendation
        };
        let readiness_after = format!(
            "ready_entries={} outcomes={} variants={}",
            ready_dirs.len(),
            outcome_records,
            comparable_variants
        );
        reason_codes.push(ReasonCode::OfficialEvidenceCounted);
        reason_codes.push(ReasonCode::OfficialEvidenceRunCompleted);
        let report = OfficialEvidenceRunReport {
            ready_entries: ready_entry_ids,
            real_evidence_status: if config.run_real_evidence {
                "ran".to_string()
            } else {
                "skipped".to_string()
            },
            outcome_records,
            comparable_variants,
            readiness_before,
            readiness_after,
            recommendation,
            blockers,
            warnings,
            reason_codes: dedupe_reasons(reason_codes),
        };
        let _ = report.write_to_dir(Path::new(&config.output_root));
        report
    }
}

fn merge_recommendation(
    current: OfficialEvidenceRecommendation,
    next: RealEvidenceRecommendation,
) -> OfficialEvidenceRecommendation {
    let next = match next {
        RealEvidenceRecommendation::NeedMoreExperiments => {
            OfficialEvidenceRecommendation::NeedMoreExperiments
        }
        RealEvidenceRecommendation::MissingRealLocalData => {
            OfficialEvidenceRecommendation::MissingAuth
        }
        RealEvidenceRecommendation::ImproveDataFirst => {
            OfficialEvidenceRecommendation::ImproveDataFirst
        }
        RealEvidenceRecommendation::ImproveRiskGovernorFirst => {
            OfficialEvidenceRecommendation::ImproveRiskGovernorFirst
        }
        RealEvidenceRecommendation::ImproveSignalModelFirst => {
            OfficialEvidenceRecommendation::ImproveSignalModelFirst
        }
        RealEvidenceRecommendation::HoldCurrentScope => {
            OfficialEvidenceRecommendation::HoldCurrentScope
        }
        RealEvidenceRecommendation::ReadyForSixPersonaDesignReview => {
            OfficialEvidenceRecommendation::ReadyForSixPersonaDesignReview
        }
    };
    match (current, next) {
        (OfficialEvidenceRecommendation::MissingAuth, _)
        | (_, OfficialEvidenceRecommendation::MissingAuth) => {
            OfficialEvidenceRecommendation::MissingAuth
        }
        (OfficialEvidenceRecommendation::ImproveDataFirst, _)
        | (_, OfficialEvidenceRecommendation::ImproveDataFirst) => {
            OfficialEvidenceRecommendation::ImproveDataFirst
        }
        (OfficialEvidenceRecommendation::ImproveRiskGovernorFirst, _)
        | (_, OfficialEvidenceRecommendation::ImproveRiskGovernorFirst) => {
            OfficialEvidenceRecommendation::ImproveRiskGovernorFirst
        }
        (OfficialEvidenceRecommendation::ImproveSignalModelFirst, _)
        | (_, OfficialEvidenceRecommendation::ImproveSignalModelFirst) => {
            OfficialEvidenceRecommendation::ImproveSignalModelFirst
        }
        (OfficialEvidenceRecommendation::NeedMoreExperiments, _)
        | (_, OfficialEvidenceRecommendation::NeedMoreExperiments) => {
            OfficialEvidenceRecommendation::NeedMoreExperiments
        }
        (OfficialEvidenceRecommendation::ReadyForSixPersonaDesignReview, _)
        | (_, OfficialEvidenceRecommendation::ReadyForSixPersonaDesignReview) => {
            OfficialEvidenceRecommendation::ReadyForSixPersonaDesignReview
        }
        _ => OfficialEvidenceRecommendation::HoldCurrentScope,
    }
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}

fn default_true() -> bool {
    true
}

fn default_output_root() -> String {
    "target/soma_official_evidence_run".to_string()
}

fn default_one() -> usize {
    1
}

fn default_two() -> usize {
    2
}

fn default_twenty() -> usize {
    20
}
