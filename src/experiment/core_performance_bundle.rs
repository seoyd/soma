use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

use super::core_performance_regression::summary_from_scorecard;
use super::core_performance_scorecard::CorePerformanceScorecard;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceScorecardBundle {
    pub scorecard: CorePerformanceScorecard,
    pub output_dir: String,
    pub written_files: BTreeMap<String, String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CorePerformanceScorecardBundle {
    pub fn write_to_dir(&self, dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(dir).map_err(|err| err.to_string())?;
        write_file(
            dir.join("artifact_inventory.txt"),
            &self.scorecard.artifact_inventory.to_text(),
        )?;
        write_file(
            dir.join("signal_quality.txt"),
            &self.scorecard.signal_quality_report.to_text(),
        )?;
        write_file(
            dir.join("committee_value_attribution.txt"),
            &self.scorecard.committee_value_attribution_report.to_text(),
        )?;
        write_file(
            dir.join("risk_governor_value.txt"),
            &self.scorecard.risk_governor_value_report.to_text(),
        )?;
        write_file(
            dir.join("no_trade_value.txt"),
            &self.scorecard.no_trade_value_report.to_text(),
        )?;
        write_file(
            dir.join("latency_budget.txt"),
            &self.scorecard.latency_budget_report.to_text(),
        )?;
        write_file(
            dir.join("regression_guard.txt"),
            &self
                .scorecard
                .regression_report
                .as_ref()
                .map(|report| report.to_text())
                .unwrap_or_else(|| "regression_guard=none".to_string()),
        )?;
        write_file(
            dir.join("bottleneck_report.txt"),
            &self.scorecard.bottleneck_report.to_text(),
        )?;
        write_file(
            dir.join("core_performance_scorecard.txt"),
            &self.scorecard.to_text(),
        )?;
        write_file(
            dir.join("core_performance_scorecard.json"),
            &self.scorecard.to_json_string()?,
        )?;
        write_file(
            dir.join("core_performance_regression_summary.json"),
            &serde_json::to_string_pretty(&summary_from_scorecard(&self.scorecard))
                .map_err(|err| err.to_string())?,
        )?;
        let bundle_path = dir.join("core_performance_bundle.json");
        write_file(
            &bundle_path,
            &serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )?;
        Ok(bundle_path)
    }
}

fn write_file(path: impl AsRef<Path>, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|err| err.to_string())
}
