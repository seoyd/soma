use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, Regime, stable_reason_codes};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::committee_scenario_loader::{
    CommitteeScenarioMaterializationLevel, CommitteeScenarioRow, CommitteeScenarioSet,
    CommitteeScenarioSourceKind,
};
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::official_evidence_replication::OfficialEvidenceReplicationConfig;
use super::official_replication_inventory::{
    OfficialReplicationArtifactDescriptor, OfficialReplicationArtifactInventory,
    OfficialReplicationArtifactKind,
};
use super::persona_card_lite::PersonaHorizon;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialRowInjectionPolicy {
    pub prefer_existing_official_committee_pack: bool,
    pub prefer_evidence_lane_rows: bool,
    pub allow_canonical_csv_to_scenario_rows: bool,
    pub require_preflight_ready: bool,
    pub require_provenance_official: bool,
    pub max_rows_per_symbol: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialEvidenceBoundary {
    OfficialNonCrypto,
    OfficialCryptoOnly,
    Controlled,
    ResearchOnly,
    FixtureOnly,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialSkippedRow {
    pub scenario_row_id: String,
    pub source_path: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialRowInjectionResult {
    pub injected_rows: Vec<CommitteeScenarioRow>,
    pub skipped_rows: Vec<OfficialSkippedRow>,
    pub official_row_count: usize,
    pub non_crypto_official_row_count: usize,
    pub crypto_only_row_count: usize,
    pub skipped_missing_provenance: usize,
    pub skipped_missing_preflight: usize,
    pub skipped_research_only: usize,
    pub skipped_fixture: usize,
    pub skipped_summary_derived: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialRowInjector;

impl Default for OfficialRowInjectionPolicy {
    fn default() -> Self {
        Self {
            prefer_existing_official_committee_pack: true,
            prefer_evidence_lane_rows: true,
            allow_canonical_csv_to_scenario_rows: true,
            require_preflight_ready: true,
            require_provenance_official: true,
            max_rows_per_symbol: 500,
            reason_codes: vec![ReasonCode::OfficialRowInjectionBuilt],
        }
    }
}

impl OfficialRowInjector {
    pub fn inject(
        &self,
        config: &OfficialEvidenceReplicationConfig,
        inventory: &OfficialReplicationArtifactInventory,
        policy: &OfficialRowInjectionPolicy,
    ) -> Result<OfficialRowInjectionResult, String> {
        let mut result = OfficialRowInjectionResult {
            injected_rows: Vec::new(),
            skipped_rows: Vec::new(),
            official_row_count: 0,
            non_crypto_official_row_count: 0,
            crypto_only_row_count: 0,
            skipped_missing_provenance: 0,
            skipped_missing_preflight: 0,
            skipped_research_only: 0,
            skipped_fixture: 0,
            skipped_summary_derived: 0,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialRowInjectionBuilt,
                ReasonCode::DeterministicPath,
            ]),
        };
        let allowed_symbols = collect_allowed_symbols(config, inventory);
        let mut injected_any = false;

        if policy.prefer_existing_official_committee_pack {
            for descriptor in inventory.descriptors.iter().filter(|descriptor| {
                descriptor.artifact_kind == OfficialReplicationArtifactKind::OfficialCommitteePack
            }) {
                let pack =
                    OfficialCommitteeScenarioPack::from_json_path(Path::new(&descriptor.path))?;
                let mut rows = pack.rows;
                rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
                for row in rows {
                    handle_candidate_row(
                        row,
                        descriptor,
                        config,
                        policy,
                        &allowed_symbols,
                        &mut result,
                    );
                }
                if !result.injected_rows.is_empty() {
                    injected_any = true;
                }
            }
        }

        if !injected_any && policy.prefer_evidence_lane_rows {
            for descriptor in inventory.descriptors.iter().filter(|descriptor| {
                descriptor.artifact_kind == OfficialReplicationArtifactKind::EvidenceLaneReport
            }) {
                for row in rows_from_evidence_lane(descriptor)? {
                    handle_candidate_row(
                        row,
                        descriptor,
                        config,
                        policy,
                        &allowed_symbols,
                        &mut result,
                    );
                }
            }
            injected_any = !result.injected_rows.is_empty();
        }

        if !injected_any && policy.allow_canonical_csv_to_scenario_rows {
            for descriptor in inventory.descriptors.iter().filter(|descriptor| {
                descriptor.artifact_kind == OfficialReplicationArtifactKind::OfficialCanonicalCsv
            }) {
                for row in rows_from_canonical_csv(descriptor, policy.max_rows_per_symbol)? {
                    handle_candidate_row(
                        row,
                        descriptor,
                        config,
                        policy,
                        &allowed_symbols,
                        &mut result,
                    );
                }
            }
        }

        result
            .injected_rows
            .sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
        result.skipped_rows.sort_by(|left, right| {
            left.scenario_row_id
                .cmp(&right.scenario_row_id)
                .then(left.source_path.cmp(&right.source_path))
                .then(left.reason.cmp(&right.reason))
        });
        result.official_row_count = result
            .injected_rows
            .iter()
            .filter(|row| {
                matches!(
                    classify_row_boundary(row),
                    OfficialEvidenceBoundary::OfficialNonCrypto
                        | OfficialEvidenceBoundary::OfficialCryptoOnly
                )
            })
            .count();
        result.non_crypto_official_row_count = result
            .injected_rows
            .iter()
            .filter(|row| classify_row_boundary(row) == OfficialEvidenceBoundary::OfficialNonCrypto)
            .count();
        result.crypto_only_row_count = result
            .injected_rows
            .iter()
            .filter(|row| {
                classify_row_boundary(row) == OfficialEvidenceBoundary::OfficialCryptoOnly
            })
            .count();
        Ok(result)
    }
}

impl OfficialRowInjectionResult {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("injected_row_count={}", self.injected_rows.len()),
            format!("skipped_row_count={}", self.skipped_rows.len()),
            format!("official_row_count={}", self.official_row_count),
            format!(
                "non_crypto_official_row_count={}",
                self.non_crypto_official_row_count
            ),
            format!("crypto_only_row_count={}", self.crypto_only_row_count),
            format!(
                "skipped_missing_provenance={}",
                self.skipped_missing_provenance
            ),
            format!(
                "skipped_missing_preflight={}",
                self.skipped_missing_preflight
            ),
            format!("skipped_research_only={}", self.skipped_research_only),
            format!("skipped_fixture={}", self.skipped_fixture),
            format!("skipped_summary_derived={}", self.skipped_summary_derived),
        ];
        lines.extend(self.injected_rows.iter().map(|row| {
            format!(
                "injected={};symbol={};boundary={:?};source={:?};evidence={:?}",
                row.scenario_row_id,
                row.symbol,
                classify_row_boundary(row),
                row.source_kind,
                row.evidence_source_kind,
            )
        }));
        lines.extend(self.skipped_rows.iter().map(|row| {
            format!(
                "skipped={};source_path={};reason={}",
                row.scenario_row_id, row.source_path, row.reason
            )
        }));
        lines.join("\n")
    }

    pub fn to_scenario_set(&self, scenario_id: &str) -> CommitteeScenarioSet {
        CommitteeScenarioSet {
            scenario_id: scenario_id.to_string(),
            row_count: self.injected_rows.len(),
            official_row_count: self.official_row_count,
            research_only_row_count: self
                .injected_rows
                .iter()
                .filter(|row| classify_row_boundary(row) == OfficialEvidenceBoundary::ResearchOnly)
                .count(),
            fixture_row_count: self
                .injected_rows
                .iter()
                .filter(|row| classify_row_boundary(row) == OfficialEvidenceBoundary::FixtureOnly)
                .count(),
            skipped_row_count: self.skipped_rows.len(),
            source_summary: self
                .injected_rows
                .iter()
                .fold(BTreeMap::<String, usize>::new(), |mut acc, row| {
                    *acc.entry(format!("{:?}", classify_row_boundary(row)))
                        .or_insert(0) += 1;
                    acc
                })
                .into_iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join("|"),
            rows: self.injected_rows.clone(),
            reason_codes: self.reason_codes.clone(),
        }
    }
}

pub fn classify_row_boundary(row: &CommitteeScenarioRow) -> OfficialEvidenceBoundary {
    let provenance = row.provenance_summary.to_ascii_lowercase();
    if matches!(
        row.evidence_source_kind,
        EvidenceSourceKind::YFinanceResearch | EvidenceSourceKind::ExternalPredictionOnly
    ) || provenance.contains("yfinance")
    {
        return OfficialEvidenceBoundary::ResearchOnly;
    }
    if matches!(
        row.evidence_source_kind,
        EvidenceSourceKind::TestFixture | EvidenceSourceKind::SyntheticFixture
    ) || matches!(
        row.source_kind,
        CommitteeScenarioSourceKind::Fixture | CommitteeScenarioSourceKind::SyntheticTest
    ) || provenance.contains("fixture")
        || provenance.contains("synthetic")
    {
        return OfficialEvidenceBoundary::FixtureOnly;
    }
    let true_official = row.evidence_source_kind == EvidenceSourceKind::OfficialApiCollected
        && !provenance.contains("controlled")
        && !provenance.contains("fixture")
        && !provenance.contains("yfinance")
        && (provenance.contains("official-api-collected")
            || provenance.contains("row-level-provenance")
            || provenance.contains("official_provider=true")
            || provenance.contains("downloaded_by_soma=true")
            || provenance.contains("official provider"));
    if true_official {
        if row.market == ProviderMarket::Crypto {
            OfficialEvidenceBoundary::OfficialCryptoOnly
        } else {
            OfficialEvidenceBoundary::OfficialNonCrypto
        }
    } else if matches!(
        row.evidence_source_kind,
        EvidenceSourceKind::OfficialApiCollected | EvidenceSourceKind::RealLocal
    ) {
        OfficialEvidenceBoundary::Controlled
    } else {
        OfficialEvidenceBoundary::Unknown
    }
}

fn handle_candidate_row(
    row: CommitteeScenarioRow,
    descriptor: &OfficialReplicationArtifactDescriptor,
    config: &OfficialEvidenceReplicationConfig,
    policy: &OfficialRowInjectionPolicy,
    allowed_symbols: &BTreeSet<String>,
    result: &mut OfficialRowInjectionResult,
) {
    let normalized_symbol = normalize_symbol(&row.symbol);
    if !allowed_symbols.is_empty() && !allowed_symbols.contains(&normalized_symbol) {
        result.skipped_rows.push(OfficialSkippedRow {
            scenario_row_id: row.scenario_row_id,
            source_path: descriptor.path.clone(),
            reason: "symbol_out_of_scope".to_string(),
        });
        return;
    }
    if row.materialization_level != CommitteeScenarioMaterializationLevel::RowLevel
        && !config.allow_summary_derived_rows
    {
        result.skipped_summary_derived += 1;
        result.skipped_rows.push(OfficialSkippedRow {
            scenario_row_id: row.scenario_row_id,
            source_path: descriptor.path.clone(),
            reason: "summary_derived".to_string(),
        });
        return;
    }
    let boundary = classify_row_boundary(&row);
    if policy.require_provenance_official
        && matches!(
            boundary,
            OfficialEvidenceBoundary::OfficialNonCrypto
                | OfficialEvidenceBoundary::OfficialCryptoOnly
        )
        && !descriptor.provenance_available
    {
        result.skipped_missing_provenance += 1;
        result.skipped_rows.push(OfficialSkippedRow {
            scenario_row_id: row.scenario_row_id,
            source_path: descriptor.path.clone(),
            reason: "missing_provenance".to_string(),
        });
        return;
    }
    if policy.require_preflight_ready
        && matches!(
            boundary,
            OfficialEvidenceBoundary::OfficialNonCrypto | OfficialEvidenceBoundary::Controlled
        )
        && !descriptor.preflight_available
    {
        result.skipped_missing_preflight += 1;
        result.skipped_rows.push(OfficialSkippedRow {
            scenario_row_id: row.scenario_row_id,
            source_path: descriptor.path.clone(),
            reason: "missing_preflight".to_string(),
        });
        return;
    }
    match boundary {
        OfficialEvidenceBoundary::ResearchOnly if !config.allow_yfinance_research => {
            result.skipped_research_only += 1;
            result.skipped_rows.push(OfficialSkippedRow {
                scenario_row_id: row.scenario_row_id,
                source_path: descriptor.path.clone(),
                reason: "research_only".to_string(),
            });
        }
        OfficialEvidenceBoundary::FixtureOnly if !config.allow_fixture => {
            result.skipped_fixture += 1;
            result.skipped_rows.push(OfficialSkippedRow {
                scenario_row_id: row.scenario_row_id,
                source_path: descriptor.path.clone(),
                reason: "fixture_only".to_string(),
            });
        }
        OfficialEvidenceBoundary::Controlled if !config.allow_controlled_fixture => {
            result.skipped_fixture += 1;
            result.skipped_rows.push(OfficialSkippedRow {
                scenario_row_id: row.scenario_row_id,
                source_path: descriptor.path.clone(),
                reason: "controlled_only".to_string(),
            });
        }
        _ => result.injected_rows.push(row),
    }
}

fn rows_from_evidence_lane(
    descriptor: &OfficialReplicationArtifactDescriptor,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let text = fs::read_to_string(&descriptor.path).map_err(|err| err.to_string())?;
    let json: Value = serde_json::from_str(&text).map_err(|err| err.to_string())?;
    let provenance = json
        .get("provenance")
        .and_then(Value::as_str)
        .unwrap_or("official-api-collected")
        .to_string();
    let rows = json
        .get("lane_reports")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut converted = rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| {
            let symbol = row
                .get("symbol")
                .and_then(Value::as_str)
                .unwrap_or("UNKNOWN")
                .to_string();
            let timestamp_ms = row
                .get("timestamp_ms")
                .and_then(Value::as_u64)
                .or_else(|| row.get("timestamp").and_then(Value::as_u64))
                .map(normalize_timestamp_ms)
                .unwrap_or(1_700_000_000_000 + index as u64);
            let market = parse_market(row.get("market").and_then(Value::as_str), Some(&symbol));
            let source_kind = if provenance.to_ascii_lowercase().contains("yfinance") {
                EvidenceSourceKind::YFinanceResearch
            } else if provenance.to_ascii_lowercase().contains("fixture") {
                EvidenceSourceKind::TestFixture
            } else {
                EvidenceSourceKind::OfficialApiCollected
            };
            CommitteeScenarioRow {
                scenario_row_id: format!("{}-lane-{index:04}", path_stem_slug(&descriptor.path)),
                symbol: symbol.clone(),
                timestamp_ms,
                source_kind: CommitteeScenarioSourceKind::EvidenceLaneReport,
                evidence_source_kind: source_kind,
                market,
                target_horizon: PersonaHorizon::Swing,
                feature_vector: None,
                regime: Regime::TrendUp,
                signal_summary: "official-evidence-lane".to_string(),
                data_quality_score: row
                    .get("data_quality_score")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.80),
                spread_bps: row.get("spread_bps").and_then(Value::as_f64),
                expected_edge_after_cost: row
                    .get("expected_edge_after_cost")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
                expected_drawdown: row
                    .get("expected_drawdown")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.01),
                risk_snapshot_summary: Some("official-lane".to_string()),
                provenance_summary: if source_kind == EvidenceSourceKind::OfficialApiCollected {
                    format!("row-level-provenance: {provenance}")
                } else {
                    provenance.clone()
                },
                benchmark_status: Some("Ready".to_string()),
                baseline_signal_summary: row
                    .get("baseline_signal_summary")
                    .and_then(Value::as_str)
                    .map(|value| value.to_string())
                    .or(Some("Approve".to_string())),
                external_prediction_summary: row
                    .get("external_prediction_summary")
                    .and_then(Value::as_str)
                    .map(|value| value.to_string()),
                no_trade_counterfactual: None,
                risk_denial_counterfactual: None,
                outcome_reference: row
                    .get("outcome_reference")
                    .and_then(Value::as_str)
                    .map(|value| value.to_string()),
                materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
                materialization_confidence: 1.0,
                reason_codes: stable_reason_codes(&[
                    ReasonCode::OfficialRowInjectionBuilt,
                    ReasonCode::CommitteeRowLevelMaterialized,
                ]),
            }
        })
        .collect::<Vec<_>>();
    converted.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
    Ok(converted)
}

fn rows_from_canonical_csv(
    descriptor: &OfficialReplicationArtifactDescriptor,
    max_rows_per_symbol: usize,
) -> Result<Vec<CommitteeScenarioRow>, String> {
    let text = fs::read_to_string(&descriptor.path).map_err(|err| err.to_string())?;
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let header = lines
        .next()
        .ok_or_else(|| format!("{} is empty", descriptor.path))?
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let timestamp_index = header
        .iter()
        .position(|value| value == "timestamp" || value == "timestamp_ms")
        .ok_or_else(|| format!("{} missing timestamp column", descriptor.path))?;
    let open_index = header
        .iter()
        .position(|value| value == "open")
        .ok_or_else(|| format!("{} missing open column", descriptor.path))?;
    let high_index = header
        .iter()
        .position(|value| value == "high")
        .ok_or_else(|| format!("{} missing high column", descriptor.path))?;
    let low_index = header
        .iter()
        .position(|value| value == "low")
        .ok_or_else(|| format!("{} missing low column", descriptor.path))?;
    let close_index = header
        .iter()
        .position(|value| value == "close")
        .ok_or_else(|| format!("{} missing close column", descriptor.path))?;
    let volume_index = header.iter().position(|value| value == "volume");
    let symbol = descriptor
        .symbol
        .clone()
        .unwrap_or_else(|| path_stem_slug(&descriptor.path));
    let source_kind = if descriptor.source_research_only {
        EvidenceSourceKind::YFinanceResearch
    } else if descriptor.source_fixture_only {
        EvidenceSourceKind::TestFixture
    } else if descriptor.provenance_available {
        EvidenceSourceKind::OfficialApiCollected
    } else {
        EvidenceSourceKind::RealLocal
    };
    let provenance_summary = if source_kind == EvidenceSourceKind::OfficialApiCollected {
        format!(
            "row-level-provenance: official-api-collected; path={}",
            descriptor.path
        )
    } else {
        format!("controlled-local-csv; path={}", descriptor.path)
    };
    let mut rows = Vec::new();
    for (index, line) in lines.take(max_rows_per_symbol).enumerate() {
        let columns = line
            .split(',')
            .map(|value| value.trim())
            .collect::<Vec<_>>();
        if columns.len() <= close_index {
            continue;
        }
        let open = columns[open_index].parse::<f64>().unwrap_or(0.0);
        let high = columns[high_index].parse::<f64>().unwrap_or(open);
        let low = columns[low_index].parse::<f64>().unwrap_or(open);
        let close = columns[close_index].parse::<f64>().unwrap_or(open);
        let volume = volume_index
            .and_then(|column| columns.get(column))
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        let data_quality_score = if open > 0.0 && high >= low && close > 0.0 {
            0.92
        } else {
            0.50
        };
        let expected_edge_after_cost = if open > 0.0 {
            ((close - open) / open).clamp(-0.20, 0.20)
        } else {
            0.0
        };
        let expected_drawdown = if open > 0.0 {
            ((open - low) / open).abs().clamp(0.0, 0.20)
        } else {
            0.01
        };
        rows.push(CommitteeScenarioRow {
            scenario_row_id: format!("{}-csv-{index:04}", path_stem_slug(&descriptor.path)),
            symbol: symbol.clone(),
            timestamp_ms: normalize_timestamp_ms(
                columns[timestamp_index].parse::<u64>().unwrap_or(0),
            ),
            source_kind: CommitteeScenarioSourceKind::EvidenceLaneReport,
            evidence_source_kind: source_kind,
            market: descriptor
                .market
                .unwrap_or_else(|| parse_market(None, Some(&symbol))),
            target_horizon: PersonaHorizon::Swing,
            feature_vector: None,
            regime: Regime::TrendUp,
            signal_summary: "official-canonical-row-injection".to_string(),
            data_quality_score,
            spread_bps: Some(if volume > 0.0 { 4.0 } else { 8.0 }),
            expected_edge_after_cost,
            expected_drawdown,
            risk_snapshot_summary: Some("limited-canonical-row".to_string()),
            provenance_summary: provenance_summary.clone(),
            benchmark_status: Some("Ready".to_string()),
            baseline_signal_summary: Some(if expected_edge_after_cost >= 0.0 {
                "Approve".to_string()
            } else {
                "NoTrade".to_string()
            }),
            external_prediction_summary: None,
            no_trade_counterfactual: None,
            risk_denial_counterfactual: None,
            outcome_reference: None,
            materialization_level: CommitteeScenarioMaterializationLevel::RowLevel,
            materialization_confidence: 0.60,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialRowInjectionBuilt,
                ReasonCode::CommitteeRowLevelMaterialized,
                ReasonCode::FeatureUnavailable,
            ]),
        });
    }
    rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
    Ok(rows)
}

fn collect_allowed_symbols(
    config: &OfficialEvidenceReplicationConfig,
    inventory: &OfficialReplicationArtifactInventory,
) -> BTreeSet<String> {
    inventory
        .descriptors
        .iter()
        .filter_map(|descriptor| descriptor.symbol.as_ref())
        .map(|symbol| normalize_symbol(symbol))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(config.max_symbols)
        .collect()
}

fn parse_market(explicit: Option<&str>, symbol: Option<&str>) -> ProviderMarket {
    match explicit.map(|value| value.to_ascii_lowercase()) {
        Some(value) if value.contains("crypto") => ProviderMarket::Crypto,
        Some(value) if value.contains("korean") || value.contains("krx") => {
            ProviderMarket::KoreanEquity
        }
        Some(value) if value.contains("us") || value.contains("equity") => ProviderMarket::USEquity,
        _ => {
            if symbol.is_some_and(|value| {
                value.contains('-')
                    || value.to_ascii_uppercase().contains("BTC")
                    || value.to_ascii_uppercase().contains("ETH")
            }) {
                ProviderMarket::Crypto
            } else {
                ProviderMarket::USEquity
            }
        }
    }
}

fn normalize_symbol(symbol: &str) -> String {
    symbol
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

fn path_stem_slug(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| {
            stem.chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
                .collect::<String>()
        })
        .unwrap_or_else(|| "artifact".to_string())
}

fn normalize_timestamp_ms(value: u64) -> u64 {
    if value == 0 {
        1_700_000_000_000
    } else if value < 1_000_000_000_000 {
        value.saturating_mul(1000)
    } else {
        value
    }
}
