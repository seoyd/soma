use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};

use super::kis_candle_sufficiency::{KISCandleSufficiencyReport, KISCandleSufficiencyStatus};
use super::kis_symbol_whitelist::KISMarket;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KISOutcomeLinkClosureConfig {
    pub closure_id: String,
    #[serde(default)]
    pub kis_activation_report_paths: Vec<String>,
    #[serde(default)]
    pub kis_canonical_csv_paths: Vec<String>,
    #[serde(default)]
    pub kis_candle_sufficiency_paths: Vec<String>,
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
pub enum KISOutcomeLinkClosureStatus {
    KISOutcomeLinksImproved,
    KISCounterfactualsImproved,
    KISCompleteRowsImproved,
    StillMissingOfficialCandles,
    StillMissingFutureWindows,
    StillMissingOutcomeLinks,
    StillMissingCounterfactuals,
    StillNeedMoreKISRows,
    EndpointPolicyBlocked,
    NoImprovement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KISOutcomeLinkClosureRecommendation {
    CollectLongerKISWindow,
    MoreKISOfficialRows,
    MoreOutcomeDiversity,
    MoreCounterfactualDepth,
    RunDiversitySweep,
    RunCorePerformance,
    KeepTrinity,
    NeedMoreEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KISOutcomeLinkClosureReport {
    pub closure_id: String,
    pub kis_official_rows: usize,
    pub kis_official_ready_candles: usize,
    pub generated_outcome_links: usize,
    pub generated_no_trade_counterfactuals: usize,
    pub generated_risk_denied_counterfactuals: usize,
    pub complete_kis_rows: usize,
    pub domestic_complete_rows: usize,
    pub overseas_complete_rows: usize,
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
    pub closure_status: KISOutcomeLinkClosureStatus,
    pub final_recommendation: KISOutcomeLinkClosureRecommendation,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KISOutcomeLinkClosureRunner;

#[derive(Default)]
struct SupplementalCounts {
    risk_rows: usize,
    baseline_rows: usize,
    unique_outcomes: BTreeSet<String>,
}

impl Default for KISOutcomeLinkClosureConfig {
    fn default() -> Self {
        Self {
            closure_id: "kis_outcome_link_closure".to_string(),
            kis_activation_report_paths: Vec::new(),
            kis_canonical_csv_paths: Vec::new(),
            kis_candle_sufficiency_paths: Vec::new(),
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

impl KISOutcomeLinkClosureConfig {
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
        .chain(self.kis_activation_report_paths.iter().map(String::as_str))
        .chain(self.kis_canonical_csv_paths.iter().map(String::as_str))
        .chain(self.kis_candle_sufficiency_paths.iter().map(String::as_str))
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
            return Err("kis outcome closure id must not be empty".to_string());
        }
        if !self.validate_local_paths().is_empty() {
            return Err("kis outcome closure paths must be local".to_string());
        }
        Ok(())
    }
}

impl KISOutcomeLinkClosureRunner {
    pub fn run(
        &self,
        config: &KISOutcomeLinkClosureConfig,
    ) -> Result<KISOutcomeLinkClosureReport, String> {
        config.validate()?;
        let report = build_outcome_link_closure_report(config)?;
        let output_dir = config.output_dir();
        report.write_to_dir(&output_dir)?;
        Ok(report)
    }
}

impl KISOutcomeLinkClosureReport {
    pub fn to_text(&self) -> String {
        [
            format!("closure_id={}", self.closure_id),
            format!("kis_official_rows={}", self.kis_official_rows),
            format!(
                "kis_official_ready_candles={}",
                self.kis_official_ready_candles
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
            format!("complete_kis_rows={}", self.complete_kis_rows),
            format!("domestic_complete_rows={}", self.domestic_complete_rows),
            format!("overseas_complete_rows={}", self.overseas_complete_rows),
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
            output_dir.join("kis_outcome_link_closure.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("kis_outcome_link_closure.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(())
    }
}

pub fn build_outcome_link_closure_report(
    config: &KISOutcomeLinkClosureConfig,
) -> Result<KISOutcomeLinkClosureReport, String> {
    let sufficiency = load_sufficiency(config)?;
    let supplemental = load_supplemental_counts(&config.official_ready_rows_paths);
    let kis_official_rows = sufficiency
        .items
        .iter()
        .filter(|item| item.official_ready)
        .map(|item| item.row_count)
        .sum::<usize>();
    let kis_official_ready_candles = sufficiency
        .items
        .iter()
        .filter(|item| item.official_ready)
        .map(|item| item.row_count)
        .sum::<usize>();
    let generated_outcome_links = if config.run_outcome_linkage_v3 {
        sufficiency
            .items
            .iter()
            .filter(|item| item.benchmark_ready && item.no_lookahead_safe)
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
    let complete_kis_rows = if config.run_complete_row_close_v2 {
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
    let (domestic_complete_rows, overseas_complete_rows) =
        allocate_complete_rows(&sufficiency, complete_kis_rows);
    let missing_future_window_rows = if config.run_future_window_requirements {
        sufficiency
            .items
            .iter()
            .map(|item| item.missing_future_bars.unwrap_or(0).min(item.row_count))
            .sum::<usize>()
    } else {
        0
    };
    let missing_outcome_rows = kis_official_ready_candles.saturating_sub(generated_outcome_links);
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
        } else if complete_kis_rows == 0 {
            "CoreBlockedByCounterfactuals"
        } else {
            "CoreResearchReady"
        }
        .to_string(),
    );
    let previous_primary_bottleneck = Some("MissingOfficialCandles".to_string());
    let current_primary_bottleneck = Some(
        if matches!(
            sufficiency.sufficiency_status,
            KISCandleSufficiencyStatus::EndpointPolicyBlocked
        ) {
            "EndpointPolicyBlocked"
        } else if kis_official_ready_candles == 0 {
            "MissingOfficialCandles"
        } else if generated_outcome_links == 0 && missing_future_window_rows > 0 {
            "MissingFutureWindows"
        } else if generated_outcome_links == 0 {
            "MissingOutcomeLinks"
        } else if complete_kis_rows == 0 {
            "MissingCounterfactuals"
        } else {
            "NeedMoreOutcomeDiversity"
        }
        .to_string(),
    );
    let closure_status = if matches!(
        sufficiency.sufficiency_status,
        KISCandleSufficiencyStatus::EndpointPolicyBlocked
    ) {
        KISOutcomeLinkClosureStatus::EndpointPolicyBlocked
    } else if kis_official_ready_candles == 0 {
        KISOutcomeLinkClosureStatus::StillMissingOfficialCandles
    } else if missing_future_window_rows > 0 && generated_outcome_links == 0 {
        KISOutcomeLinkClosureStatus::StillMissingFutureWindows
    } else if generated_outcome_links == 0 {
        KISOutcomeLinkClosureStatus::StillMissingOutcomeLinks
    } else if complete_kis_rows == 0 {
        KISOutcomeLinkClosureStatus::StillMissingCounterfactuals
    } else if generated_risk_denied_counterfactuals > 0 {
        KISOutcomeLinkClosureStatus::KISCompleteRowsImproved
    } else if generated_no_trade_counterfactuals > 0 {
        KISOutcomeLinkClosureStatus::KISCounterfactualsImproved
    } else if generated_outcome_links > 0 {
        KISOutcomeLinkClosureStatus::KISOutcomeLinksImproved
    } else if kis_official_rows == 0 {
        KISOutcomeLinkClosureStatus::StillNeedMoreKISRows
    } else {
        KISOutcomeLinkClosureStatus::NoImprovement
    };
    let final_recommendation = match closure_status {
        KISOutcomeLinkClosureStatus::EndpointPolicyBlocked => {
            KISOutcomeLinkClosureRecommendation::NeedMoreEvidence
        }
        KISOutcomeLinkClosureStatus::StillMissingOfficialCandles => {
            KISOutcomeLinkClosureRecommendation::MoreKISOfficialRows
        }
        KISOutcomeLinkClosureStatus::StillMissingFutureWindows => {
            KISOutcomeLinkClosureRecommendation::CollectLongerKISWindow
        }
        KISOutcomeLinkClosureStatus::StillMissingOutcomeLinks => {
            KISOutcomeLinkClosureRecommendation::CollectLongerKISWindow
        }
        KISOutcomeLinkClosureStatus::StillMissingCounterfactuals => {
            KISOutcomeLinkClosureRecommendation::MoreCounterfactualDepth
        }
        KISOutcomeLinkClosureStatus::StillNeedMoreKISRows => {
            KISOutcomeLinkClosureRecommendation::MoreKISOfficialRows
        }
        KISOutcomeLinkClosureStatus::KISOutcomeLinksImproved
            if supplemental.unique_outcomes.len() < 2 =>
        {
            KISOutcomeLinkClosureRecommendation::MoreOutcomeDiversity
        }
        KISOutcomeLinkClosureStatus::KISOutcomeLinksImproved
        | KISOutcomeLinkClosureStatus::KISCounterfactualsImproved
        | KISOutcomeLinkClosureStatus::KISCompleteRowsImproved
            if config.run_official_evidence_diversity_sweep =>
        {
            KISOutcomeLinkClosureRecommendation::RunDiversitySweep
        }
        KISOutcomeLinkClosureStatus::KISOutcomeLinksImproved
        | KISOutcomeLinkClosureStatus::KISCounterfactualsImproved
        | KISOutcomeLinkClosureStatus::KISCompleteRowsImproved
            if config.run_core_performance =>
        {
            KISOutcomeLinkClosureRecommendation::RunCorePerformance
        }
        KISOutcomeLinkClosureStatus::KISOutcomeLinksImproved
        | KISOutcomeLinkClosureStatus::KISCounterfactualsImproved
        | KISOutcomeLinkClosureStatus::KISCompleteRowsImproved => {
            KISOutcomeLinkClosureRecommendation::KeepTrinity
        }
        KISOutcomeLinkClosureStatus::NoImprovement => {
            KISOutcomeLinkClosureRecommendation::NeedMoreEvidence
        }
    };
    let reason_codes = stable_reason_codes(
        &[
            vec![ReasonCode::KISOutcomeLinkClosureBuilt],
            sufficiency.reason_codes.clone(),
            if generated_outcome_links == 0 {
                vec![ReasonCode::MissingOfficialData]
            } else {
                vec![]
            },
            if missing_future_window_rows > 0 {
                vec![ReasonCode::InsufficientBars]
            } else {
                vec![]
            },
        ]
        .concat(),
    );
    Ok(KISOutcomeLinkClosureReport {
        closure_id: config.closure_id.clone(),
        kis_official_rows,
        kis_official_ready_candles,
        generated_outcome_links,
        generated_no_trade_counterfactuals,
        generated_risk_denied_counterfactuals,
        complete_kis_rows,
        domestic_complete_rows,
        overseas_complete_rows,
        missing_future_window_rows,
        missing_outcome_rows,
        missing_counterfactual_rows,
        previous_core_status,
        current_core_status,
        previous_primary_bottleneck,
        current_primary_bottleneck,
        bottleneck_changed: true,
        closure_status,
        final_recommendation,
        reason_codes,
    })
}

fn allocate_complete_rows(
    sufficiency: &KISCandleSufficiencyReport,
    complete_rows: usize,
) -> (usize, usize) {
    let domestic_capacity = sufficiency
        .items
        .iter()
        .filter(|item| item.market == KISMarket::KoreanEquity && item.benchmark_ready)
        .map(|item| {
            item.row_count
                .saturating_sub(item.required_future_bars.unwrap_or_default())
        })
        .sum::<usize>();
    let domestic_complete_rows = complete_rows.min(domestic_capacity);
    let overseas_complete_rows = complete_rows.saturating_sub(domestic_complete_rows);
    (domestic_complete_rows, overseas_complete_rows)
}

fn load_sufficiency(
    config: &KISOutcomeLinkClosureConfig,
) -> Result<KISCandleSufficiencyReport, String> {
    if let Some(path) = config.kis_candle_sufficiency_paths.first() {
        return KISCandleSufficiencyReport::from_json_path(Path::new(path));
    }
    if !config.kis_canonical_csv_paths.is_empty() {
        return Ok(KISCandleSufficiencyReport::build(
            &format!("{}-derived-sufficiency", config.closure_id),
            &config.kis_canonical_csv_paths,
            config.barrier_profile_registry_path.as_deref(),
        ));
    }
    Err(
        "kis outcome closure requires a local candle sufficiency report or canonical csv path"
            .to_string(),
    )
}

fn load_supplemental_counts(paths: &[String]) -> SupplementalCounts {
    let mut counts = SupplementalCounts::default();
    for path in paths {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let rows = value
            .get("rows")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_else(|| value.as_array().cloned().unwrap_or_default());
        if path.contains("risk") {
            counts.risk_rows += rows.len();
        }
        if path.contains("baseline") {
            counts.baseline_rows += rows.len();
        }
        for row in rows {
            if let Some(outcome) = row.get("outcome").and_then(Value::as_str) {
                counts.unique_outcomes.insert(outcome.to_string());
            }
        }
    }
    counts
}

fn default_output_root() -> String {
    "target/soma_kis_market_data_activation".to_string()
}

fn default_true() -> bool {
    true
}
