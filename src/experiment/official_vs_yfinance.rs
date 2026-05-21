use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OfficialVsYFinanceStatus {
    ResearchOnlyNoOfficialClaim,
    ResearchComparisonOnly,
    OfficialOnly,
    NoEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialVsYFinanceInterpretation {
    pub status: OfficialVsYFinanceStatus,
    pub official_ready_count: usize,
    pub yfinance_research_count: usize,
    pub can_compare_for_research: bool,
    pub can_count_as_official: bool,
    pub can_count_as_readiness: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialVsYFinanceInterpretation {
    pub fn to_text(&self) -> String {
        [
            format!("status={:?}", self.status),
            format!("official_ready_count={}", self.official_ready_count),
            format!("yfinance_research_count={}", self.yfinance_research_count),
            format!("can_compare_for_research={}", self.can_compare_for_research),
            format!("can_count_as_official={}", self.can_count_as_official),
            format!("can_count_as_readiness={}", self.can_count_as_readiness),
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
            output_dir.join("official_vs_yfinance.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        fs::write(output_dir.join("official_vs_yfinance.txt"), self.to_text())
            .map_err(|err| err.to_string())?;
        Ok(())
    }
}

pub fn build_official_vs_yfinance_interpretation(
    official_ready_count: usize,
    yfinance_research_count: usize,
    official_reference_metric: Option<f64>,
    yfinance_reference_metric: Option<f64>,
) -> OfficialVsYFinanceInterpretation {
    let status = match (official_ready_count > 0, yfinance_research_count > 0) {
        (false, false) => OfficialVsYFinanceStatus::NoEvidence,
        (false, true) => OfficialVsYFinanceStatus::ResearchOnlyNoOfficialClaim,
        (true, false) => OfficialVsYFinanceStatus::OfficialOnly,
        (true, true) => OfficialVsYFinanceStatus::ResearchComparisonOnly,
    };

    let mut warnings = Vec::new();
    if yfinance_research_count > 0 {
        warnings.push(
            "yfinance evidence is unofficial and cannot count as official coverage".to_string(),
        );
        warnings.push(
            "yfinance evidence cannot satisfy readiness thresholds in this repository".to_string(),
        );
    }
    if let (Some(official), Some(yfinance)) = (official_reference_metric, yfinance_reference_metric)
    {
        let denom = official.abs().max(1e-9);
        let rel_gap = (official - yfinance).abs() / denom;
        if rel_gap > 0.05 {
            warnings.push("DataSourceMismatch: official and yfinance metrics diverge".to_string());
        }
    }

    OfficialVsYFinanceInterpretation {
        status,
        official_ready_count,
        yfinance_research_count,
        can_compare_for_research: official_ready_count > 0 && yfinance_research_count > 0,
        can_count_as_official: false,
        can_count_as_readiness: false,
        warnings,
        reason_codes: vec![
            ReasonCode::OfficialVsYFinanceInterpretationBuilt,
            ReasonCode::YFinanceUnofficialEvidence,
        ],
    }
}
