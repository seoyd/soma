use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KRXDownstreamRerunSummary {
    pub official_replication_ran: bool,
    pub candle_pack_ran: bool,
    pub ready_match_close_ran: bool,
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
    pub outcome_links_before: Option<usize>,
    #[serde(default)]
    pub outcome_links_after: Option<usize>,
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

impl Default for KRXDownstreamRerunSummary {
    fn default() -> Self {
        Self {
            official_replication_ran: false,
            candle_pack_ran: false,
            ready_match_close_ran: false,
            complete_row_close_v2_ran: false,
            scaleout_ran: false,
            diversity_sweep_ran: false,
            committee_benchmark_ran: false,
            core_performance_ran: false,
            official_rows_before: None,
            official_rows_after: None,
            outcome_links_before: None,
            outcome_links_after: None,
            diversity_status_before: None,
            diversity_status_after: None,
            committee_status_before: None,
            committee_status_after: None,
            core_status_before: None,
            core_status_after: None,
            primary_bottleneck_before: None,
            primary_bottleneck_after: None,
            reason_codes: vec![ReasonCode::KRXDownstreamRerunBuilt],
        }
    }
}

impl KRXDownstreamRerunSummary {
    pub fn to_text(&self) -> String {
        [
            format!("official_replication_ran={}", self.official_replication_ran),
            format!("candle_pack_ran={}", self.candle_pack_ran),
            format!("ready_match_close_ran={}", self.ready_match_close_ran),
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
                self.official_rows_before
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "official_rows_after={}",
                self.official_rows_after
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "outcome_links_before={}",
                self.outcome_links_before
                    .map(|value| value.to_string())
                    .unwrap_or_default()
            ),
            format!(
                "outcome_links_after={}",
                self.outcome_links_after
                    .map(|value| value.to_string())
                    .unwrap_or_default()
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

    pub fn finalize(&mut self, extra_reason_codes: &[ReasonCode]) {
        let mut reason_codes = self.reason_codes.clone();
        reason_codes.extend(extra_reason_codes.iter().cloned());
        self.reason_codes = stable_reason_codes(&reason_codes);
    }
}
