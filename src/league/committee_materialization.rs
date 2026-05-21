use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, Regime};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::committee_artifact_resolver::{
    CommitteeArtifactDescriptor, CommitteeArtifactKind, CommitteeArtifactResolver,
};
use super::committee_scenario_loader::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioMaterializationLevel,
    CommitteeScenarioRow, CommitteeScenarioSet, CommitteeScenarioSourceKind,
};
use super::persona_card_lite::PersonaHorizon;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeMaterializationConfig {
    pub materialization_id: String,
    #[serde(default)]
    pub input_artifact_paths: Vec<String>,
    #[serde(default = "default_allowed_artifacts")]
    pub allowed_artifact_kinds: Vec<CommitteeArtifactKind>,
    pub output_root: String,
    #[serde(default = "default_max_rows")]
    pub max_rows: usize,
    #[serde(default = "default_max_symbols")]
    pub max_symbols: usize,
    #[serde(default = "default_max_bytes")]
    pub max_bytes: usize,
    #[serde(default)]
    pub allow_summary_derived_rows: bool,
    #[serde(default = "default_true")]
    pub prefer_row_level_artifacts: bool,
    #[serde(default = "default_true")]
    pub require_provenance: bool,
    #[serde(default = "default_min_quality")]
    pub min_data_quality: f64,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeScenarioMaterializerV2;

impl Default for CommitteeMaterializationConfig {
    fn default() -> Self {
        Self {
            materialization_id: "committee_materialization".to_string(),
            input_artifact_paths: Vec::new(),
            allowed_artifact_kinds: default_allowed_artifacts(),
            output_root: "target/soma_committee_materialization".to_string(),
            max_rows: default_max_rows(),
            max_symbols: default_max_symbols(),
            max_bytes: default_max_bytes(),
            allow_summary_derived_rows: true,
            prefer_row_level_artifacts: true,
            require_provenance: true,
            min_data_quality: default_min_quality(),
            reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
        }
    }
}

impl CommitteeMaterializationConfig {
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

    pub fn validate(&self) -> Result<(), String> {
        if self.output_root.contains("://")
            || self
                .input_artifact_paths
                .iter()
                .any(|path| path.contains("://"))
        {
            return Err("committee materialization paths must be local".to_string());
        }
        if self.max_rows == 0 || self.max_rows > default_max_rows() {
            return Err("committee materialization max_rows must be between 1 and 100".to_string());
        }
        if self.max_symbols == 0 || self.max_symbols > default_max_symbols() {
            return Err(
                "committee materialization max_symbols must be between 1 and 50".to_string(),
            );
        }
        if self.max_bytes == 0 || self.max_bytes > default_max_bytes() {
            return Err(
                "committee materialization max_bytes must be between 1 and 5000000".to_string(),
            );
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.materialization_id)
    }
}

impl CommitteeScenarioMaterializerV2 {
    pub fn materialize(
        &self,
        config: &CommitteeMaterializationConfig,
    ) -> Result<CommitteeScenarioSet, String> {
        config.validate()?;
        let resolver = CommitteeArtifactResolver;
        let mut rows = Vec::new();
        let mut total_bytes = 0usize;
        let mut seen_symbols = BTreeSet::new();
        let input_paths = if config.input_artifact_paths.is_empty() {
            default_virtual_inputs(config)
        } else {
            config.input_artifact_paths.clone()
        };

        for path in input_paths {
            let descriptor = resolver.resolve(&path);
            if !config
                .allowed_artifact_kinds
                .contains(&descriptor.artifact_kind)
                && descriptor.artifact_kind != CommitteeArtifactKind::Unknown
            {
                continue;
            }
            if let Ok(metadata) = fs::metadata(&path) {
                total_bytes += metadata.len() as usize;
                if total_bytes > config.max_bytes {
                    break;
                }
            }
            let materialized = materialize_descriptor(config, &descriptor)?;
            for row in materialized {
                if row.data_quality_score < config.min_data_quality {
                    continue;
                }
                if seen_symbols.len() >= config.max_symbols && !seen_symbols.contains(&row.symbol) {
                    continue;
                }
                seen_symbols.insert(row.symbol.clone());
                rows.push(row);
                if rows.len() >= config.max_rows {
                    break;
                }
            }
            if rows.len() >= config.max_rows {
                break;
            }
        }

        rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
        let mut sources = rows
            .iter()
            .map(|row| format!("{:?}", row.source_kind))
            .collect::<Vec<_>>();
        sources.sort();
        sources.dedup();
        Ok(CommitteeScenarioSet {
            scenario_id: config.materialization_id.clone(),
            row_count: rows.len(),
            official_row_count: rows
                .iter()
                .filter(|row| {
                    row.evidence_source_kind.readiness_eligible()
                        && row.materialization_level
                            == CommitteeScenarioMaterializationLevel::RowLevel
                })
                .count(),
            research_only_row_count: rows
                .iter()
                .filter(|row| row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch)
                .count(),
            fixture_row_count: rows
                .iter()
                .filter(|row| {
                    matches!(
                        row.source_kind,
                        CommitteeScenarioSourceKind::Fixture
                            | CommitteeScenarioSourceKind::SyntheticTest
                    )
                })
                .count(),
            skipped_row_count: 0,
            source_summary: sources.join("|"),
            rows,
            reason_codes: vec![ReasonCode::CommitteeMaterializationBuilt],
        })
    }
}

fn materialize_descriptor(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let official_like = matches!(
        descriptor.artifact_kind,
        CommitteeArtifactKind::CoreCheckedBenchmarkReport
            | CommitteeArtifactKind::OfficialBenchmarkReport
            | CommitteeArtifactKind::SourceAwareBenchmarkReport
    );
    if config.prefer_row_level_artifacts
        && descriptor.row_level_available
        && (!official_like || !config.require_provenance || descriptor.provenance_available)
    {
        let rows = try_materialize_row_level(config, descriptor)?;
        if !rows.is_empty() {
            return Ok(rows);
        }
    }
    if config.allow_summary_derived_rows {
        let mut rows = fallback_summary_rows(config, descriptor)?;
        for row in &mut rows {
            row.reason_codes
                .push(ReasonCode::CommitteeSummaryFallbackUsed);
        }
        return Ok(rows);
    }
    Ok(Vec::new())
}

fn try_materialize_row_level(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    match descriptor.artifact_kind {
        CommitteeArtifactKind::CanonicalOhlcvCsv => materialize_csv_rows(config, descriptor),
        CommitteeArtifactKind::CommitteeV1Bundle => materialize_v1_bundle_rows(config, descriptor),
        CommitteeArtifactKind::Unknown => Ok(Vec::new()),
        _ => materialize_json_rows(config, descriptor),
    }
}

fn materialize_v1_bundle_rows(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let text = fs::read_to_string(&descriptor.path).map_err(|err| err.to_string())?;
    let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    let rows = value
        .get("scenario_set")
        .and_then(|set| set.get("rows"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    materialize_rows_from_values(config, descriptor, rows)
}

fn materialize_json_rows(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let text = fs::read_to_string(&descriptor.path).map_err(|err| err.to_string())?;
    let value: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    let rows = [
        "rows",
        "records",
        "scenarios",
        "lane_reports",
        "symbols",
        "yfinance_symbols",
    ]
    .iter()
    .find_map(|key| value.get(key).and_then(Value::as_array).cloned())
    .unwrap_or_default();
    materialize_rows_from_values(config, descriptor, rows)
}

fn materialize_rows_from_values(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
    rows: Vec<Value>,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let mut materialized = Vec::new();
    for (index, row) in rows.into_iter().enumerate() {
        let symbol = row
            .get("symbol")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| row.as_str().map(str::to_string))
            .or_else(|| descriptor.symbol.clone())
            .unwrap_or_else(|| default_symbol(descriptor).to_string());
        let market = row
            .get("market")
            .and_then(Value::as_str)
            .map(parse_market)
            .or(descriptor.market)
            .unwrap_or_else(|| default_market(descriptor));
        let evidence_source_kind = default_evidence_source(descriptor);
        if config.require_provenance
            && evidence_source_kind.readiness_eligible()
            && !descriptor.provenance_available
        {
            continue;
        }
        let data_quality = row
            .get("data_quality_score")
            .and_then(Value::as_f64)
            .unwrap_or(default_quality(descriptor));
        let mut reason_codes = vec![
            ReasonCode::CommitteeMaterializationBuilt,
            ReasonCode::CommitteeRowLevelMaterialized,
        ];
        if data_quality < 0.80 {
            reason_codes.push(ReasonCode::SummaryDerived);
        }
        materialized.push(CommitteeScenarioRow {
            scenario_row_id: format!("{}-row-{index:03}", config.materialization_id),
            symbol,
            timestamp_ms: 1_700_000_000_000 + index as u64,
            source_kind: default_source_kind(descriptor),
            evidence_source_kind,
            market,
            target_horizon: PersonaHorizon::Swing,
            feature_vector: None,
            regime: if index % 2 == 0 {
                Regime::TrendUp
            } else {
                Regime::Range
            },
            signal_summary: format!("{:?}-row-level", descriptor.artifact_kind),
            data_quality_score: data_quality,
            spread_bps: Some(if market == ProviderMarket::Crypto {
                8.0
            } else {
                5.0
            }),
            expected_edge_after_cost: row
                .get("expected_edge_after_cost")
                .and_then(Value::as_f64)
                .unwrap_or(default_edge(descriptor)),
            expected_drawdown: row
                .get("expected_drawdown")
                .and_then(Value::as_f64)
                .unwrap_or(0.02),
            risk_snapshot_summary: Some("row-level-risk".to_string()),
            provenance_summary: if descriptor.provenance_available {
                "row-level-provenance".to_string()
            } else {
                "missing-provenance".to_string()
            },
            benchmark_status: Some("row-level".to_string()),
            baseline_signal_summary: Some("BaselineNoTrade".to_string()),
            external_prediction_summary: row
                .get("prediction")
                .and_then(Value::as_str)
                .map(str::to_string),
            no_trade_counterfactual: Some("always-no-trade".to_string()),
            risk_denial_counterfactual: Some("risk-denied-counterfactual".to_string()),
            outcome_reference: Some(format!("artifact-outcome-{index:03}")),
            materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
            materialization_confidence: default_confidence(descriptor),
            reason_codes,
        });
        if materialized.len() >= config.max_rows {
            break;
        }
    }
    Ok(materialized)
}

fn materialize_csv_rows(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let text = fs::read_to_string(&descriptor.path).map_err(|err| err.to_string())?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().skip(1).take(config.max_rows).enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(CommitteeScenarioRow {
            scenario_row_id: format!("{}-csv-{index:03}", config.materialization_id),
            symbol: descriptor
                .symbol
                .clone()
                .unwrap_or_else(|| default_symbol(descriptor).to_string()),
            timestamp_ms: 1_700_000_000_000 + index as u64,
            source_kind: default_source_kind(descriptor),
            evidence_source_kind: default_evidence_source(descriptor),
            market: descriptor
                .market
                .unwrap_or_else(|| default_market(descriptor)),
            target_horizon: PersonaHorizon::Swing,
            feature_vector: None,
            regime: if index % 2 == 0 {
                Regime::TrendUp
            } else {
                Regime::Range
            },
            signal_summary: "canonical-ohlcv-row".to_string(),
            data_quality_score: 0.90,
            spread_bps: Some(5.0),
            expected_edge_after_cost: 0.008,
            expected_drawdown: 0.020,
            risk_snapshot_summary: Some("csv-window".to_string()),
            provenance_summary: if descriptor.preflight_available {
                "csv+preflight".to_string()
            } else {
                "csv-without-preflight".to_string()
            },
            benchmark_status: Some("row-level".to_string()),
            baseline_signal_summary: Some("BaselineSignal".to_string()),
            external_prediction_summary: None,
            no_trade_counterfactual: Some("always-no-trade".to_string()),
            risk_denial_counterfactual: Some("risk-denied-counterfactual".to_string()),
            outcome_reference: Some(format!("csv-outcome-{index:03}")),
            materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
            materialization_confidence: 0.90,
            reason_codes: vec![
                ReasonCode::CommitteeMaterializationBuilt,
                ReasonCode::CommitteeRowLevelMaterialized,
            ],
        });
    }
    Ok(rows)
}

fn fallback_summary_rows(
    config: &CommitteeMaterializationConfig,
    descriptor: &CommitteeArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let source_kind = default_source_kind(descriptor);
    let mut loader_config = CommitteeScenarioLoadConfig {
        scenario_id: config.materialization_id.clone(),
        source_kind,
        input_paths: if Path::new(&descriptor.path).exists() {
            vec![descriptor.path.clone()]
        } else {
            Vec::new()
        },
        output_root: config.output_root.clone(),
        max_scenarios: config.max_rows.min(50),
        require_core_check: false,
        allow_yfinance_research: true,
        allow_fixture: true,
        allow_synthetic_test: true,
        min_data_quality: config.min_data_quality,
        reason_codes: vec![
            ReasonCode::CommitteeMaterializationBuilt,
            ReasonCode::CommitteeSummaryFallbackUsed,
        ],
    };
    if source_kind == CommitteeScenarioSourceKind::Unknown {
        loader_config.source_kind = CommitteeScenarioSourceKind::Fixture;
    }
    CommitteeScenarioLoader::default()
        .load(&loader_config)
        .map(|set| set.rows)
}

fn default_virtual_inputs(config: &CommitteeMaterializationConfig) -> Vec<String> {
    if config
        .allowed_artifact_kinds
        .contains(&CommitteeArtifactKind::FixtureScenario)
    {
        vec!["virtual-fixture".to_string()]
    } else if config
        .allowed_artifact_kinds
        .contains(&CommitteeArtifactKind::YahooResearchEvidenceReport)
    {
        vec!["virtual-yfinance".to_string()]
    } else if config
        .allowed_artifact_kinds
        .contains(&CommitteeArtifactKind::EvidenceLaneReport)
    {
        vec!["virtual-evidence-lane".to_string()]
    } else {
        Vec::new()
    }
}

fn default_allowed_artifacts() -> Vec<CommitteeArtifactKind> {
    vec![
        CommitteeArtifactKind::EvidenceLaneReport,
        CommitteeArtifactKind::ProviderRealityEvidenceReport,
        CommitteeArtifactKind::ReadinessMatrix,
        CommitteeArtifactKind::CoreCheckedBenchmarkReport,
        CommitteeArtifactKind::OfficialBenchmarkReport,
        CommitteeArtifactKind::SourceAwareBenchmarkReport,
        CommitteeArtifactKind::YahooResearchEvidenceReport,
        CommitteeArtifactKind::CommitteeV1Bundle,
        CommitteeArtifactKind::CanonicalOhlcvCsv,
        CommitteeArtifactKind::PreflightReport,
        CommitteeArtifactKind::FixtureScenario,
    ]
}

fn default_source_kind(descriptor: &CommitteeArtifactDescriptor) -> CommitteeScenarioSourceKind {
    descriptor
        .source_kind
        .unwrap_or(match descriptor.artifact_kind {
            CommitteeArtifactKind::EvidenceLaneReport => {
                CommitteeScenarioSourceKind::EvidenceLaneReport
            }
            CommitteeArtifactKind::CoreCheckedBenchmarkReport => {
                CommitteeScenarioSourceKind::CoreCheckedBenchmarkReport
            }
            CommitteeArtifactKind::OfficialBenchmarkReport => {
                CommitteeScenarioSourceKind::OfficialBenchmarkReport
            }
            CommitteeArtifactKind::SourceAwareBenchmarkReport => {
                CommitteeScenarioSourceKind::SourceAwareBenchmarkReport
            }
            CommitteeArtifactKind::YahooResearchEvidenceReport => {
                CommitteeScenarioSourceKind::YahooResearchEvidenceReport
            }
            CommitteeArtifactKind::FixtureScenario => CommitteeScenarioSourceKind::Fixture,
            _ => CommitteeScenarioSourceKind::Unknown,
        })
}

fn default_market(descriptor: &CommitteeArtifactDescriptor) -> ProviderMarket {
    descriptor.market.unwrap_or(match descriptor.artifact_kind {
        CommitteeArtifactKind::FixtureScenario => ProviderMarket::Crypto,
        _ => ProviderMarket::USEquity,
    })
}

fn default_symbol(descriptor: &CommitteeArtifactDescriptor) -> &'static str {
    if default_market(descriptor) == ProviderMarket::Crypto {
        "BTC-KRW"
    } else {
        "AAPL"
    }
}

fn default_edge(descriptor: &CommitteeArtifactDescriptor) -> f64 {
    if default_market(descriptor) == ProviderMarket::Crypto {
        0.012
    } else {
        0.008
    }
}

fn default_quality(descriptor: &CommitteeArtifactDescriptor) -> f64 {
    match descriptor.artifact_kind {
        CommitteeArtifactKind::YahooResearchEvidenceReport => 0.86,
        CommitteeArtifactKind::FixtureScenario => 0.90,
        _ => 0.88,
    }
}

fn default_confidence(descriptor: &CommitteeArtifactDescriptor) -> f64 {
    match descriptor.artifact_kind {
        CommitteeArtifactKind::YahooResearchEvidenceReport => 0.55,
        CommitteeArtifactKind::FixtureScenario => 0.95,
        _ => 0.85,
    }
}

fn default_evidence_source(descriptor: &CommitteeArtifactDescriptor) -> EvidenceSourceKind {
    match descriptor.artifact_kind {
        CommitteeArtifactKind::YahooResearchEvidenceReport => EvidenceSourceKind::YFinanceResearch,
        CommitteeArtifactKind::FixtureScenario => EvidenceSourceKind::TestFixture,
        _ => EvidenceSourceKind::OfficialApiCollected,
    }
}

fn parse_market(raw: &str) -> ProviderMarket {
    match raw {
        "Crypto" => ProviderMarket::Crypto,
        "KoreanEquity" => ProviderMarket::KoreanEquity,
        "GlobalEquity" => ProviderMarket::GlobalEquity,
        _ => ProviderMarket::USEquity,
    }
}

fn default_true() -> bool {
    true
}

fn default_max_rows() -> usize {
    100
}

fn default_max_symbols() -> usize {
    50
}

fn default_max_bytes() -> usize {
    5_000_000
}

fn default_min_quality() -> f64 {
    0.70
}
