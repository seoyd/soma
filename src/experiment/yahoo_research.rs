use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{YFinanceImportConfig, run_yfinance_preflight_bridge};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YahooResearchEvidenceConfig {
    pub research_id: String,
    pub output_root: String,
    pub imports: Vec<YFinanceImportConfig>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct YahooResearchEvidenceReport {
    pub research_id: String,
    pub yfinance_symbols: Vec<String>,
    pub canonical_csv_paths: Vec<String>,
    pub provenance_paths: Vec<String>,
    pub preflight_statuses: Vec<String>,
    pub official_readiness_eligible_count: usize,
    pub benchmark_eligible_count: usize,
    pub total_rows: usize,
    pub total_storage_bytes: u64,
    pub generated_config_paths: Vec<String>,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct YahooResearchEvidenceRunner;

impl Default for YahooResearchEvidenceConfig {
    fn default() -> Self {
        Self {
            research_id: "yahoo_research".to_string(),
            output_root: "target/soma_yahoo_research".to_string(),
            imports: vec![YFinanceImportConfig::default()],
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl YahooResearchEvidenceConfig {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }

    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.research_id)
    }
}

impl YahooResearchEvidenceReport {
    pub fn to_text(&self) -> String {
        [
            format!("research_id={}", self.research_id),
            format!("yfinance_symbols={}", self.yfinance_symbols.join(" | ")),
            format!(
                "canonical_csv_paths={}",
                self.canonical_csv_paths.join(" | ")
            ),
            format!("provenance_paths={}", self.provenance_paths.join(" | ")),
            format!("preflight_statuses={}", self.preflight_statuses.join(" | ")),
            format!(
                "official_readiness_eligible_count={}",
                self.official_readiness_eligible_count
            ),
            format!("benchmark_eligible_count={}", self.benchmark_eligible_count),
            format!("total_rows={}", self.total_rows),
            format!("total_storage_bytes={}", self.total_storage_bytes),
            format!(
                "generated_config_paths={}",
                self.generated_config_paths.join(" | ")
            ),
            format!("warnings={}", self.warnings.join(" | ")),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("yahoo_research_evidence_report.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("yahoo_research_evidence_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl YahooResearchEvidenceRunner {
    pub fn run(
        &self,
        config: &YahooResearchEvidenceConfig,
    ) -> Result<YahooResearchEvidenceReport, String> {
        if config.output_root.contains("://")
            || config
                .imports
                .iter()
                .any(|value| !value.validate_local_paths().is_empty())
        {
            return Err("yahoo-research paths must be local".to_string());
        }
        let output_dir = config.output_dir();
        fs::create_dir_all(&output_dir).map_err(|err| err.to_string())?;

        let mut yfinance_symbols = Vec::new();
        let mut canonical_csv_paths = Vec::new();
        let mut provenance_paths = Vec::new();
        let mut preflight_statuses = Vec::new();
        let mut official_readiness_eligible_count = 0usize;
        let mut benchmark_eligible_count = 0usize;
        let mut total_rows = 0usize;
        let mut total_storage_bytes = 0u64;
        let mut generated_config_paths = Vec::new();

        for import in &config.imports {
            let bridge = run_yfinance_preflight_bridge(import)?;
            yfinance_symbols.push(import.symbol.clone());
            canonical_csv_paths.push(import.canonical_csv_path.clone());
            if let Some(path) = import.provenance_path.clone() {
                provenance_paths.push(path);
            }
            preflight_statuses.push(format!("{:?}", bridge.preflight_report.final_status));
            if bridge.official_readiness_eligible {
                official_readiness_eligible_count += 1;
            }
            if bridge.benchmark_eligible {
                benchmark_eligible_count += 1;
            }
            total_rows += bridge.preflight_report.row_count;
            total_storage_bytes += file_size_u64(&import.canonical_csv_path)?;
            generated_config_paths.extend(bridge.generated_config_paths);
        }

        let report = YahooResearchEvidenceReport {
            research_id: config.research_id.clone(),
            yfinance_symbols,
            canonical_csv_paths,
            provenance_paths,
            preflight_statuses,
            official_readiness_eligible_count,
            benchmark_eligible_count,
            total_rows,
            total_storage_bytes,
            generated_config_paths,
            warnings: vec![
                "yfinance research data stays separate from official evidence claims".to_string(),
                "official_readiness_eligible_count should remain zero".to_string(),
            ],
            reason_codes: vec![
                ReasonCode::YFinanceResearchReportBuilt,
                ReasonCode::YFinanceUnofficialEvidence,
            ],
        };
        report.write_to_dir(&output_dir)?;
        Ok(report)
    }
}

fn file_size_u64(path: &str) -> Result<u64, String> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|err| err.to_string())
}
