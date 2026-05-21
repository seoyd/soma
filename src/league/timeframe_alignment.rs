use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::official_candle_coverage_pack::{normalize_timeframe_label, timeframe_seconds};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeframeAlignmentStatus {
    ExactMatch,
    CompatibleAggregation,
    CompatibleDownsampleDiagnosticOnly,
    IncompatibleUpsample,
    IncompatibleMixedGranularity,
    MissingScenarioTimeframe,
    MissingCandleTimeframe,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeframeAlignmentRecord {
    pub scenario_row_id: String,
    pub scenario_timeframe: String,
    pub candle_series_id: String,
    pub candle_timeframe: String,
    pub status: TimeframeAlignmentStatus,
    pub aggregation_required: bool,
    pub aggregation_allowed: bool,
    pub diagnostic_only: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeframeAlignmentOverallStatus {
    HealthyTimeframeAlignment,
    NeedsAggregationPermission,
    IncompatibleTimeframes,
    DiagnosticOnly,
    #[default]
    InsufficientTimeframeMetadata,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimeframeAlignmentReport {
    pub records: Vec<TimeframeAlignmentRecord>,
    pub exact_match_count: usize,
    pub compatible_aggregation_count: usize,
    pub incompatible_count: usize,
    pub diagnostic_only_count: usize,
    pub alignment_status: TimeframeAlignmentOverallStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeframeAlignmentInput {
    pub scenario_row_id: String,
    pub scenario_timeframe: String,
    pub candle_series_id: String,
    pub candle_timeframe: String,
}

pub fn build_timeframe_alignment_report(
    inputs: &[TimeframeAlignmentInput],
    allow_aggregation: bool,
    allow_downsample_diagnostic: bool,
) -> TimeframeAlignmentReport {
    let mut records = inputs
        .iter()
        .map(|input| build_record(input, allow_aggregation, allow_downsample_diagnostic))
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        left.scenario_row_id
            .cmp(&right.scenario_row_id)
            .then(left.candle_series_id.cmp(&right.candle_series_id))
            .then(left.scenario_timeframe.cmp(&right.scenario_timeframe))
            .then(left.candle_timeframe.cmp(&right.candle_timeframe))
    });
    let exact_match_count = records
        .iter()
        .filter(|record| record.status == TimeframeAlignmentStatus::ExactMatch)
        .count();
    let compatible_aggregation_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.status,
                TimeframeAlignmentStatus::CompatibleAggregation
                    | TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly
            )
        })
        .count();
    let incompatible_count = records
        .iter()
        .filter(|record| {
            matches!(
                record.status,
                TimeframeAlignmentStatus::IncompatibleUpsample
                    | TimeframeAlignmentStatus::IncompatibleMixedGranularity
            )
        })
        .count();
    let diagnostic_only_count = records
        .iter()
        .filter(|record| record.diagnostic_only)
        .count();
    let alignment_status = determine_overall_status(&records);
    TimeframeAlignmentReport {
        records,
        exact_match_count,
        compatible_aggregation_count,
        incompatible_count,
        diagnostic_only_count,
        alignment_status,
        reason_codes: stable_reason_codes(&[ReasonCode::CandleAlignmentBuilt]),
    }
}

impl TimeframeAlignmentReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("exact_match_count={}", self.exact_match_count),
            format!(
                "compatible_aggregation_count={}",
                self.compatible_aggregation_count
            ),
            format!("incompatible_count={}", self.incompatible_count),
            format!("diagnostic_only_count={}", self.diagnostic_only_count),
            format!("alignment_status={:?}", self.alignment_status),
        ];
        lines.extend(self.records.iter().map(|record| {
            format!(
                "scenario_row_id={};scenario_timeframe={};candle_series_id={};candle_timeframe={};status={:?};aggregation_required={};aggregation_allowed={};diagnostic_only={}",
                record.scenario_row_id,
                record.scenario_timeframe,
                record.candle_series_id,
                record.candle_timeframe,
                record.status,
                record.aggregation_required,
                record.aggregation_allowed,
                record.diagnostic_only,
            )
        }));
        lines.join("\n")
    }
}

fn build_record(
    input: &TimeframeAlignmentInput,
    allow_aggregation: bool,
    allow_downsample_diagnostic: bool,
) -> TimeframeAlignmentRecord {
    let scenario_timeframe = normalize_timeframe_label(&input.scenario_timeframe);
    let candle_timeframe = normalize_timeframe_label(&input.candle_timeframe);
    let mut reason_codes = Vec::new();
    let (status, aggregation_required, aggregation_allowed, diagnostic_only) = match (
        timeframe_seconds(&scenario_timeframe),
        timeframe_seconds(&candle_timeframe),
    ) {
        (None, _) if scenario_timeframe.is_empty() || scenario_timeframe == "unknown" => {
            reason_codes.push(ReasonCode::UnsupportedTimeframe);
            (
                TimeframeAlignmentStatus::MissingScenarioTimeframe,
                false,
                false,
                true,
            )
        }
        (_, None) if candle_timeframe.is_empty() || candle_timeframe == "unknown" => {
            reason_codes.push(ReasonCode::UnsupportedTimeframe);
            (
                TimeframeAlignmentStatus::MissingCandleTimeframe,
                false,
                false,
                true,
            )
        }
        (Some(scenario_seconds), Some(candle_seconds)) if scenario_seconds == candle_seconds => {
            (TimeframeAlignmentStatus::ExactMatch, false, true, false)
        }
        (Some(scenario_seconds), Some(candle_seconds))
            if scenario_seconds > candle_seconds && scenario_seconds % candle_seconds == 0 =>
        {
            (
                TimeframeAlignmentStatus::CompatibleAggregation,
                true,
                allow_aggregation,
                !allow_aggregation,
            )
        }
        (Some(scenario_seconds), Some(candle_seconds))
            if scenario_seconds < candle_seconds && candle_seconds % scenario_seconds == 0 =>
        {
            if allow_downsample_diagnostic {
                (
                    TimeframeAlignmentStatus::CompatibleDownsampleDiagnosticOnly,
                    false,
                    false,
                    true,
                )
            } else {
                (
                    TimeframeAlignmentStatus::IncompatibleUpsample,
                    false,
                    false,
                    true,
                )
            }
        }
        (Some(_), Some(_)) => (
            TimeframeAlignmentStatus::IncompatibleMixedGranularity,
            false,
            false,
            true,
        ),
        (None, _) => (
            TimeframeAlignmentStatus::MissingScenarioTimeframe,
            false,
            false,
            true,
        ),
        (_, None) => (
            TimeframeAlignmentStatus::MissingCandleTimeframe,
            false,
            false,
            true,
        ),
    };
    TimeframeAlignmentRecord {
        scenario_row_id: input.scenario_row_id.clone(),
        scenario_timeframe,
        candle_series_id: input.candle_series_id.clone(),
        candle_timeframe,
        status,
        aggregation_required,
        aggregation_allowed,
        diagnostic_only,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn determine_overall_status(
    records: &[TimeframeAlignmentRecord],
) -> TimeframeAlignmentOverallStatus {
    if records.is_empty() {
        return TimeframeAlignmentOverallStatus::InsufficientTimeframeMetadata;
    }
    if records.iter().any(|record| {
        matches!(
            record.status,
            TimeframeAlignmentStatus::MissingScenarioTimeframe
                | TimeframeAlignmentStatus::MissingCandleTimeframe
                | TimeframeAlignmentStatus::Unknown
        )
    }) {
        return TimeframeAlignmentOverallStatus::InsufficientTimeframeMetadata;
    }
    if records.iter().any(|record| {
        matches!(
            record.status,
            TimeframeAlignmentStatus::IncompatibleUpsample
                | TimeframeAlignmentStatus::IncompatibleMixedGranularity
        )
    }) {
        return TimeframeAlignmentOverallStatus::IncompatibleTimeframes;
    }
    if records.iter().any(|record| {
        record.status == TimeframeAlignmentStatus::CompatibleAggregation
            && !record.aggregation_allowed
    }) {
        return TimeframeAlignmentOverallStatus::NeedsAggregationPermission;
    }
    if records.iter().any(|record| record.diagnostic_only) {
        return TimeframeAlignmentOverallStatus::DiagnosticOnly;
    }
    TimeframeAlignmentOverallStatus::HealthyTimeframeAlignment
}
