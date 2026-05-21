use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    OfficialCollectionReport, ProviderAuthPreflightReport, ProviderAuthStatusKind, ProviderKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VenueGroup {
    Crypto,
    KoreanEquity,
    USEquity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VenueCoverageTarget {
    pub venue_group: VenueGroup,
    pub min_ready_datasets: usize,
    pub min_outcome_records: usize,
    pub min_symbols: usize,
    pub min_timeframes: usize,
    pub required: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VenueCoverageExpansionPlan {
    pub plan_id: String,
    pub targets: Vec<VenueCoverageTarget>,
    #[serde(default)]
    pub collection_plan_path: Option<String>,
    #[serde(default)]
    pub existing_collection_report_path: Option<String>,
    pub max_total_symbols: usize,
    pub max_total_rows: usize,
    pub max_total_requests: usize,
    pub max_total_bytes: usize,
    #[serde(default)]
    pub allow_crypto_only: bool,
    #[serde(default = "default_true")]
    pub allow_missing_equity_auth: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VenueCoverageTargetResult {
    pub venue_group: VenueGroup,
    pub ready_datasets: usize,
    pub outcome_records: usize,
    pub symbol_count: usize,
    pub timeframe_count: usize,
    pub auth_blocked: bool,
    pub passed: bool,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VenueCoverageStatus {
    NoOfficialData,
    CryptoOnly,
    CryptoAndKoreanEquity,
    CryptoAndUSEquity,
    MultiVenuePartial,
    MultiVenueReady,
    MissingAuth,
    NeedMoreOfficialData,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VenueCoverageExpansionReport {
    pub plan_id: String,
    pub target_results: Vec<VenueCoverageTargetResult>,
    pub crypto_status: String,
    pub korean_equity_status: String,
    pub us_equity_status: String,
    pub missing_auth_summary: Vec<String>,
    pub skipped_summary: Vec<String>,
    pub coverage_status: VenueCoverageStatus,
    pub warnings: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for VenueCoverageExpansionPlan {
    fn default() -> Self {
        Self {
            plan_id: "venue_coverage_targets".to_string(),
            targets: vec![
                VenueCoverageTarget {
                    venue_group: VenueGroup::Crypto,
                    min_ready_datasets: 1,
                    min_outcome_records: 20,
                    min_symbols: 1,
                    min_timeframes: 1,
                    required: true,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                },
                VenueCoverageTarget {
                    venue_group: VenueGroup::KoreanEquity,
                    min_ready_datasets: 1,
                    min_outcome_records: 20,
                    min_symbols: 1,
                    min_timeframes: 1,
                    required: false,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                },
                VenueCoverageTarget {
                    venue_group: VenueGroup::USEquity,
                    min_ready_datasets: 1,
                    min_outcome_records: 20,
                    min_symbols: 1,
                    min_timeframes: 1,
                    required: false,
                    reason_codes: vec![ReasonCode::DeterministicPath],
                },
            ],
            collection_plan_path: None,
            existing_collection_report_path: None,
            max_total_symbols: 3,
            max_total_rows: 1500,
            max_total_requests: 10,
            max_total_bytes: 16 * 1024 * 1024,
            allow_crypto_only: false,
            allow_missing_equity_auth: true,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl VenueCoverageExpansionPlan {
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

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let mut reasons = Vec::new();
        for path in [
            self.collection_plan_path.as_deref(),
            self.existing_collection_report_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if path.contains("://") {
                reasons.push(ReasonCode::RemotePathRejected);
            }
        }
        dedupe_reasons(reasons)
    }
}

impl VenueCoverageExpansionReport {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("plan_id={}", self.plan_id),
            format!("coverage_status={:?}", self.coverage_status),
            format!("crypto_status={}", self.crypto_status),
            format!("korean_equity_status={}", self.korean_equity_status),
            format!("us_equity_status={}", self.us_equity_status),
            format!(
                "missing_auth_summary={}",
                self.missing_auth_summary.join("|")
            ),
            format!("skipped_summary={}", self.skipped_summary.join("|")),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        for result in &self.target_results {
            lines.push(format!(
                "target={:?};ready_datasets={};outcome_records={};symbol_count={};timeframe_count={};auth_blocked={};passed={}",
                result.venue_group,
                result.ready_datasets,
                result.outcome_records,
                result.symbol_count,
                result.timeframe_count,
                result.auth_blocked,
                result.passed,
            ));
        }
        lines.join("\n")
    }
}

pub fn build_venue_coverage_report(
    plan: &VenueCoverageExpansionPlan,
    collection_report: Option<&OfficialCollectionReport>,
    auth_preflight_report: Option<&ProviderAuthPreflightReport>,
) -> VenueCoverageExpansionReport {
    let mut rows_by_group = BTreeMap::<VenueGroup, usize>::new();
    let mut symbols_by_group = BTreeMap::<VenueGroup, BTreeSet<String>>::new();
    let mut timeframes_by_group = BTreeMap::<VenueGroup, BTreeSet<String>>::new();
    let mut skipped_summary = Vec::new();
    let mut missing_auth_summary = auth_preflight_report
        .map(|report| {
            let mut providers = report.missing_auth_providers.clone();
            providers.extend(report.missing_endpoint_providers.clone());
            providers.sort();
            providers.dedup();
            providers
        })
        .unwrap_or_default();

    if let Some(report) = collection_report {
        for entry in &report.entry_reports {
            if entry.provider_kind == ProviderKind::MockFixture {
                skipped_summary.push(format!("excluded-non-official:{}", entry.entry_id));
                continue;
            }
            let Some(group) = venue_group_for_provider(entry.provider_kind) else {
                continue;
            };
            if entry.ready_for_evidence {
                *rows_by_group.entry(group).or_insert(0) += entry.row_count;
                symbols_by_group
                    .entry(group)
                    .or_default()
                    .insert(entry.symbol.clone());
                timeframes_by_group
                    .entry(group)
                    .or_default()
                    .insert(format!("{:?}", entry.timeframe));
            } else if matches!(
                entry.status,
                crate::data::OfficialCollectionEntryStatus::SkippedMissingAuth
            ) {
                skipped_summary.push(format!("missing-auth:{}", entry.entry_id));
                let provider = match entry.provider_kind {
                    ProviderKind::KrxOpenApi
                    | ProviderKind::DataGoKrFscStockPrice
                    | ProviderKind::KoreaInvestmentMarketData
                    | ProviderKind::KoscomProfessional => "krx",
                    ProviderKind::AlphaVantage
                    | ProviderKind::PolygonProfessional
                    | ProviderKind::NasdaqDataLink => "alphavantage",
                    ProviderKind::Alpaca => "alpaca",
                    ProviderKind::Upbit => "upbit",
                    ProviderKind::Binance | ProviderKind::Korbit => "binance",
                    ProviderKind::MockFixture => "mock-fixture",
                    ProviderKind::Unknown => "unknown",
                }
                .to_string();
                if !missing_auth_summary.contains(&provider) {
                    missing_auth_summary.push(provider);
                }
            }
        }
    }

    let mut target_results = plan
        .targets
        .iter()
        .map(|target| {
            let ready_datasets = symbols_by_group
                .get(&target.venue_group)
                .map(|symbols| symbols.len())
                .unwrap_or(0);
            let outcome_records = rows_by_group.get(&target.venue_group).copied().unwrap_or(0);
            let symbol_count = symbols_by_group
                .get(&target.venue_group)
                .map(|symbols| symbols.len())
                .unwrap_or(0);
            let timeframe_count = timeframes_by_group
                .get(&target.venue_group)
                .map(|values| values.len())
                .unwrap_or(0);
            let auth_blocked = auth_blocks_target(target.venue_group, auth_preflight_report);
            let passed = !auth_blocked
                && ready_datasets >= target.min_ready_datasets
                && outcome_records >= target.min_outcome_records
                && symbol_count >= target.min_symbols
                && timeframe_count >= target.min_timeframes;
            let mut warnings = Vec::new();
            let mut reason_codes = vec![ReasonCode::VenueCoveragePlanBuilt];
            if ready_datasets > 0 && symbol_count <= 1 && target.min_symbols > 1 {
                warnings.push("one-symbol evidence is still weak".to_string());
                reason_codes.push(ReasonCode::VenueCoverageWeakEvidence);
            }
            if auth_blocked {
                warnings.push("auth availability blocks this venue target".to_string());
                reason_codes.push(ReasonCode::MissingAuth);
            }
            VenueCoverageTargetResult {
                venue_group: target.venue_group,
                ready_datasets,
                outcome_records,
                symbol_count,
                timeframe_count,
                auth_blocked,
                passed,
                warnings,
                reason_codes: dedupe_reasons(reason_codes),
            }
        })
        .collect::<Vec<_>>();
    target_results.sort_by(|left, right| left.venue_group.cmp(&right.venue_group));

    let crypto_ready = target_results
        .iter()
        .find(|result| result.venue_group == VenueGroup::Crypto)
        .is_some_and(|result| result.ready_datasets > 0);
    let korean_ready = target_results
        .iter()
        .find(|result| result.venue_group == VenueGroup::KoreanEquity)
        .is_some_and(|result| result.ready_datasets > 0);
    let us_ready = target_results
        .iter()
        .find(|result| result.venue_group == VenueGroup::USEquity)
        .is_some_and(|result| result.ready_datasets > 0);
    let required_targets_passed =
        plan.targets
            .iter()
            .filter(|target| target.required)
            .all(|target| {
                target_results
                    .iter()
                    .find(|result| result.venue_group == target.venue_group)
                    .is_some_and(|result| result.passed)
            });

    let coverage_status = if !missing_auth_summary.is_empty()
        && !plan.allow_missing_equity_auth
        && !required_targets_passed
    {
        VenueCoverageStatus::MissingAuth
    } else if !crypto_ready && !korean_ready && !us_ready {
        if !missing_auth_summary.is_empty() {
            VenueCoverageStatus::MissingAuth
        } else {
            VenueCoverageStatus::NoOfficialData
        }
    } else if crypto_ready && !korean_ready && !us_ready {
        VenueCoverageStatus::CryptoOnly
    } else if crypto_ready && korean_ready && !us_ready {
        if required_targets_passed {
            VenueCoverageStatus::CryptoAndKoreanEquity
        } else {
            VenueCoverageStatus::MultiVenuePartial
        }
    } else if crypto_ready && !korean_ready && us_ready {
        if required_targets_passed {
            VenueCoverageStatus::CryptoAndUSEquity
        } else {
            VenueCoverageStatus::MultiVenuePartial
        }
    } else if required_targets_passed {
        VenueCoverageStatus::MultiVenueReady
    } else {
        VenueCoverageStatus::NeedMoreOfficialData
    };

    let mut warnings = target_results
        .iter()
        .flat_map(|result| result.warnings.clone())
        .collect::<Vec<_>>();
    missing_auth_summary.sort();
    if matches!(coverage_status, VenueCoverageStatus::CryptoOnly) && !plan.allow_crypto_only {
        warnings.push("official coverage remains crypto-only".to_string());
    }
    let mut reason_codes = vec![ReasonCode::VenueCoveragePlanBuilt];
    if matches!(coverage_status, VenueCoverageStatus::CryptoOnly) {
        reason_codes.push(ReasonCode::BenchmarkCryptoOnlyEvidence);
    }
    if target_results.iter().any(|result| result.auth_blocked) {
        reason_codes.push(ReasonCode::MissingAuth);
    }

    VenueCoverageExpansionReport {
        plan_id: plan.plan_id.clone(),
        crypto_status: target_status_line(&target_results, VenueGroup::Crypto),
        korean_equity_status: target_status_line(&target_results, VenueGroup::KoreanEquity),
        us_equity_status: target_status_line(&target_results, VenueGroup::USEquity),
        target_results,
        missing_auth_summary,
        skipped_summary,
        coverage_status,
        warnings,
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn venue_group_for_provider(provider_kind: ProviderKind) -> Option<VenueGroup> {
    match provider_kind {
        ProviderKind::Upbit | ProviderKind::Binance | ProviderKind::Korbit => {
            Some(VenueGroup::Crypto)
        }
        ProviderKind::KrxOpenApi
        | ProviderKind::DataGoKrFscStockPrice
        | ProviderKind::KoreaInvestmentMarketData
        | ProviderKind::KoscomProfessional => Some(VenueGroup::KoreanEquity),
        ProviderKind::AlphaVantage
        | ProviderKind::Alpaca
        | ProviderKind::PolygonProfessional
        | ProviderKind::NasdaqDataLink => Some(VenueGroup::USEquity),
        ProviderKind::MockFixture | ProviderKind::Unknown => None,
    }
}

fn auth_blocks_target(
    venue_group: VenueGroup,
    auth_preflight_report: Option<&ProviderAuthPreflightReport>,
) -> bool {
    let Some(report) = auth_preflight_report else {
        return false;
    };
    let providers = report
        .statuses
        .iter()
        .filter(|status| match venue_group {
            VenueGroup::Crypto => status.provider_kind == ProviderKind::Upbit,
            VenueGroup::KoreanEquity => {
                matches!(
                    status.provider_kind,
                    ProviderKind::KrxOpenApi | ProviderKind::KoreaInvestmentMarketData
                )
            }
            VenueGroup::USEquity => {
                matches!(
                    status.provider_kind,
                    ProviderKind::AlphaVantage | ProviderKind::Alpaca
                )
            }
        })
        .collect::<Vec<_>>();
    !providers.is_empty()
        && providers.iter().all(|status| {
            matches!(
                status.status,
                ProviderAuthStatusKind::MissingAuth
                    | ProviderAuthStatusKind::MissingEndpointTemplate
                    | ProviderAuthStatusKind::Deferred
                    | ProviderAuthStatusKind::UnsafeSecretExposure
            )
        })
}

fn target_status_line(results: &[VenueCoverageTargetResult], venue_group: VenueGroup) -> String {
    results
        .iter()
        .find(|result| result.venue_group == venue_group)
        .map(|result| {
            format!(
                "passed={};ready_datasets={};outcomes={};symbols={};timeframes={};auth_blocked={}",
                result.passed,
                result.ready_datasets,
                result.outcome_records,
                result.symbol_count,
                result.timeframe_count,
                result.auth_blocked
            )
        })
        .unwrap_or_else(|| "missing".to_string())
}

fn default_true() -> bool {
    true
}

fn dedupe_reasons(values: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
