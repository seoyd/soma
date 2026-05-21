use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::core_performance_scorecard::CorePerformanceScorecardConfig;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CorePerformanceArtifactKind {
    CoreCheckReport,
    OfficialReplicationReport,
    OfficialCommitteeBenchmarkReport,
    CommitteeOutcomeCoverageBundle,
    CommitteeReferencePackBundle,
    CommitteeBenchmarkBundle,
    SourceAwareBenchmarkReport,
    YahooResearchEvidenceReport,
    ProviderReadinessReport,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceArtifactDescriptor {
    pub path: String,
    pub artifact_kind: CorePerformanceArtifactKind,
    #[serde(default)]
    pub source_kind: Option<EvidenceSourceKind>,
    #[serde(default)]
    pub market: Option<ProviderMarket>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub timeframe: Option<String>,
    pub official: bool,
    pub non_crypto_official: bool,
    pub crypto_only: bool,
    pub research_only: bool,
    pub fixture_only: bool,
    pub controlled_only: bool,
    pub row_level_available: bool,
    pub outcome_linked_available: bool,
    pub counterfactual_available: bool,
    pub baseline_reference_available: bool,
    pub core_check_available: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorePerformanceArtifactInventory {
    pub descriptors: Vec<CorePerformanceArtifactDescriptor>,
    pub official_artifact_count: usize,
    pub non_crypto_official_count: usize,
    pub crypto_only_count: usize,
    pub research_only_count: usize,
    pub fixture_only_count: usize,
    pub controlled_only_count: usize,
    pub outcome_linked_count: usize,
    pub counterfactual_count: usize,
    pub baseline_reference_count: usize,
    pub unknown_count: usize,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl CorePerformanceArtifactInventory {
    pub fn from_config(config: &CorePerformanceScorecardConfig) -> Self {
        let mut hinted_paths = Vec::new();
        hinted_paths.extend(
            config
                .core_check_report_paths
                .iter()
                .cloned()
                .map(|path| (path, Some(CorePerformanceArtifactKind::CoreCheckReport))),
        );
        hinted_paths.extend(
            config
                .official_replication_report_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::OfficialReplicationReport),
                    )
                }),
        );
        hinted_paths.extend(
            config
                .committee_official_benchmark_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport),
                    )
                }),
        );
        hinted_paths.extend(
            config
                .committee_outcome_coverage_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle),
                    )
                }),
        );
        hinted_paths.extend(
            config
                .committee_reference_pack_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::CommitteeReferencePackBundle),
                    )
                }),
        );
        hinted_paths.extend(
            config
                .committee_benchmark_bundle_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::CommitteeBenchmarkBundle),
                    )
                }),
        );
        hinted_paths.extend(
            config
                .source_aware_benchmark_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::SourceAwareBenchmarkReport),
                    )
                }),
        );
        hinted_paths.extend(
            config
                .yahoo_research_report_paths
                .iter()
                .cloned()
                .map(|path| {
                    (
                        path,
                        Some(CorePerformanceArtifactKind::YahooResearchEvidenceReport),
                    )
                }),
        );
        let readiness_paths = config
            .all_artifact_paths()
            .into_iter()
            .filter(|path| path.to_ascii_lowercase().contains("provider_readiness"))
            .map(|path| {
                (
                    path,
                    Some(CorePerformanceArtifactKind::ProviderReadinessReport),
                )
            })
            .collect::<Vec<_>>();
        hinted_paths.extend(readiness_paths);
        Self::from_hinted_paths(&hinted_paths)
    }

    pub fn from_paths(paths: &[String]) -> Self {
        let mut deduped = BTreeSet::new();
        let hinted = paths
            .iter()
            .filter(|path| deduped.insert((*path).clone()))
            .map(|path| (path.clone(), None))
            .collect::<Vec<_>>();
        Self::from_hinted_paths(&hinted)
    }

    pub fn from_hinted_paths(paths: &[(String, Option<CorePerformanceArtifactKind>)]) -> Self {
        let mut deduped = BTreeSet::new();
        let mut descriptors = paths
            .iter()
            .filter(|(path, _)| deduped.insert(path.clone()))
            .map(|(path, hint)| describe_artifact(path, *hint))
            .collect::<Vec<_>>();
        finalize_descriptors(&mut descriptors)
    }

    pub fn from_resolved_entries(
        entries: &[(String, Option<CorePerformanceArtifactKind>, Option<Value>)],
    ) -> Self {
        let mut deduped = BTreeSet::new();
        let mut descriptors = entries
            .iter()
            .filter(|(path, _, _)| deduped.insert(path.clone()))
            .map(|(path, hint, value)| {
                value
                    .as_ref()
                    .map(|value| describe_artifact_from_value(path, *hint, Some(value)))
                    .unwrap_or_else(|| describe_artifact(path, *hint))
            })
            .collect::<Vec<_>>();
        finalize_descriptors(&mut descriptors)
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("official_artifact_count={}", self.official_artifact_count),
            format!(
                "non_crypto_official_count={}",
                self.non_crypto_official_count
            ),
            format!("crypto_only_count={}", self.crypto_only_count),
            format!("research_only_count={}", self.research_only_count),
            format!("fixture_only_count={}", self.fixture_only_count),
            format!("controlled_only_count={}", self.controlled_only_count),
            format!("outcome_linked_count={}", self.outcome_linked_count),
            format!("counterfactual_count={}", self.counterfactual_count),
            format!("baseline_reference_count={}", self.baseline_reference_count),
            format!("unknown_count={}", self.unknown_count),
        ];
        lines.extend(self.descriptors.iter().map(|descriptor| {
            format!(
                "path={};kind={:?};source_kind={};market={};symbol={};timeframe={};official={};non_crypto_official={};crypto_only={};research_only={};fixture_only={};controlled_only={};row_level_available={};outcome_linked_available={};counterfactual_available={};baseline_reference_available={};core_check_available={}",
                descriptor.path,
                descriptor.artifact_kind,
                descriptor.source_kind.map(|value| format!("{value:?}")).unwrap_or_default(),
                descriptor.market.map(|value| format!("{value:?}")).unwrap_or_default(),
                descriptor.symbol.clone().unwrap_or_default(),
                descriptor.timeframe.clone().unwrap_or_default(),
                descriptor.official,
                descriptor.non_crypto_official,
                descriptor.crypto_only,
                descriptor.research_only,
                descriptor.fixture_only,
                descriptor.controlled_only,
                descriptor.row_level_available,
                descriptor.outcome_linked_available,
                descriptor.counterfactual_available,
                descriptor.baseline_reference_available,
                descriptor.core_check_available,
            )
        }));
        lines.join("\n")
    }
}

fn finalize_descriptors(
    descriptors: &mut Vec<CorePerformanceArtifactDescriptor>,
) -> CorePerformanceArtifactInventory {
    descriptors.sort_by(|left, right| left.path.cmp(&right.path));
    let official_artifact_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.official)
        .count();
    let non_crypto_official_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.non_crypto_official)
        .count();
    let crypto_only_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.crypto_only)
        .count();
    let research_only_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.research_only)
        .count();
    let fixture_only_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.fixture_only)
        .count();
    let controlled_only_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.controlled_only)
        .count();
    let outcome_linked_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.outcome_linked_available)
        .count();
    let counterfactual_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.counterfactual_available)
        .count();
    let baseline_reference_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.baseline_reference_available)
        .count();
    let unknown_count = descriptors
        .iter()
        .filter(|descriptor| descriptor.artifact_kind == CorePerformanceArtifactKind::Unknown)
        .count();
    CorePerformanceArtifactInventory {
        descriptors: descriptors.clone(),
        official_artifact_count,
        non_crypto_official_count,
        crypto_only_count,
        research_only_count,
        fixture_only_count,
        controlled_only_count,
        outcome_linked_count,
        counterfactual_count,
        baseline_reference_count,
        unknown_count,
        reason_codes: stable_reason_codes(&[
            ReasonCode::CorePerformanceInventoryBuilt,
            ReasonCode::DeterministicPath,
        ]),
    }
}

fn describe_artifact(
    path: &str,
    hint: Option<CorePerformanceArtifactKind>,
) -> CorePerformanceArtifactDescriptor {
    let path_ref = Path::new(path);
    let bytes = fs::read(path_ref).ok();
    let text = bytes
        .as_deref()
        .map(String::from_utf8_lossy)
        .map(|value| value.to_string())
        .unwrap_or_default();
    let json = serde_json::from_str::<Value>(&text).ok();
    describe_artifact_from_inputs(path, hint, &text, json.as_ref(), path_ref.exists())
}

fn describe_artifact_from_value(
    path: &str,
    hint: Option<CorePerformanceArtifactKind>,
    value: Option<&Value>,
) -> CorePerformanceArtifactDescriptor {
    let text = value
        .and_then(|value| serde_json::to_string(value).ok())
        .unwrap_or_default();
    describe_artifact_from_inputs(path, hint, &text, value, Path::new(path).exists())
}

fn describe_artifact_from_inputs(
    path: &str,
    hint: Option<CorePerformanceArtifactKind>,
    text: &str,
    json: Option<&Value>,
    path_exists: bool,
) -> CorePerformanceArtifactDescriptor {
    let path_ref = Path::new(path);
    let lowered = path.to_ascii_lowercase();
    let artifact_kind = hint.unwrap_or_else(|| detect_kind(path_ref, &lowered, json));
    let json_has_non_crypto_official = json_non_crypto_official(json);
    let source_kind = detect_source_kind(
        &lowered,
        text,
        json,
        artifact_kind,
        json_has_non_crypto_official,
    );
    let market = detect_market(&lowered, text, json);
    let symbol = detect_symbol(path_ref, json);
    let timeframe = detect_timeframe(path_ref, text, json);
    let research_only = matches!(
        artifact_kind,
        CorePerformanceArtifactKind::YahooResearchEvidenceReport
    ) || matches!(source_kind, Some(EvidenceSourceKind::YFinanceResearch))
        || lowered.contains("yfinance")
        || lowered.contains("yahoo")
        || json_has_status(json, &["YFinanceResearchOnly", "ResearchOnly"]);
    let fixture_only = matches!(
        source_kind,
        Some(EvidenceSourceKind::SyntheticFixture | EvidenceSourceKind::TestFixture)
    ) || lowered.contains("fixture")
        || lowered.contains("mock")
        || json_has_status(json, &["FixtureOnly", "FixtureOnlyCoverage"]);
    let controlled_only = !json_has_non_crypto_official
        && (lowered.contains("controlled")
            || lowered.contains("diagnostics_only")
            || json_has_status(
                json,
                &[
                    "ControlledOnly",
                    "ControlledSufficiencyOnly",
                    "DiagnosticsOnly",
                    "ReferencePackDiagnosticOnly",
                ],
            )
            || text.contains("controlled_only_ratio")
            || text.contains("controlled evidence"));
    let crypto_only = market == Some(ProviderMarket::Crypto)
        || json_has_status(
            json,
            &["CryptoOnly", "CryptoOnlyCoverage", "CryptoOnlySufficiency"],
        );
    let row_level_available = matches!(
        artifact_kind,
        CorePerformanceArtifactKind::CommitteeReferencePackBundle
    ) || matches!(
        artifact_kind,
        CorePerformanceArtifactKind::OfficialReplicationReport
    ) || lowered.ends_with(".csv")
        || json_field_gt_zero(
            json,
            &["row_level_count", "row_level_rows", "official_row_count"],
        )
        || text.contains("row_level");
    let outcome_linked_available = json_field_gt_zero(
        json,
        &[
            "outcome_linked_rows",
            "outcome_link_count",
            "outcome_linked_count",
            "comparable_rows",
        ],
    ) || text.contains("outcome_linked_rows=")
        || text.contains("outcome_link_count=");
    let counterfactual_available = json_field_gt_zero(
        json,
        &[
            "no_trade_counterfactual_count",
            "risk_denied_counterfactual_count",
            "no_trade_counterfactuals",
            "risk_denied_counterfactuals",
            "built_count",
        ],
    ) || text.contains("counterfactual")
        || text.contains("avoided_loss_total");
    let baseline_reference_available = json_field_gt_zero(
        json,
        &[
            "baseline_reference_count",
            "baseline_linked_rows",
            "generated_baseline_count",
        ],
    ) || text.contains("baseline_reference_count=")
        || text.contains("baseline_linked_rows=");
    let core_check_available = artifact_kind == CorePerformanceArtifactKind::CoreCheckReport
        || json.is_some_and(|value| {
            value.get("runtime_state_report").is_some() && value.get("live_safety_report").is_some()
        });
    let official = (json_has_non_crypto_official
        || (!path.ends_with(".toml")
            && matches!(
                artifact_kind,
                CorePerformanceArtifactKind::OfficialReplicationReport
                    | CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport
                    | CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle
                    | CorePerformanceArtifactKind::CommitteeReferencePackBundle
                    | CorePerformanceArtifactKind::CommitteeBenchmarkBundle
            )))
        && !research_only
        && !fixture_only
        && !controlled_only
        && !crypto_only;
    let non_crypto_official = official && market != Some(ProviderMarket::Crypto);

    let mut reason_codes = vec![ReasonCode::CorePerformanceInventoryBuilt];
    if !path_exists {
        reason_codes.push(ReasonCode::MissingFile);
    }
    if path.contains("://") {
        reason_codes.push(ReasonCode::RemotePathRejected);
    }
    if artifact_kind == CorePerformanceArtifactKind::Unknown {
        reason_codes.push(ReasonCode::CommitteeArtifactUnknown);
    }
    if controlled_only {
        reason_codes.push(ReasonCode::ControlledOnlyEvidence);
    }
    if crypto_only {
        reason_codes.push(ReasonCode::CryptoOnlyEvidence);
    }
    if research_only {
        reason_codes.push(ReasonCode::YFinanceResearchOnly);
    }

    CorePerformanceArtifactDescriptor {
        path: path.to_string(),
        artifact_kind,
        source_kind,
        market,
        symbol,
        timeframe,
        official,
        non_crypto_official,
        crypto_only,
        research_only,
        fixture_only,
        controlled_only,
        row_level_available,
        outcome_linked_available,
        counterfactual_available,
        baseline_reference_available,
        core_check_available,
        reason_codes: stable_reason_codes(&reason_codes),
    }
}

fn detect_kind(path: &Path, lowered: &str, json: Option<&Value>) -> CorePerformanceArtifactKind {
    if lowered.contains("core_check")
        || lowered.contains("core_readiness")
        || json.is_some_and(|value| value.get("runtime_state_report").is_some())
    {
        CorePerformanceArtifactKind::CoreCheckReport
    } else if lowered.contains("official_replication")
        || json.is_some_and(|value| value.get("row_injection_result").is_some())
    {
        CorePerformanceArtifactKind::OfficialReplicationReport
    } else if lowered.contains("committee_official_benchmark")
        || json.is_some_and(|value| value.get("outcome_linked_vs_baseline_report").is_some())
        || json.is_some_and(|value| value.get("official_scenario_pack").is_some())
    {
        CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport
    } else if lowered.contains("outcome_coverage")
        || json.is_some_and(|value| value.get("coverage_report").is_some())
        || json.is_some_and(|value| value.get("performance_matrix").is_some())
        || json
            .is_some_and(|value| value.get("coverage_id").is_some() && value.get("cells").is_some())
    {
        CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle
    } else if lowered.contains("reference_pack")
        || json.is_some_and(|value| value.get("reference_pack").is_some())
        || json.is_some_and(|value| value.get("generated_references").is_some())
    {
        CorePerformanceArtifactKind::CommitteeReferencePackBundle
    } else if lowered.contains("committee_benchmark")
        || json.is_some_and(|value| {
            value.get("benchmark_report").is_some() && value.get("replay_report").is_some()
        })
    {
        CorePerformanceArtifactKind::CommitteeBenchmarkBundle
    } else if lowered.contains("source_benchmark")
        || lowered.contains("source_aware")
        || json.is_some_and(|value| value.get("dataset_inventory").is_some())
    {
        CorePerformanceArtifactKind::SourceAwareBenchmarkReport
    } else if lowered.contains("provider_readiness")
        || json.is_some_and(|value| {
            value.get("selection_results").is_some() && value.get("catalog").is_some()
        })
    {
        CorePerformanceArtifactKind::ProviderReadinessReport
    } else if lowered.contains("yahoo_research")
        || lowered.contains("yfinance")
        || json.is_some_and(|value| value.get("yfinance_symbols").is_some())
    {
        CorePerformanceArtifactKind::YahooResearchEvidenceReport
    } else if lowered.ends_with(".json") && path.exists() {
        CorePerformanceArtifactKind::Unknown
    } else {
        CorePerformanceArtifactKind::Unknown
    }
}

fn detect_source_kind(
    lowered: &str,
    text: &str,
    json: Option<&Value>,
    artifact_kind: CorePerformanceArtifactKind,
    json_has_non_crypto_official: bool,
) -> Option<EvidenceSourceKind> {
    if let Some(value) = json.and_then(|value| {
        value
            .get("source_kind")
            .and_then(Value::as_str)
            .or_else(|| value.get("provenance")?.get("source_kind")?.as_str())
            .or_else(|| {
                value
                    .get("artifact_inventory")?
                    .get("descriptors")?
                    .as_array()?
                    .first()?
                    .get("source_kind")?
                    .as_str()
            })
    }) {
        return Some(parse_source_kind(value));
    }
    if json_has_non_crypto_official {
        Some(EvidenceSourceKind::OfficialApiCollected)
    } else if lowered.contains("yfinance") || lowered.contains("yahoo") {
        Some(EvidenceSourceKind::YFinanceResearch)
    } else if lowered.contains("fixture") || lowered.contains("mock") {
        Some(EvidenceSourceKind::TestFixture)
    } else if lowered.contains("controlled") || lowered.contains("diagnostics_only") {
        Some(EvidenceSourceKind::RealLocal)
    } else if matches!(
        artifact_kind,
        CorePerformanceArtifactKind::OfficialReplicationReport
            | CorePerformanceArtifactKind::OfficialCommitteeBenchmarkReport
            | CorePerformanceArtifactKind::CommitteeOutcomeCoverageBundle
            | CorePerformanceArtifactKind::CommitteeReferencePackBundle
            | CorePerformanceArtifactKind::CommitteeBenchmarkBundle
    ) && !text.to_ascii_lowercase().contains("yfinance")
        && !lowered.ends_with(".toml")
    {
        Some(EvidenceSourceKind::OfficialApiCollected)
    } else {
        None
    }
}

fn json_non_crypto_official(json: Option<&Value>) -> bool {
    json_field_gt_zero(
        json,
        &[
            "non_crypto_official_row_count",
            "non_crypto_official_count",
            "official_ready_count",
            "official_row_count",
            "official_sufficiency_replication_report.non_crypto_official_row_count",
            "artifact_inventory.non_crypto_official_count",
        ],
    ) || json_has_status(
        json,
        &[
            "OfficialReplicationReady",
            "OfficialBenchmarkReady",
            "SufficiencyGatePassedForOfficialEvidence",
        ],
    )
}

fn detect_market(lowered: &str, text: &str, json: Option<&Value>) -> Option<ProviderMarket> {
    if json.is_some_and(|value| {
        value
            .get("market")
            .and_then(Value::as_str)
            .is_some_and(|value| value.eq_ignore_ascii_case("crypto"))
    }) || lowered.contains("btc")
        || lowered.contains("eth")
        || text.contains("Crypto")
    {
        Some(ProviderMarket::Crypto)
    } else if text.contains("KoreanEquity") || lowered.contains("krx") {
        Some(ProviderMarket::KoreanEquity)
    } else if text.contains("USEquity") || lowered.contains("aapl") || lowered.contains("msft") {
        Some(ProviderMarket::USEquity)
    } else {
        None
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
                    .get("yfinance_symbols")?
                    .as_array()?
                    .first()?
                    .as_str()
                    .map(|value| value.to_string())
            })
            .or_else(|| {
                value
                    .get("artifact_inventory")?
                    .get("descriptors")?
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
            .map(|stem| stem.to_string())
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
                    .get("artifact_inventory")?
                    .get("descriptors")?
                    .as_array()?
                    .first()?
                    .get("timeframe")?
                    .as_str()
                    .map(|value| value.to_string())
            })
    })
    .or_else(|| {
        let lowered = path.to_string_lossy().to_ascii_lowercase();
        if lowered.contains("1d") || text.contains("OneDay") {
            Some("OneDay".to_string())
        } else if lowered.contains("1m") || text.contains("OneMinute") {
            Some("OneMinute".to_string())
        } else {
            None
        }
    })
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
        "syntheticfixture" | "synthetic-fixture" => EvidenceSourceKind::SyntheticFixture,
        "testfixture" | "test-fixture" => EvidenceSourceKind::TestFixture,
        _ => EvidenceSourceKind::Unknown,
    }
}

fn json_has_status(json: Option<&Value>, statuses: &[&str]) -> bool {
    json.and_then(|value| {
        value
            .get("final_status")
            .and_then(Value::as_str)
            .or_else(|| value.get("current_official_status")?.as_str())
            .or_else(|| value.get("current_status")?.as_str())
            .or_else(|| value.get("status")?.as_str())
    })
    .is_some_and(|value| statuses.iter().any(|status| value == *status))
}

fn json_field_gt_zero(json: Option<&Value>, fields: &[&str]) -> bool {
    fields.iter().any(|field| {
        value_by_path(json, field)
            .and_then(as_usize)
            .is_some_and(|value| value > 0)
    })
}

fn value_by_path<'a>(json: Option<&'a Value>, path: &str) -> Option<&'a Value> {
    let value = json?;
    let mut current = value;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn as_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .map(|value| value as usize)
        .or_else(|| value.as_i64().map(|value| value.max(0) as usize))
}
