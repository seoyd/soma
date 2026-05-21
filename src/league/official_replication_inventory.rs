use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backtest::Timeframe;
use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{
    DataProvenance, EvidenceSourceKind, OfficialCollectionReport, PreflightReport, ProviderKind,
    ProviderMarket,
};
use crate::experiment::{OfficialProviderReadinessReport, ProviderRealityReport};

use super::committee_outcome_coverage::CommitteeOutcomeCoverageReport;
use super::committee_reference_pack::GeneratedCommitteeReferencePack;
use super::official_committee_pack::OfficialCommitteeScenarioPack;
use super::official_evidence_replication::OfficialEvidenceReplicationConfig;
use super::sufficiency_closure::SufficiencyClosureReport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReplicationArtifactKind {
    ProviderReadinessReport,
    ProviderRealityReport,
    OfficialCollectionReport,
    OfficialCanonicalCsv,
    OfficialPreflightReport,
    OfficialProvenance,
    EvidenceLaneReport,
    OfficialCommitteePack,
    GeneratedReferencePack,
    SufficiencyClosureReport,
    OutcomeCoverageReport,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReplicationArtifactDescriptor {
    pub path: String,
    pub artifact_kind: OfficialReplicationArtifactKind,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    #[serde(default)]
    pub source_kind: Option<EvidenceSourceKind>,
    #[serde(default)]
    pub market: Option<ProviderMarket>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    pub source_official: bool,
    pub source_crypto_only: bool,
    pub source_research_only: bool,
    pub source_fixture_only: bool,
    pub provenance_available: bool,
    pub preflight_available: bool,
    pub candle_available: bool,
    pub row_level_available: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReplicationArtifactInventory {
    pub descriptors: Vec<OfficialReplicationArtifactDescriptor>,
    pub official_artifact_count: usize,
    pub non_crypto_official_artifact_count: usize,
    pub crypto_only_artifact_count: usize,
    pub research_only_artifact_count: usize,
    pub fixture_artifact_count: usize,
    pub missing_provenance_count: usize,
    pub missing_preflight_count: usize,
    pub missing_candle_count: usize,
    pub unknown_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialReplicationInventoryResolver;

impl OfficialReplicationInventoryResolver {
    pub fn resolve(
        &self,
        config: &OfficialEvidenceReplicationConfig,
    ) -> OfficialReplicationArtifactInventory {
        let mut paths = BTreeSet::new();
        for path in config.all_artifact_paths() {
            paths.insert(path);
        }
        OfficialReplicationArtifactInventory::from_paths(&paths.into_iter().collect::<Vec<_>>())
    }
}

impl OfficialReplicationArtifactInventory {
    pub fn from_paths(paths: &[String]) -> Self {
        let mut descriptors = paths
            .iter()
            .map(|path| describe_artifact(path))
            .collect::<Vec<_>>();
        descriptors.sort_by(|left, right| left.path.cmp(&right.path));
        let official_artifact_count = descriptors.iter().filter(|d| d.source_official).count();
        let non_crypto_official_artifact_count = descriptors
            .iter()
            .filter(|d| d.source_official && !d.source_crypto_only)
            .count();
        let crypto_only_artifact_count =
            descriptors.iter().filter(|d| d.source_crypto_only).count();
        let research_only_artifact_count = descriptors
            .iter()
            .filter(|d| d.source_research_only)
            .count();
        let fixture_artifact_count = descriptors.iter().filter(|d| d.source_fixture_only).count();
        let missing_provenance_count = descriptors
            .iter()
            .filter(|d| needs_provenance(d.artifact_kind) && !d.provenance_available)
            .count();
        let missing_preflight_count = descriptors
            .iter()
            .filter(|d| needs_preflight(d.artifact_kind) && !d.preflight_available)
            .count();
        let missing_candle_count = descriptors
            .iter()
            .filter(|d| needs_candles(d.artifact_kind) && !d.candle_available)
            .count();
        let unknown_count = descriptors
            .iter()
            .filter(|d| d.artifact_kind == OfficialReplicationArtifactKind::Unknown)
            .count();
        Self {
            descriptors,
            official_artifact_count,
            non_crypto_official_artifact_count,
            crypto_only_artifact_count,
            research_only_artifact_count,
            fixture_artifact_count,
            missing_provenance_count,
            missing_preflight_count,
            missing_candle_count,
            unknown_count,
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialReplicationInventoryBuilt,
                ReasonCode::DeterministicPath,
            ]),
        }
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("official_artifact_count={}", self.official_artifact_count),
            format!(
                "non_crypto_official_artifact_count={}",
                self.non_crypto_official_artifact_count
            ),
            format!(
                "crypto_only_artifact_count={}",
                self.crypto_only_artifact_count
            ),
            format!(
                "research_only_artifact_count={}",
                self.research_only_artifact_count
            ),
            format!("fixture_artifact_count={}", self.fixture_artifact_count),
            format!("missing_provenance_count={}", self.missing_provenance_count),
            format!("missing_preflight_count={}", self.missing_preflight_count),
            format!("missing_candle_count={}", self.missing_candle_count),
            format!("unknown_count={}", self.unknown_count),
        ];
        lines.extend(self.descriptors.iter().map(|descriptor| {
            format!(
                "path={};kind={:?};provider={};source={};market={};symbol={};timeframe={};official={};crypto_only={};research_only={};fixture_only={};provenance_available={};preflight_available={};candle_available={};row_level_available={}",
                descriptor.path,
                descriptor.artifact_kind,
                descriptor
                    .provider_kind
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default(),
                descriptor
                    .source_kind
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default(),
                descriptor
                    .market
                    .map(|value| format!("{value:?}"))
                    .unwrap_or_default(),
                descriptor.symbol.clone().unwrap_or_default(),
                descriptor.timeframe.clone().unwrap_or_default(),
                descriptor.source_official,
                descriptor.source_crypto_only,
                descriptor.source_research_only,
                descriptor.source_fixture_only,
                descriptor.provenance_available,
                descriptor.preflight_available,
                descriptor.candle_available,
                descriptor.row_level_available,
            )
        }));
        lines.join("\n")
    }
}

fn describe_artifact(path: &str) -> OfficialReplicationArtifactDescriptor {
    let path_ref = Path::new(path);
    let bytes = fs::read(path_ref).ok();
    let text = bytes
        .as_deref()
        .map(String::from_utf8_lossy)
        .map(|value| value.to_string())
        .unwrap_or_default();
    let json = serde_json::from_str::<Value>(&text).ok();
    let lowered = path.to_ascii_lowercase();
    let artifact_kind = detect_kind(path_ref, &lowered, json.as_ref());
    let symbol = detect_symbol(path_ref, json.as_ref());
    let timeframe = detect_timeframe(path_ref, &text, json.as_ref());
    let market = detect_market(path_ref, &text, json.as_ref(), symbol.as_deref());
    let provider_kind = detect_provider_kind(&text, json.as_ref());
    let source_kind = detect_source_kind(path_ref, &text, json.as_ref(), artifact_kind);
    let provenance_available =
        detect_provenance_available(path_ref, &text, json.as_ref(), artifact_kind);
    let preflight_available =
        detect_preflight_available(path_ref, &text, json.as_ref(), artifact_kind);
    let candle_available = detect_candle_available(path_ref, &text, json.as_ref(), artifact_kind);
    let row_level_available = detect_row_level_available(&text, json.as_ref(), artifact_kind);
    let source_research_only = matches!(source_kind, Some(EvidenceSourceKind::YFinanceResearch))
        || lowered.contains("yfinance")
        || lowered.contains("yahoo");
    let source_fixture_only = matches!(
        source_kind,
        Some(EvidenceSourceKind::TestFixture | EvidenceSourceKind::SyntheticFixture)
    ) || lowered.contains("fixture")
        || lowered.contains("mock");
    let source_crypto_only =
        market == Some(ProviderMarket::Crypto) && !source_research_only && !source_fixture_only;
    let source_official = !source_research_only
        && !source_fixture_only
        && matches!(source_kind, Some(EvidenceSourceKind::OfficialApiCollected));
    let mut reason_codes = vec![ReasonCode::OfficialReplicationInventoryBuilt];
    if !path_ref.exists() {
        reason_codes.push(ReasonCode::MissingFile);
    }
    if artifact_kind == OfficialReplicationArtifactKind::Unknown {
        reason_codes.push(ReasonCode::CommitteeArtifactUnknown);
    }
    if !path_ref.exists() && path.contains("://") {
        reason_codes.push(ReasonCode::RemotePathRejected);
    }
    OfficialReplicationArtifactDescriptor {
        path: path.to_string(),
        artifact_kind,
        provider_kind,
        source_kind,
        market,
        symbol,
        timeframe,
        source_official,
        source_crypto_only,
        source_research_only,
        source_fixture_only,
        provenance_available,
        preflight_available,
        candle_available,
        row_level_available,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn detect_kind(
    path: &Path,
    lowered: &str,
    json: Option<&Value>,
) -> OfficialReplicationArtifactKind {
    if lowered.contains("provider_readiness")
        || json.is_some_and(|value| {
            value.get("selection_results").is_some() && value.get("catalog").is_some()
        })
    {
        OfficialReplicationArtifactKind::ProviderReadinessReport
    } else if lowered.contains("provider_reality")
        || json.is_some_and(|value| {
            value.get("entitlement_statuses").is_some() && value.get("recommendations").is_some()
        })
    {
        OfficialReplicationArtifactKind::ProviderRealityReport
    } else if lowered.contains("official_collection_report")
        || json.is_some_and(|value| {
            value.get("entry_reports").is_some()
                && value.get("official_api_collected_count").is_some()
        })
    {
        OfficialReplicationArtifactKind::OfficialCollectionReport
    } else if lowered.ends_with(".csv") {
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
    } else if lowered.contains("preflight")
        || json.is_some_and(|value| {
            value.get("final_status").is_some() && value.get("checks").is_some()
        })
    {
        OfficialReplicationArtifactKind::OfficialPreflightReport
    } else if lowered.contains("provenance")
        || json.is_some_and(|value| {
            value.get("source_kind").is_some() && value.get("source_label").is_some()
        })
    {
        OfficialReplicationArtifactKind::OfficialProvenance
    } else if lowered.contains("evidence_lane")
        || json.is_some_and(|value| {
            value.get("lane_reports").is_some() || value.get("lane_status").is_some()
        })
    {
        OfficialReplicationArtifactKind::EvidenceLaneReport
    } else if lowered.contains("official_scenario_pack")
        || json.is_some_and(|value| {
            value.get("rows").is_some() && value.get("official_row_count").is_some()
        })
    {
        OfficialReplicationArtifactKind::OfficialCommitteePack
    } else if lowered.contains("generated_reference_pack")
        || json.is_some_and(|value| {
            value.get("generated_references").is_some() && value.get("alignment_report").is_some()
        })
    {
        OfficialReplicationArtifactKind::GeneratedReferencePack
    } else if lowered.contains("sufficiency_closure")
        || json.is_some_and(|value| {
            value.get("closure_id").is_some() && value.get("current_status").is_some()
        })
    {
        OfficialReplicationArtifactKind::SufficiencyClosureReport
    } else if lowered.contains("outcome_coverage")
        || json
            .is_some_and(|value| value.get("coverage_id").is_some() && value.get("cells").is_some())
    {
        OfficialReplicationArtifactKind::OutcomeCoverageReport
    } else if lowered.ends_with(".json") && path.exists() {
        if OfficialCommitteeScenarioPack::from_json_path(path).is_ok() {
            OfficialReplicationArtifactKind::OfficialCommitteePack
        } else if GeneratedCommitteeReferencePack::from_json_path(path).is_ok() {
            OfficialReplicationArtifactKind::GeneratedReferencePack
        } else if OfficialCollectionReport::from_json_path(path).is_ok() {
            OfficialReplicationArtifactKind::OfficialCollectionReport
        } else if ProviderRealityReport::from_json_path(path).is_ok() {
            OfficialReplicationArtifactKind::ProviderRealityReport
        } else {
            OfficialReplicationArtifactKind::Unknown
        }
    } else {
        OfficialReplicationArtifactKind::Unknown
    }
}

fn detect_symbol(path: &Path, json: Option<&Value>) -> Option<String> {
    json.and_then(|value| {
        value
            .get("symbol")
            .and_then(Value::as_str)
            .map(|value| value.to_string())
            .or_else(|| {
                value
                    .get("entry_reports")?
                    .as_array()?
                    .first()?
                    .get("symbol")?
                    .as_str()
                    .map(|value| value.to_string())
            })
            .or_else(|| {
                value
                    .get("rows")?
                    .as_array()?
                    .first()?
                    .get("symbol")?
                    .as_str()
                    .map(|value| value.to_string())
            })
    })
    .or_else(|| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|stem| stem.trim_end_matches("_candles").to_string())
    })
}

fn detect_timeframe(path: &Path, text: &str, json: Option<&Value>) -> Option<String> {
    json.and_then(|value| {
        value
            .get("timeframe")
            .and_then(Value::as_str)
            .map(|value| value.to_string())
            .or_else(|| {
                value
                    .get("entry_reports")?
                    .as_array()?
                    .first()?
                    .get("timeframe")
                    .map(|value| value.to_string().trim_matches('"').to_string())
            })
            .or_else(|| {
                value
                    .get("scenario_rows")?
                    .as_array()?
                    .first()?
                    .get("target_horizon")?
                    .as_str()
                    .map(|value| value.to_string())
            })
    })
    .or_else(|| {
        if text.contains("OneDay") || path.to_string_lossy().to_ascii_lowercase().contains("1d") {
            Some(format!("{:?}", Timeframe::OneDay))
        } else if text.contains("OneMinute")
            || path.to_string_lossy().to_ascii_lowercase().contains("1m")
        {
            Some(format!("{:?}", Timeframe::OneMinute))
        } else {
            None
        }
    })
}

fn detect_market(
    path: &Path,
    text: &str,
    json: Option<&Value>,
    symbol: Option<&str>,
) -> Option<ProviderMarket> {
    if json.is_some_and(|value| {
        value
            .get("market")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("crypto"))
    }) || text.contains("\"market\":\"Crypto\"")
        || path.to_string_lossy().to_ascii_lowercase().contains("btc")
        || symbol.is_some_and(is_crypto_symbol)
    {
        Some(ProviderMarket::Crypto)
    } else if text.contains("KoreanEquity")
        || path.to_string_lossy().to_ascii_lowercase().contains("krx")
    {
        Some(ProviderMarket::KoreanEquity)
    } else if text.contains("USEquity") || symbol.is_some_and(is_us_equity_symbol) {
        Some(ProviderMarket::USEquity)
    } else {
        None
    }
}

fn detect_provider_kind(text: &str, json: Option<&Value>) -> Option<ProviderKind> {
    let direct = json.and_then(|value| {
        value
            .get("provider_kind")
            .and_then(Value::as_str)
            .map(parse_provider_kind)
            .or_else(|| {
                value
                    .get("entry_reports")?
                    .as_array()?
                    .first()?
                    .get("provider_kind")?
                    .as_str()
                    .map(parse_provider_kind)
            })
    });
    direct.or_else(|| {
        let lowered = text.to_ascii_lowercase();
        if lowered.contains("alphavantage") {
            Some(ProviderKind::AlphaVantage)
        } else if lowered.contains("krx") {
            Some(ProviderKind::KrxOpenApi)
        } else if lowered.contains("data-go-kr") {
            Some(ProviderKind::DataGoKrFscStockPrice)
        } else if lowered.contains("alpaca") {
            Some(ProviderKind::Alpaca)
        } else if lowered.contains("upbit") {
            Some(ProviderKind::Upbit)
        } else {
            None
        }
    })
}

fn detect_source_kind(
    path: &Path,
    text: &str,
    json: Option<&Value>,
    artifact_kind: OfficialReplicationArtifactKind,
) -> Option<EvidenceSourceKind> {
    if let Some(source_kind) = json.and_then(|value| {
        value
            .get("source_kind")
            .and_then(Value::as_str)
            .map(parse_source_kind)
            .or_else(|| {
                value
                    .get("provenance")?
                    .get("source_kind")?
                    .as_str()
                    .map(parse_source_kind)
            })
            .or_else(|| {
                value
                    .get("rows")?
                    .as_array()?
                    .first()?
                    .get("evidence_source_kind")?
                    .as_str()
                    .map(parse_source_kind)
            })
    }) {
        return Some(source_kind);
    }
    let lowered = path.to_string_lossy().to_ascii_lowercase();
    if lowered.contains("yfinance") || text.to_ascii_lowercase().contains("yfinance") {
        Some(EvidenceSourceKind::YFinanceResearch)
    } else if lowered.contains("fixture") || lowered.contains("mock") {
        Some(EvidenceSourceKind::TestFixture)
    } else if matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
    ) && lowered.contains("official")
    {
        Some(EvidenceSourceKind::OfficialApiCollected)
    } else if matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialCollectionReport
            | OfficialReplicationArtifactKind::OfficialCommitteePack
            | OfficialReplicationArtifactKind::GeneratedReferencePack
            | OfficialReplicationArtifactKind::EvidenceLaneReport
    ) && !lowered.contains("controlled")
        && !lowered.contains("fixture")
    {
        Some(EvidenceSourceKind::OfficialApiCollected)
    } else {
        let inferred = DataProvenance::inferred_from_path(path.to_str()).source_kind;
        (inferred != EvidenceSourceKind::Unknown).then_some(inferred)
    }
}

fn detect_provenance_available(
    path: &Path,
    text: &str,
    json: Option<&Value>,
    artifact_kind: OfficialReplicationArtifactKind,
) -> bool {
    if matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialProvenance
    ) {
        return true;
    }
    if text.contains("provenance")
        || json.is_some_and(|value| {
            value.get("provenance").is_some() || value.get("provenance_path").is_some()
        })
    {
        return true;
    }
    find_neighbor(path, "provenance").is_some()
}

fn detect_preflight_available(
    path: &Path,
    text: &str,
    json: Option<&Value>,
    artifact_kind: OfficialReplicationArtifactKind,
) -> bool {
    if matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialPreflightReport
    ) {
        return true;
    }
    if text.contains("preflight")
        || json.is_some_and(|value| {
            value.get("preflight").is_some()
                || value.get("preflight_status").is_some()
                || value
                    .get("entry_reports")
                    .and_then(Value::as_array)
                    .is_some_and(|entries| {
                        entries
                            .iter()
                            .any(|entry| entry.get("preflight_status").is_some())
                    })
        })
    {
        return true;
    }
    find_neighbor(path, "preflight").is_some()
}

fn detect_candle_available(
    path: &Path,
    text: &str,
    json: Option<&Value>,
    artifact_kind: OfficialReplicationArtifactKind,
) -> bool {
    if matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
    ) {
        return true;
    }
    if text.contains("\"candles\"")
        || json.is_some_and(|value| value.get("candles").is_some())
        || find_neighbor(path, "candle").is_some()
    {
        return true;
    }
    if matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialCollectionReport
    ) {
        return json.is_some_and(|value| {
            value
                .get("entry_reports")
                .and_then(Value::as_array)
                .is_some_and(|entries| {
                    entries
                        .iter()
                        .any(|entry| entry.get("canonical_csv_path").is_some())
                })
        });
    }
    false
}

fn detect_row_level_available(
    text: &str,
    json: Option<&Value>,
    artifact_kind: OfficialReplicationArtifactKind,
) -> bool {
    matches!(
        artifact_kind,
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
    ) || text.contains("row_level")
        || json.is_some_and(|value| {
            value.get("lane_reports").is_some()
                || value.get("rows").is_some()
                || value.get("generated_references").is_some()
        })
}

fn find_neighbor(path: &Path, needle: &str) -> Option<PathBuf> {
    let parent = path.parent()?;
    let entries = fs::read_dir(parent).ok()?;
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|candidate| {
            candidate
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.to_ascii_lowercase().contains(needle))
        })
}

fn needs_provenance(kind: OfficialReplicationArtifactKind) -> bool {
    matches!(
        kind,
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
            | OfficialReplicationArtifactKind::EvidenceLaneReport
            | OfficialReplicationArtifactKind::OfficialCommitteePack
    )
}

fn needs_preflight(kind: OfficialReplicationArtifactKind) -> bool {
    matches!(
        kind,
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
            | OfficialReplicationArtifactKind::EvidenceLaneReport
    )
}

fn needs_candles(kind: OfficialReplicationArtifactKind) -> bool {
    matches!(
        kind,
        OfficialReplicationArtifactKind::OfficialCanonicalCsv
            | OfficialReplicationArtifactKind::OfficialCommitteePack
    )
}

fn parse_provider_kind(value: &str) -> ProviderKind {
    match value.to_ascii_lowercase().as_str() {
        "krxopenapi" | "krx" => ProviderKind::KrxOpenApi,
        "datagokrfscstockprice" | "data-go-kr-fsc-stock-price" | "datagokr" => {
            ProviderKind::DataGoKrFscStockPrice
        }
        "alphavantage" => ProviderKind::AlphaVantage,
        "alpaca" => ProviderKind::Alpaca,
        "upbit" => ProviderKind::Upbit,
        "mockfixture" | "mock-fixture" => ProviderKind::MockFixture,
        _ => ProviderKind::Unknown,
    }
}

fn parse_source_kind(value: &str) -> EvidenceSourceKind {
    match value.to_ascii_lowercase().as_str() {
        "officialapicollected" | "official-api-collected" => {
            EvidenceSourceKind::OfficialApiCollected
        }
        "reallocal" | "real-local" => EvidenceSourceKind::RealLocal,
        "yfinanceresearch" | "yfinance-research" | "yfinance" => {
            EvidenceSourceKind::YFinanceResearch
        }
        "testfixture" | "test-fixture" => EvidenceSourceKind::TestFixture,
        "syntheticfixture" | "synthetic-fixture" => EvidenceSourceKind::SyntheticFixture,
        _ => EvidenceSourceKind::Unknown,
    }
}

fn is_crypto_symbol(symbol: &str) -> bool {
    let upper = symbol.to_ascii_uppercase();
    upper.contains("BTC")
        || upper.contains("ETH")
        || upper.contains("USDT")
        || upper.contains("KRW")
}

fn is_us_equity_symbol(symbol: &str) -> bool {
    symbol.chars().all(|value| value.is_ascii_uppercase()) && symbol.len() <= 5
}

impl OfficialProviderReadinessReport {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }
}

impl PreflightReport {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }
}

impl SufficiencyClosureReport {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }
}

impl CommitteeOutcomeCoverageReport {
    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }
}
