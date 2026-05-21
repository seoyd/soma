use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeV1RunConfig {
    pub run_id: String,
    #[serde(default)]
    pub scenario_load_config_path: Option<String>,
    #[serde(default)]
    pub replay_config_path: Option<String>,
    #[serde(default)]
    pub diagnostics_config_path: Option<String>,
    #[serde(default)]
    pub source_report_paths: Vec<String>,
    #[serde(default)]
    pub yfinance_report_paths: Vec<String>,
    #[serde(default)]
    pub evidence_lane_report_paths: Vec<String>,
    #[serde(default)]
    pub fixture_paths: Vec<String>,
    pub output_root: String,
    #[serde(default = "default_max_scenarios")]
    pub max_scenarios: usize,
    #[serde(default = "default_max_decisions")]
    pub max_decisions: usize,
    #[serde(default)]
    pub require_core_check: bool,
    #[serde(default = "default_true")]
    pub allow_fixture: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_summary_derived_rows: bool,
    #[serde(default = "default_true")]
    pub run_scenario_loading: bool,
    #[serde(default = "default_true")]
    pub run_replay: bool,
    #[serde(default = "default_true")]
    pub run_diagnostics: bool,
    #[serde(default = "default_true")]
    pub run_quality_metrics: bool,
    #[serde(default = "default_true")]
    pub run_calibration_suggestions: bool,
    #[serde(default = "default_true")]
    pub run_readiness_gate: bool,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for CommitteeV1RunConfig {
    fn default() -> Self {
        Self {
            run_id: "committee_v1".to_string(),
            scenario_load_config_path: None,
            replay_config_path: None,
            diagnostics_config_path: None,
            source_report_paths: Vec::new(),
            yfinance_report_paths: Vec::new(),
            evidence_lane_report_paths: Vec::new(),
            fixture_paths: Vec::new(),
            output_root: "target/soma_committee_v1".to_string(),
            max_scenarios: default_max_scenarios(),
            max_decisions: default_max_decisions(),
            require_core_check: false,
            allow_fixture: true,
            allow_yfinance_research: true,
            allow_crypto_only: true,
            allow_summary_derived_rows: true,
            run_scenario_loading: true,
            run_replay: true,
            run_diagnostics: true,
            run_quality_metrics: true,
            run_calibration_suggestions: true,
            run_readiness_gate: true,
            reason_codes: vec![ReasonCode::CommitteeV1Built],
        }
    }
}

impl CommitteeV1RunConfig {
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

    pub fn validate(&self) -> Result<(), String> {
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
                .diagnostics_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .source_report_paths
                .iter()
                .chain(self.yfinance_report_paths.iter())
                .chain(self.evidence_lane_report_paths.iter())
                .chain(self.fixture_paths.iter())
                .any(|path| path.contains("://"))
        {
            return Err("committee-v1 paths must be local".to_string());
        }
        if self.max_scenarios == 0 || self.max_scenarios > default_max_scenarios() {
            return Err("committee-v1 max_scenarios must be between 1 and 50".to_string());
        }
        if self.max_decisions == 0 || self.max_decisions > default_max_decisions() {
            return Err("committee-v1 max_decisions must be between 1 and 50".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.run_id)
    }
}

fn default_true() -> bool {
    true
}

fn default_max_scenarios() -> usize {
    50
}

fn default_max_decisions() -> usize {
    50
}
