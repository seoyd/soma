use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::experiment::{CoreCheckedBenchmarkReport, VenueCoverageExpansionReport, VenueGroup};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialEvidenceDelta {
    pub comparable: bool,
    pub previous_ready_datasets: usize,
    pub current_ready_datasets: usize,
    pub added_ready_datasets: isize,
    pub previous_outcome_records: usize,
    pub current_outcome_records: usize,
    pub added_outcome_records: isize,
    pub previous_venue_coverage: Vec<String>,
    pub current_venue_coverage: Vec<String>,
    pub added_venues: Vec<String>,
    pub previous_status: Option<String>,
    pub current_status: String,
    pub improvement_detected: bool,
    pub regressions: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl OfficialEvidenceDelta {
    pub fn to_text(&self) -> String {
        [
            format!("comparable={}", self.comparable),
            format!("previous_ready_datasets={}", self.previous_ready_datasets),
            format!("current_ready_datasets={}", self.current_ready_datasets),
            format!("added_ready_datasets={}", self.added_ready_datasets),
            format!("previous_outcome_records={}", self.previous_outcome_records),
            format!("current_outcome_records={}", self.current_outcome_records),
            format!("added_outcome_records={}", self.added_outcome_records),
            format!(
                "previous_venue_coverage={}",
                self.previous_venue_coverage.join("|")
            ),
            format!(
                "current_venue_coverage={}",
                self.current_venue_coverage.join("|")
            ),
            format!("added_venues={}", self.added_venues.join("|")),
            format!(
                "previous_status={}",
                self.previous_status.clone().unwrap_or_default()
            ),
            format!("current_status={}", self.current_status),
            format!("improvement_detected={}", self.improvement_detected),
            format!("regressions={}", self.regressions.join(" | ")),
        ]
        .join("\n")
    }
}

pub fn build_official_evidence_delta(
    previous_report: Option<&CoreCheckedBenchmarkReport>,
    current_report: Option<&CoreCheckedBenchmarkReport>,
    coverage_report: &VenueCoverageExpansionReport,
) -> OfficialEvidenceDelta {
    let previous_ready_datasets = previous_report
        .map(core_ready_dataset_count)
        .unwrap_or_default();
    let current_ready_datasets = current_report
        .map(core_ready_dataset_count)
        .unwrap_or_default();
    let previous_outcome_records = previous_report.map(core_outcome_count).unwrap_or_default();
    let current_outcome_records = current_report.map(core_outcome_count).unwrap_or_default();
    let previous_venue_coverage = previous_report.map(core_venue_coverage).unwrap_or_default();
    let current_venue_coverage = venue_coverage_from_report(coverage_report);
    let added_venues = current_venue_coverage
        .iter()
        .filter(|venue| !previous_venue_coverage.contains(*venue))
        .cloned()
        .collect::<Vec<_>>();
    let mut regressions = Vec::new();
    if let (Some(previous), Some(current)) = (previous_report, current_report) {
        if current
            .calibration_report
            .as_ref()
            .zip(previous.calibration_report.as_ref())
            .is_some_and(|(current, previous)| current.avg_brier_score > previous.avg_brier_score)
        {
            regressions.push("calibration worsened".to_string());
        }
        if current
            .model_comparison_report
            .as_ref()
            .zip(previous.model_comparison_report.as_ref())
            .is_some_and(|(current, previous)| {
                current.avg_delta_max_drawdown_pct > previous.avg_delta_max_drawdown_pct
            })
        {
            regressions.push("drawdown comparison worsened".to_string());
        }
        if current
            .risk_ai_interaction_report
            .as_ref()
            .zip(previous.risk_ai_interaction_report.as_ref())
            .is_some_and(|(current, previous)| current.denial_rate > previous.denial_rate + 0.05)
        {
            regressions.push("risk denial rate worsened".to_string());
        }
    }
    let improvement_detected = !added_venues.is_empty()
        || current_ready_datasets > previous_ready_datasets
        || current_outcome_records > previous_outcome_records;

    OfficialEvidenceDelta {
        comparable: previous_report.is_some(),
        previous_ready_datasets,
        current_ready_datasets,
        added_ready_datasets: current_ready_datasets as isize - previous_ready_datasets as isize,
        previous_outcome_records,
        current_outcome_records,
        added_outcome_records: current_outcome_records as isize - previous_outcome_records as isize,
        previous_venue_coverage,
        current_venue_coverage: current_venue_coverage.clone(),
        added_venues,
        previous_status: previous_report.map(|report| format!("{:?}", report.final_status)),
        current_status: current_report
            .map(|report| format!("{:?}", report.final_status))
            .unwrap_or_else(|| format!("{:?}", coverage_report.coverage_status)),
        improvement_detected,
        regressions,
        reason_codes: vec![ReasonCode::OfficialEvidenceDeltaBuilt],
    }
}

fn core_ready_dataset_count(report: &CoreCheckedBenchmarkReport) -> usize {
    report
        .dataset_selection
        .as_ref()
        .map(|selection| selection.selected_entries.len())
        .unwrap_or_default()
}

fn core_outcome_count(report: &CoreCheckedBenchmarkReport) -> usize {
    report
        .dataset_bundle
        .as_ref()
        .map(|bundle| bundle.label_counts.values().sum())
        .or_else(|| {
            report
                .baseline_report
                .as_ref()
                .map(|summary| summary.total_trades)
        })
        .unwrap_or_default()
}

fn core_venue_coverage(report: &CoreCheckedBenchmarkReport) -> Vec<String> {
    let mut venues = Vec::new();
    if report
        .dataset_selection
        .as_ref()
        .is_some_and(|selection| !selection.crypto_entries.is_empty())
    {
        venues.push("Crypto".to_string());
    }
    if report
        .dataset_selection
        .as_ref()
        .is_some_and(|selection| !selection.korean_equity_entries.is_empty())
    {
        venues.push("KoreanEquity".to_string());
    }
    if report
        .dataset_selection
        .as_ref()
        .is_some_and(|selection| !selection.us_equity_entries.is_empty())
    {
        venues.push("USEquity".to_string());
    }
    venues
}

fn venue_coverage_from_report(report: &VenueCoverageExpansionReport) -> Vec<String> {
    report
        .target_results
        .iter()
        .filter(|result| result.ready_datasets > 0)
        .map(|result| match result.venue_group {
            VenueGroup::Crypto => "Crypto".to_string(),
            VenueGroup::KoreanEquity => "KoreanEquity".to_string(),
            VenueGroup::USEquity => "USEquity".to_string(),
        })
        .collect()
}
