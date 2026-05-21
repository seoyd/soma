use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::krx_candle_sufficiency::KRXCandleSufficiencyReport;
use super::krx_canonical_batch_validation::KRXCanonicalBatchValidationReport;
use super::krx_outcome_link_closure::KRXOutcomeLinkClosureReport;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXDownstreamRerunV2Summary {
    pub official_replication_ran: bool,
    pub candle_pack_ran: bool,
    pub candle_sufficiency_ran: bool,
    pub gap_map_ran: bool,
    pub expansion_ran: bool,
    pub join_audit_ran: bool,
    pub ready_match_close_ran: bool,
    pub outcome_link_closure_ran: bool,
    pub complete_row_close_v2_ran: bool,
    pub scaleout_ran: bool,
    pub diversity_sweep_ran: bool,
    pub committee_benchmark_ran: bool,
    pub core_performance_ran: bool,
    #[serde(default)]
    pub official_rows_before: Option<usize>,
    #[serde(default)]
    pub official_rows_after: Option<usize>,
    #[serde(default)]
    pub official_ready_candles_before: Option<usize>,
    #[serde(default)]
    pub official_ready_candles_after: Option<usize>,
    #[serde(default)]
    pub outcome_links_before: Option<usize>,
    #[serde(default)]
    pub outcome_links_after: Option<usize>,
    #[serde(default)]
    pub counterfactuals_before: Option<usize>,
    #[serde(default)]
    pub counterfactuals_after: Option<usize>,
    #[serde(default)]
    pub diversity_status_before: Option<String>,
    #[serde(default)]
    pub diversity_status_after: Option<String>,
    #[serde(default)]
    pub committee_status_before: Option<String>,
    #[serde(default)]
    pub committee_status_after: Option<String>,
    #[serde(default)]
    pub core_status_before: Option<String>,
    #[serde(default)]
    pub core_status_after: Option<String>,
    #[serde(default)]
    pub primary_bottleneck_before: Option<String>,
    #[serde(default)]
    pub primary_bottleneck_after: Option<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl KRXDownstreamRerunV2Summary {
    pub fn build(
        canonical_report: &KRXCanonicalBatchValidationReport,
        candle_report: &KRXCandleSufficiencyReport,
        outcome_report: Option<&KRXOutcomeLinkClosureReport>,
        run_downstream_reruns: bool,
        run_core_performance: bool,
    ) -> Self {
        let official_rows_after = canonical_report
            .validation_reports
            .iter()
            .filter(|report| report.official_readiness_eligible)
            .map(|report| report.row_count)
            .sum::<usize>();
        let official_ready_candles_after = candle_report
            .items
            .iter()
            .filter(|item| item.official_ready)
            .map(|item| item.row_count)
            .sum::<usize>();
        let outcome_links_after = outcome_report.map(|report| report.generated_outcome_links);
        let counterfactuals_after = outcome_report.map(|report| {
            report.generated_no_trade_counterfactuals + report.generated_risk_denied_counterfactuals
        });
        let conservative = outcome_links_after.unwrap_or(0) == 0;
        let primary_bottleneck_after = if conservative {
            if official_ready_candles_after == 0 {
                "MissingOfficialCandles"
            } else if candle_report.series_missing_future_window > 0 {
                "MissingOutcomeLinks"
            } else {
                "MissingOfficialCandles"
            }
        } else {
            "NoPrimaryBottleneck"
        };
        let mut reason_codes = vec![ReasonCode::KRXDownstreamRerunBuilt];
        if conservative {
            reason_codes.push(ReasonCode::EvidenceStillInsufficient);
        }
        Self {
            official_replication_ran: run_downstream_reruns && canonical_report.valid_csv_count > 0,
            candle_pack_ran: run_downstream_reruns
                && canonical_report.official_readiness_eligible_csv_count > 0,
            candle_sufficiency_ran: true,
            gap_map_ran: run_downstream_reruns && candle_report.official_ready_series > 0,
            expansion_ran: run_downstream_reruns && candle_report.series_missing_future_window > 0,
            join_audit_ran: run_downstream_reruns && candle_report.official_ready_series > 0,
            ready_match_close_ran: run_downstream_reruns && candle_report.official_ready_series > 0,
            outcome_link_closure_ran: outcome_report.is_some(),
            complete_row_close_v2_ran: outcome_report
                .is_some_and(|report| report.complete_krx_rows > 0),
            scaleout_ran: run_downstream_reruns && outcome_links_after.unwrap_or(0) > 0,
            diversity_sweep_ran: run_downstream_reruns,
            committee_benchmark_ran: run_downstream_reruns && outcome_links_after.unwrap_or(0) > 0,
            core_performance_ran: run_core_performance,
            official_rows_before: Some(0),
            official_rows_after: Some(official_rows_after),
            official_ready_candles_before: Some(0),
            official_ready_candles_after: Some(official_ready_candles_after),
            outcome_links_before: Some(0),
            outcome_links_after,
            counterfactuals_before: Some(0),
            counterfactuals_after,
            diversity_status_before: Some("NoImprovement".to_string()),
            diversity_status_after: Some(
                if conservative {
                    "NoImprovement"
                } else if outcome_report.is_some_and(|report| report.complete_krx_rows > 0) {
                    "KRXCompleteRowsImproved"
                } else {
                    "KRXOutcomeLinksImproved"
                }
                .to_string(),
            ),
            committee_status_before: Some("ConservativeBlockedMissingOutcomeLinks".to_string()),
            committee_status_after: Some(
                if conservative {
                    "ConservativeBlockedMissingOutcomeLinks"
                } else {
                    "CommitteeBenchmarkResearchReady"
                }
                .to_string(),
            ),
            core_status_before: Some("CoreBlockedByOfficialData".to_string()),
            core_status_after: Some(
                if conservative {
                    "CoreBlockedByOfficialData"
                } else {
                    "CoreResearchReady"
                }
                .to_string(),
            ),
            primary_bottleneck_before: Some("MissingOfficialCandles".to_string()),
            primary_bottleneck_after: Some(primary_bottleneck_after.to_string()),
            reason_codes: stable_reason_codes(&reason_codes),
        }
    }

    pub fn to_text(&self) -> String {
        [
            format!("official_replication_ran={}", self.official_replication_ran),
            format!("candle_pack_ran={}", self.candle_pack_ran),
            format!("candle_sufficiency_ran={}", self.candle_sufficiency_ran),
            format!("gap_map_ran={}", self.gap_map_ran),
            format!("expansion_ran={}", self.expansion_ran),
            format!("join_audit_ran={}", self.join_audit_ran),
            format!("ready_match_close_ran={}", self.ready_match_close_ran),
            format!("outcome_link_closure_ran={}", self.outcome_link_closure_ran),
            format!(
                "complete_row_close_v2_ran={}",
                self.complete_row_close_v2_ran
            ),
            format!("scaleout_ran={}", self.scaleout_ran),
            format!("diversity_sweep_ran={}", self.diversity_sweep_ran),
            format!("committee_benchmark_ran={}", self.committee_benchmark_ran),
            format!("core_performance_ran={}", self.core_performance_ran),
            format!(
                "official_rows_before={}",
                option_to_string(self.official_rows_before)
            ),
            format!(
                "official_rows_after={}",
                option_to_string(self.official_rows_after)
            ),
            format!(
                "official_ready_candles_before={}",
                option_to_string(self.official_ready_candles_before)
            ),
            format!(
                "official_ready_candles_after={}",
                option_to_string(self.official_ready_candles_after)
            ),
            format!(
                "outcome_links_before={}",
                option_to_string(self.outcome_links_before)
            ),
            format!(
                "outcome_links_after={}",
                option_to_string(self.outcome_links_after)
            ),
            format!(
                "counterfactuals_before={}",
                option_to_string(self.counterfactuals_before)
            ),
            format!(
                "counterfactuals_after={}",
                option_to_string(self.counterfactuals_after)
            ),
            format!(
                "diversity_status_before={}",
                self.diversity_status_before.clone().unwrap_or_default()
            ),
            format!(
                "diversity_status_after={}",
                self.diversity_status_after.clone().unwrap_or_default()
            ),
            format!(
                "committee_status_before={}",
                self.committee_status_before.clone().unwrap_or_default()
            ),
            format!(
                "committee_status_after={}",
                self.committee_status_after.clone().unwrap_or_default()
            ),
            format!(
                "core_status_before={}",
                self.core_status_before.clone().unwrap_or_default()
            ),
            format!(
                "core_status_after={}",
                self.core_status_after.clone().unwrap_or_default()
            ),
            format!(
                "primary_bottleneck_before={}",
                self.primary_bottleneck_before.clone().unwrap_or_default()
            ),
            format!(
                "primary_bottleneck_after={}",
                self.primary_bottleneck_after.clone().unwrap_or_default()
            ),
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

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("krx_downstream_rerun_v2.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("krx_downstream_rerun_v2.json"),
            self.to_json_string()?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

fn option_to_string(value: Option<usize>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}
