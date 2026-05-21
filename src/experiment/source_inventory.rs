use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{EvidenceSourceKind, MarketVenue};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceDatasetRecord {
    pub dataset_id: String,
    pub source_kind: EvidenceSourceKind,
    pub symbol: String,
    pub normalized_symbol: String,
    pub timeframe_label: String,
    #[serde(default)]
    pub venue: Option<MarketVenue>,
    #[serde(default)]
    pub canonical_csv_path: Option<String>,
    #[serde(default)]
    pub manifest_path: Option<String>,
    #[serde(default)]
    pub provenance_path: Option<String>,
    pub row_count: usize,
    pub ready_for_evidence: bool,
    pub benchmark_eligible: bool,
    #[serde(default)]
    pub adjusted_price_policy: Option<String>,
    #[serde(default)]
    pub data_quality_score: Option<f64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceKindDatasetInventory {
    pub official_datasets: Vec<SourceDatasetRecord>,
    pub yfinance_research_datasets: Vec<SourceDatasetRecord>,
    pub mock_datasets: Vec<SourceDatasetRecord>,
    pub synthetic_datasets: Vec<SourceDatasetRecord>,
    pub unknown_datasets: Vec<SourceDatasetRecord>,
    pub official_ready_count: usize,
    pub yfinance_benchmark_eligible_count: usize,
    pub readiness_eligible_count: usize,
    pub research_only_count: usize,
    pub by_symbol: BTreeMap<String, usize>,
    pub by_timeframe: BTreeMap<String, usize>,
    pub by_venue: BTreeMap<String, usize>,
    pub reason_codes: Vec<ReasonCode>,
}

pub fn build_source_kind_dataset_inventory(
    records: &[SourceDatasetRecord],
) -> SourceKindDatasetInventory {
    let mut official_datasets = Vec::new();
    let mut yfinance_research_datasets = Vec::new();
    let mut mock_datasets = Vec::new();
    let mut synthetic_datasets = Vec::new();
    let mut unknown_datasets = Vec::new();
    let mut official_ready_count = 0usize;
    let mut yfinance_benchmark_eligible_count = 0usize;
    let mut readiness_eligible_count = 0usize;
    let mut research_only_count = 0usize;
    let mut by_symbol = BTreeMap::new();
    let mut by_timeframe = BTreeMap::new();
    let mut by_venue = BTreeMap::new();

    let mut sorted = records.to_vec();
    sorted.sort_by(|left, right| {
        left.normalized_symbol
            .cmp(&right.normalized_symbol)
            .then(left.timeframe_label.cmp(&right.timeframe_label))
            .then(left.dataset_id.cmp(&right.dataset_id))
    });

    for record in sorted {
        *by_symbol
            .entry(record.normalized_symbol.clone())
            .or_insert(0usize) += 1;
        *by_timeframe
            .entry(record.timeframe_label.clone())
            .or_insert(0usize) += 1;
        *by_venue
            .entry(
                record
                    .venue
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_else(|| "Unknown".to_string()),
            )
            .or_insert(0usize) += 1;

        match record.source_kind {
            EvidenceSourceKind::OfficialApiCollected => {
                if record.ready_for_evidence {
                    official_ready_count += 1;
                    readiness_eligible_count += 1;
                }
                official_datasets.push(record);
            }
            EvidenceSourceKind::YFinanceResearch => {
                if record.benchmark_eligible {
                    yfinance_benchmark_eligible_count += 1;
                }
                research_only_count += 1;
                yfinance_research_datasets.push(record);
            }
            EvidenceSourceKind::TestFixture => {
                research_only_count += 1;
                mock_datasets.push(record);
            }
            EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::GeneratedSynthetic => {
                research_only_count += 1;
                synthetic_datasets.push(record);
            }
            EvidenceSourceKind::RealLocal | EvidenceSourceKind::ExternalPredictionOnly => {
                research_only_count += 1;
                unknown_datasets.push(record);
            }
            EvidenceSourceKind::Unknown => {
                research_only_count += 1;
                unknown_datasets.push(record);
            }
        }
    }

    SourceKindDatasetInventory {
        official_datasets,
        yfinance_research_datasets,
        mock_datasets,
        synthetic_datasets,
        unknown_datasets,
        official_ready_count,
        yfinance_benchmark_eligible_count,
        readiness_eligible_count,
        research_only_count,
        by_symbol,
        by_timeframe,
        by_venue,
        reason_codes: vec![ReasonCode::SourceInventoryBuilt],
    }
}
