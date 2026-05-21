use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};

use super::krx_candle_sufficiency::{KRXCandleSufficiencyReport, KRXCandleSufficiencyStatus};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KRXOutcomeLinkClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub krx_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub krx_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub krx_candle_sufficiency_paths: Vec<String>,
    #[serde(default)]
    pub official_ready_rows_paths: Vec<String>,
    #[serde(default)]
    pub barrier_profile_registry_path: Option<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_true")]
    pub run_future_window_requirements: bool,
    #[serde(default = "default_true")]
    pub run_outcome_linkage_v3: bool,
    #[serde(default = "default_true")]
    pub run_counterfactual_completion_v2: bool,
    #[serde(default = "default_true")]
    pub run_complete_row_close_v2: bool,
    #[serde(default)]
    pub run_official_evidence_scaleout: bool,
    #[serde(default)]
    pub run_official_evidence_diversity_sweep: bool,
    #[serde(default)]
    pub run_core_performance: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXOutcomeLinkClosureStatus {
    KRXOutcomeLinksImproved,
    KRXCounterfactualsImproved,
    KRXCompleteRowsImproved,
    StillMissingOfficialCandles,
    StillMissingFutureWindows,
    StillMissingOutcomeLinks,
    StillMissingCounterfactuals,
    StillNeedMoreKRXRows,
    NoImprovement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KRXOutcomeLinkClosureRecommendation {
    CollectLongerKRXWindow,
    MoreKRXOfficialRows,
    MoreOutcomeDiversity,
    MoreCounterfactualDepth,
    RunDiversitySweep,
    RunCorePerformance,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KRXOutcomeLinkClosureReport {
    pub closure_id: String,
    pub krx_official_rows: usize,
    pub krx_official_ready_candles: usize,
    pub generated_outcome_links: usize,
    pub generated_no_trade_counterfactuals: usize,
    pub generated_risk_denied_counterfactuals: usize,
    pub complete_krx_rows: usize,
    pub missing_future_window_rows: usize,
    pub missing_outcome_rows: usize,
    pub missing_counterfactual_rows: usize,
    #[serde(default)]
    pub previous_core_status: Option<String>,
    #[serde(default)]
    pub current_core_status: Option<String>,
    #[serde(default)]
    pub previous_primary_bottleneck: Option<String>,
    #[serde(default)]
    pub current_primary_bottleneck: Option<String>,
    pub bottleneck_changed: bool,
    pub closure_status: KRXOutcomeLinkClosureStatus,
    pub final_recommendation: KRXOutcomeLinkClosureRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KRXOutcomeLinkClosureRunner;

#[derive(Default)]
struct SupplementalCounts {
    risk_rows: usize,
    baseline_rows: usize,
    unique_outcomes: BTreeSet<String>,
}

impl Default for KRXOutcomeLinkClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "krx_outcome_link_closure".to_string(),
            krx_activation_report_paths: Vec::new(),
            krx_canonical_csv_paths: Vec::new(),
            krx_candle_sufficiency_paths: Vec::new(),
            official_ready_rows_paths: Vec::new(),
            barrier_profile_registry_path: None,
            output_root: default_output_root(),
            run_future_window_requirements: true,
            run_outcome_linkage_v3: true,
            run_counterfactual_completion_v2: true,
            run_complete_row_close_v2: true,
            run_official_evidence_scaleout: false,
            run_official_evidence_diversity_sweep: false,
            run_core_performance: false,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl KRXOutcomeLinkClosureConfig {
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

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.closure_id)
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            Some(self.output_root.as_str()),
            self.barrier_profile_registry_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        .chain(self.krx_activation_report_paths.iter().map(String::as_str))
        .chain(self.krx_canonical_csv_paths.iter().map(String::as_str))
        .chain(self.krx_candle_sufficiency_paths.iter().map(String::as_str))
        .chain(self.official_ready_rows_paths.iter().map(String::as_str))
        {
            if path.contains("://") {
                reasons.push(ReasonCode::LocalPathRejected);
            }
        }
        stable_reason_codes(&reasons)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.closure_id.trim().is_empty() {
            return Err("krx outcome closure id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("krx outcome closure paths must be local".to_string());
        }
        Ok(())
    }
}

impl KRXOutcomeLinkClosureRunner {
    pub fn run(
        &self,
        config: &KRXOutcomeLinkClosureConfig,
    ) -> Result<KRXOutcomeLinkClosureReport, String> {
        config.validate()?;
        let report = build_outcome_link_closure_report(config)?;
        let output_dir = config.output_dir();
        report.write_to_dir(&output_dir)?;
        Ok(report)
    }
}

impl KRXOutcomeLinkClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!("krx_official_rows={}", self.krx_official_rows),
            format!(
                "krx_official_ready_candles={}",
                self.krx_official_ready_candles
            ),
            format!("generated_outcome_links={}", self.generated_outcome_links),
            format!(
                "generated_no_trade_counterfactuals={}",
                self.generated_no_trade_counterfactuals
            ),
            format!(
                "generated_risk_denied_counterfactuals={}",
                self.generated_risk_denied_counterfactuals
            ),
            format!("complete_krx_rows={}", self.complete_krx_rows),
            format!(
                "missing_future_window_rows={}",
                self.missing_future_window_rows
            ),
            format!("missing_outcome_rows={}", self.missing_outcome_rows),
            format!(
                "missing_counterfactual_rows={}",
                self.missing_counterfactual_rows
            ),
            format!(
                "previous_core_status={}",
                self.previous_core_status.clone().unwrap_or_default()
            ),
            format!(
                "current_core_status={}",
                self.current_core_status.clone().unwrap_or_default()
            ),
            format!(
                "previous_primary_bottleneck={}",
                self.previous_primary_bottleneck.clone().unwrap_or_default()
            ),
            format!(
                "current_primary_bottleneck={}",
                self.current_primary_bottleneck.clone().unwrap_or_default()
            ),
            format!("bottleneck_changed={}", self.bottleneck_changed),
            format!("closure_status={:?}", self.closure_status),
            format!("final_recommendation={:?}", self.final_recommendation),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }

    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_outcome_link_closure.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_outcome_link_closure.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

pub fn build_outcome_link_closure_report(
    config: &KRXOutcomeLinkClosureConfig,
) -> Result<KRXOutcomeLinkClosureReport, String> {
    let sufficiency = load_sufficiency(config)?;
    let supplemental = load_supplemental_counts(&config.official_ready_rows_paths);
    let krx_official_rows = sufficiency
        .items
        .iter()
        .filter(|item| item.official_ready)
        .map(|item| item.row_count)
        .sum::<usize>();
    let krx_official_ready_candles = sufficiency
        .items
        .iter()
        .filter(|item| item.official_ready)
        .map(|item| item.row_count)
        .sum::<usize>();
    let generated_outcome_links = if config.run_outcome_linkage_v3 {
        sufficiency
            .items
            .iter()
            .filter(|item| item.benchmark_ready)
            .map(|item| {
                item.row_count
                    .saturating_sub(item.required_future_bars.unwrap_or_default())
            })
            .sum::<usize>()
    } else {
        0
    };
    let generated_no_trade_counterfactuals = if config.run_counterfactual_completion_v2 {
        if supplemental.baseline_rows > 0 {
            generated_outcome_links.min(supplemental.baseline_rows)
        } else {
            generated_outcome_links
        }
    } else {
        0
    };
    let generated_risk_denied_counterfactuals =
        if config.run_counterfactual_completion_v2 && supplemental.risk_rows > 0 {
            generated_outcome_links.min(supplemental.risk_rows)
        } else {
            0
        };
    let complete_krx_rows = if config.run_complete_row_close_v2 {
        if generated_risk_denied_counterfactuals > 0 {
            generated_outcome_links
                .min(generated_no_trade_counterfactuals)
                .min(generated_risk_denied_counterfactuals)
        } else {
            generated_outcome_links.min(generated_no_trade_counterfactuals)
        }
    } else {
        0
    };
    let missing_future_window_rows = if config.run_future_window_requirements {
        sufficiency
            .items
            .iter()
            .map(|item| item.missing_future_bars.unwrap_or(0).min(item.row_count))
            .sum::<usize>()
    } else {
        0
    };
    let missing_outcome_rows = krx_official_ready_candles.saturating_sub(generated_outcome_links);
    let missing_counterfactual_rows = generated_outcome_links
        .saturating_sub(generated_no_trade_counterfactuals)
        + if supplemental.risk_rows > 0 {
            generated_outcome_links.saturating_sub(generated_risk_denied_counterfactuals)
        } else {
            0
        };
    let previous_core_status = Some("CoreBlockedByOfficialData".to_string());
    let current_core_status = Some(
        if generated_outcome_links == 0 {
            "CoreBlockedByOfficialData"
        } else if complete_krx_rows == 0 {
            "CoreBlockedByCounterfactuals"
        } else {
            "CoreResearchReady"
        }
        .to_string(),
    );
    let previous_primary_bottleneck = Some("MissingOfficialCandles".to_string());
    let current_primary_bottleneck = Some(
        if krx_official_ready_candles == 0 {
            "MissingOfficialCandles"
        } else if generated_outcome_links == 0 && missing_future_window_rows > 0 {
            "MissingFutureWindows"
        } else if generated_outcome_links == 0 {
            "MissingOutcomeLinks"
        } else if missing_counterfactual_rows > 0 {
            "MissingCounterfactuals"
        } else {
            "NoPrimaryBottleneck"
        }
        .to_string(),
    );
    let closure_status = if krx_official_ready_candles == 0 {
        KRXOutcomeLinkClosureStatus::StillMissingOfficialCandles
    } else if generated_outcome_links == 0 && missing_future_window_rows > 0 {
        KRXOutcomeLinkClosureStatus::StillMissingFutureWindows
    } else if generated_outcome_links == 0 {
        KRXOutcomeLinkClosureStatus::StillMissingOutcomeLinks
    } else if generated_no_trade_counterfactuals == 0 && config.run_counterfactual_completion_v2 {
        KRXOutcomeLinkClosureStatus::StillMissingCounterfactuals
    } else if complete_krx_rows > 0 {
        KRXOutcomeLinkClosureStatus::KRXCompleteRowsImproved
    } else if generated_risk_denied_counterfactuals > 0 || generated_no_trade_counterfactuals > 0 {
        KRXOutcomeLinkClosureStatus::KRXCounterfactualsImproved
    } else if generated_outcome_links > 0 {
        KRXOutcomeLinkClosureStatus::KRXOutcomeLinksImproved
    } else if krx_official_rows == 0 {
        KRXOutcomeLinkClosureStatus::StillNeedMoreKRXRows
    } else {
        KRXOutcomeLinkClosureStatus::NoImprovement
    };
    let final_recommendation = if krx_official_ready_candles == 0 {
        KRXOutcomeLinkClosureRecommendation::MoreKRXOfficialRows
    } else if generated_outcome_links == 0 && missing_future_window_rows > 0 {
        KRXOutcomeLinkClosureRecommendation::CollectLongerKRXWindow
    } else if supplemental.unique_outcomes.len() < 2 && generated_outcome_links > 0 {
        KRXOutcomeLinkClosureRecommendation::MoreOutcomeDiversity
    } else if generated_risk_denied_counterfactuals == 0
        && config.run_counterfactual_completion_v2
        && supplemental.risk_rows > 0
    {
        KRXOutcomeLinkClosureRecommendation::MoreCounterfactualDepth
    } else if config.run_official_evidence_diversity_sweep {
        KRXOutcomeLinkClosureRecommendation::RunDiversitySweep
    } else if config.run_core_performance {
        KRXOutcomeLinkClosureRecommendation::RunCorePerformance
    } else if generated_outcome_links > 0 {
        KRXOutcomeLinkClosureRecommendation::KeepTrinity
    } else {
        KRXOutcomeLinkClosureRecommendation::NeedMoreEvidence
    };
    let mut reason_codes = config.reason_codes.clone();
    if generated_outcome_links > 0 {
        reason_codes.push(ReasonCode::CommitteeOutcomeLinked);
    } else {
        reason_codes.push(ReasonCode::OutcomeNoData);
    }
    if generated_no_trade_counterfactuals > 0 {
        reason_codes.push(ReasonCode::NoTradeCounterfactual);
    }
    if generated_risk_denied_counterfactuals > 0 {
        reason_codes.push(ReasonCode::RiskDeniedCounterfactual);
    }
    if complete_krx_rows > 0 {
        reason_codes.push(ReasonCode::CounterfactualEvaluated);
    }
    Ok(KRXOutcomeLinkClosureReport {
        closure_id: config.closure_id.clone(),
        krx_official_rows,
        krx_official_ready_candles,
        generated_outcome_links,
        generated_no_trade_counterfactuals,
        generated_risk_denied_counterfactuals,
        complete_krx_rows,
        missing_future_window_rows,
        missing_outcome_rows,
        missing_counterfactual_rows,
        bottleneck_changed: previous_primary_bottleneck != current_primary_bottleneck,
        previous_core_status,
        current_core_status,
        previous_primary_bottleneck,
        current_primary_bottleneck,
        closure_status,
        final_recommendation,
        reason_codes: stable_reason_codes(&reason_codes),
    })
}

fn load_sufficiency(
    config: &KRXOutcomeLinkClosureConfig,
) -> Result<KRXCandleSufficiencyReport, String> {
    if let Some(path) = config.krx_candle_sufficiency_paths.first() {
        return KRXCandleSufficiencyReport::from_json_path(Path::new(path));
    }
    if !config.krx_canonical_csv_paths.is_empty() {
        return Ok(KRXCandleSufficiencyReport::build(
            &format!("{}-sufficiency", config.closure_id),
            &config.krx_canonical_csv_paths,
            config.barrier_profile_registry_path.as_deref(),
        ));
    }
    Ok(KRXCandleSufficiencyReport {
        report_id: format!("{}-sufficiency", config.closure_id),
        items: Vec::new(),
        total_series: 0,
        official_ready_series: 0,
        benchmark_ready_series: 0,
        series_with_sufficient_future_window: 0,
        series_missing_future_window: 0,
        sufficiency_status: KRXCandleSufficiencyStatus::DiagnosticOnly,
        reason_codes: vec![ReasonCode::DeterministicPath],
    })
}

fn load_supplemental_counts(paths: &[String]) -> SupplementalCounts {
    let mut counts = SupplementalCounts::default();
    for path in paths {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&text) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let rows = value
            .get("rows")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let lower_path = path.to_ascii_lowercase();
        if lower_path.contains("risk") {
            counts.risk_rows += rows.len();
        }
        if lower_path.contains("baseline") {
            counts.baseline_rows += rows.len();
        }
        for row in rows {
            if let Some(outcome) = row.get("outcome").and_then(Value::as_str) {
                counts.unique_outcomes.insert(outcome.to_string());
            }
            if row.get("risk_decision").is_some() {
                counts.risk_rows += 1;
            }
            if row.get("baseline").is_some() {
                counts.baseline_rows += 1;
            }
        }
    }
    counts
}

fn default_output_root() -> String {
    "target/soma_krx_collection_closure".to_string()
}

fn default_true() -> bool {
    true
}
