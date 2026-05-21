use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::StrategyDataCheckRequest;
use super::evidence_lane::{EvidenceLane, EvidenceLaneKind};
use super::lane_storage::ProviderRealityStorageReport;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplicitEvidenceLaneConfig {
    pub lane_kind: EvidenceLaneKind,
    pub provider: String,
    #[serde(default)]
    pub symbols: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub output_subdir: Option<String>,
    #[serde(default)]
    pub max_rows: Option<usize>,
    #[serde(default)]
    pub max_requests: Option<usize>,
    #[serde(default)]
    pub allow_full_history: bool,
    #[serde(default)]
    pub allow_all_symbols: bool,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutableEvidencePlanConfig {
    pub plan_id: String,
    #[serde(default)]
    pub provider_reality_report_path: Option<String>,
    #[serde(default)]
    pub provider_recommendation_request_paths: Vec<String>,
    #[serde(default)]
    pub strategy_compatibility_requests: Vec<StrategyDataCheckRequest>,
    #[serde(default)]
    pub explicit_lanes: Vec<ExplicitEvidenceLaneConfig>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_core_check: bool,
    #[serde(default = "default_true")]
    pub run_collection: bool,
    #[serde(default = "default_true")]
    pub run_preflight: bool,
    #[serde(default = "default_true")]
    pub run_benchmark: bool,
    #[serde(default)]
    pub run_external_eval: bool,
    #[serde(default = "default_true")]
    pub allow_yfinance_research: bool,
    #[serde(default = "default_true")]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_missing_auth_skips: bool,
    #[serde(default = "default_max_lanes")]
    pub max_lanes: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_rows_per_lane")]
    pub max_rows_per_lane: usize,
    #[serde(default = "default_max_requests_per_lane")]
    pub max_requests_per_lane: usize,
    #[serde(default = "default_max_total_bytes")]
    pub max_total_bytes: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecutableEvidencePlan {
    pub plan_id: String,
    pub lanes: Vec<EvidenceLane>,
    pub skipped_lanes: Vec<EvidenceLane>,
    pub runnable_lanes: Vec<EvidenceLane>,
    pub operator_actions: Vec<String>,
    pub storage_budget_summary: ProviderRealityStorageReport,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for ExecutableEvidencePlanConfig {
    fn default() -> Self {
        Self {
            plan_id: "evidence_plan".to_string(),
            provider_reality_report_path: None,
            provider_recommendation_request_paths: Vec::new(),
            strategy_compatibility_requests: Vec::new(),
            explicit_lanes: Vec::new(),
            output_root: "target/soma_evidence_plan".to_string(),
            run_core_check: true,
            run_collection: true,
            run_preflight: true,
            run_benchmark: true,
            run_external_eval: false,
            allow_yfinance_research: true,
            allow_crypto_only: true,
            allow_missing_auth_skips: true,
            max_lanes: default_max_lanes(),
            max_symbols: default_max_symbols(),
            max_rows_per_lane: default_max_rows_per_lane(),
            max_requests_per_lane: default_max_requests_per_lane(),
            max_total_bytes: default_max_total_bytes(),
            reason_codes: vec![ReasonCode::ExecutableEvidencePlanBuilt],
        }
    }
}

impl ExecutableEvidencePlanConfig {
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

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        if contains_remote_path(&self.output_root) {
            reasons.push(ReasonCode::RemotePathRejected);
        }
        if self
            .provider_reality_report_path
            .as_deref()
            .is_some_and(contains_remote_path)
        {
            reasons.push(ReasonCode::RemotePathRejected);
        }
        if self
            .provider_recommendation_request_paths
            .iter()
            .any(|path| contains_remote_path(path))
        {
            reasons.push(ReasonCode::RemotePathRejected);
        }
        reasons
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.validate_local_paths().is_empty() {
            return Err("executable evidence plan config must use local-only paths".to_string());
        }
        if self.max_lanes == 0 || self.max_symbols == 0 {
            return Err("max_lanes and max_symbols must be positive".to_string());
        }
        if self.max_rows_per_lane == 0 || self.max_requests_per_lane == 0 {
            return Err("max_rows_per_lane and max_requests_per_lane must be positive".to_string());
        }
        if self.explicit_lanes.len() > self.max_lanes {
            return Err("explicit lane count exceeds max_lanes".to_string());
        }
        for lane in &self.explicit_lanes {
            if lane.allow_all_symbols {
                return Err("all-symbol explicit lanes are rejected".to_string());
            }
            if lane.allow_full_history {
                return Err("full-history explicit lanes are rejected".to_string());
            }
            if lane.symbols.len() > self.max_symbols {
                return Err("explicit lane symbol count exceeds max_symbols".to_string());
            }
            if lane
                .symbols
                .iter()
                .any(|symbol| symbol == "*" || symbol.eq_ignore_ascii_case("all"))
            {
                return Err("all-symbol explicit lane is rejected".to_string());
            }
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.plan_id)
    }
}

impl ExecutableEvidencePlan {
    pub fn new(
        plan_id: String,
        mut lanes: Vec<EvidenceLane>,
        mut operator_actions: Vec<String>,
        storage_budget_summary: ProviderRealityStorageReport,
    ) -> Self {
        lanes.sort_by(|left, right| left.lane_id.cmp(&right.lane_id));
        operator_actions.sort();
        operator_actions.dedup();
        let skipped_lanes = lanes
            .iter()
            .filter(|lane| !lane.is_runnable())
            .cloned()
            .collect::<Vec<_>>();
        let runnable_lanes = lanes
            .iter()
            .filter(|lane| lane.is_runnable())
            .cloned()
            .collect::<Vec<_>>();
        Self {
            plan_id,
            lanes,
            skipped_lanes,
            runnable_lanes,
            operator_actions,
            storage_budget_summary,
            reason_codes: vec![ReasonCode::ExecutableEvidencePlanBuilt],
        }
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("lane_count={}", self.lanes.len()),
            format!("runnable_lane_count={}", self.runnable_lanes.len()),
            format!("skipped_lane_count={}", self.skipped_lanes.len()),
            format!(
                "storage_budget_exceeded={}",
                self.storage_budget_summary.budget_exceeded
            ),
        ];
        for lane in &self.lanes {
            lines.push(format!(
                "lane={};kind={:?};status={:?};provider={:?};source={:?}",
                lane.lane_id,
                lane.lane_kind,
                lane.lane_status,
                lane.provider_kind,
                lane.source_kind
            ));
        }
        for action in &self.operator_actions {
            lines.push(format!("operator_action={action}"));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("executable_evidence_plan.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("executable_evidence_plan.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

fn contains_remote_path(value: &str) -> bool {
    value.contains("://")
}

fn default_true() -> bool {
    true
}

fn default_max_lanes() -> usize {
    5
}

fn default_max_symbols() -> usize {
    3
}

fn default_max_rows_per_lane() -> usize {
    500
}

fn default_max_requests_per_lane() -> usize {
    10
}

fn default_max_total_bytes() -> usize {
    1_000_000
}
