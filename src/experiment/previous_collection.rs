use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::ReasonCode;
use crate::data::{
    OfficialCollectionEntryStatus, OfficialCollectionReport, ProviderAuthPreflightReport,
    ProviderKind,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreviousCollectionComparison {
    pub previous_report_loaded: bool,
    pub comparable: bool,
    pub previous_ready_entries: Vec<String>,
    pub current_ready_entries: Vec<String>,
    pub added_ready_entries: Vec<String>,
    pub removed_ready_entries: Vec<String>,
    pub previous_missing_auth: Vec<String>,
    pub current_missing_auth: Vec<String>,
    pub fixed_missing_auth: Vec<String>,
    pub new_missing_auth: Vec<String>,
    pub reason_codes: Vec<ReasonCode>,
}

impl PreviousCollectionComparison {
    pub fn to_text(&self) -> String {
        [
            format!("previous_report_loaded={}", self.previous_report_loaded),
            format!("comparable={}", self.comparable),
            format!(
                "previous_ready_entries={}",
                self.previous_ready_entries.join("|")
            ),
            format!(
                "current_ready_entries={}",
                self.current_ready_entries.join("|")
            ),
            format!("added_ready_entries={}", self.added_ready_entries.join("|")),
            format!(
                "removed_ready_entries={}",
                self.removed_ready_entries.join("|")
            ),
            format!(
                "previous_missing_auth={}",
                self.previous_missing_auth.join("|")
            ),
            format!(
                "current_missing_auth={}",
                self.current_missing_auth.join("|")
            ),
            format!("fixed_missing_auth={}", self.fixed_missing_auth.join("|")),
            format!("new_missing_auth={}", self.new_missing_auth.join("|")),
        ]
        .join("\n")
    }
}

pub fn load_previous_collection_report(
    path: Option<&str>,
) -> Result<(Option<OfficialCollectionReport>, Vec<ReasonCode>), String> {
    let Some(path) = path else {
        return Ok((None, Vec::new()));
    };
    let path = Path::new(path);
    if !path.exists() {
        return Ok((None, vec![ReasonCode::MissingFile]));
    }
    OfficialCollectionReport::from_json_path(path)
        .map(|report| {
            (
                Some(report),
                vec![
                    ReasonCode::PreviousCollectionComparisonBuilt,
                    ReasonCode::PreviousCollectionReportLoaded,
                ],
            )
        })
        .map_err(|err| err.to_string())
}

pub fn build_previous_collection_comparison(
    previous_report: Option<&OfficialCollectionReport>,
    current_report: Option<&OfficialCollectionReport>,
    current_auth_preflight: Option<&ProviderAuthPreflightReport>,
    previous_requested: bool,
    load_reason_codes: &[ReasonCode],
) -> PreviousCollectionComparison {
    let previous_ready_entries = ready_entries(previous_report);
    let current_ready_entries = ready_entries(current_report);
    let previous_missing_auth = missing_auth(previous_report, None);
    let current_missing_auth = missing_auth(current_report, current_auth_preflight);

    let previous_ready_set = previous_ready_entries
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let current_ready_set = current_ready_entries
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let previous_missing_set = previous_missing_auth
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let current_missing_set = current_missing_auth
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();

    let added_ready_entries = current_ready_set
        .difference(&previous_ready_set)
        .cloned()
        .collect::<Vec<_>>();
    let removed_ready_entries = previous_ready_set
        .difference(&current_ready_set)
        .cloned()
        .collect::<Vec<_>>();
    let fixed_missing_auth = previous_missing_set
        .difference(&current_missing_set)
        .cloned()
        .collect::<Vec<_>>();
    let new_missing_auth = current_missing_set
        .difference(&previous_missing_set)
        .cloned()
        .collect::<Vec<_>>();

    let mut reason_codes = vec![ReasonCode::PreviousCollectionComparisonBuilt];
    reason_codes.extend(load_reason_codes.iter().cloned());
    if previous_requested && previous_report.is_none() {
        reason_codes.push(ReasonCode::MissingFile);
    }

    PreviousCollectionComparison {
        previous_report_loaded: previous_report.is_some(),
        comparable: previous_report.is_some() && current_report.is_some(),
        previous_ready_entries,
        current_ready_entries,
        added_ready_entries,
        removed_ready_entries,
        previous_missing_auth,
        current_missing_auth,
        fixed_missing_auth,
        new_missing_auth,
        reason_codes: dedupe_reasons(reason_codes),
    }
}

fn ready_entries(report: Option<&OfficialCollectionReport>) -> Vec<String> {
    let mut entries = report
        .map(|report| {
            report
                .entry_reports
                .iter()
                .filter(|entry| entry.ready_for_evidence)
                .map(|entry| entry.entry_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    entries.sort();
    entries
}

fn missing_auth(
    report: Option<&OfficialCollectionReport>,
    auth_preflight: Option<&ProviderAuthPreflightReport>,
) -> Vec<String> {
    let mut providers = report
        .map(|report| {
            report
                .entry_reports
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.status,
                        OfficialCollectionEntryStatus::SkippedMissingAuth
                    )
                })
                .map(|entry| provider_label(entry.provider_kind))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    if let Some(auth_preflight) = auth_preflight {
        for provider in auth_preflight
            .missing_auth_providers
            .iter()
            .chain(auth_preflight.missing_endpoint_providers.iter())
        {
            providers.insert(provider.clone());
        }
    }
    providers.into_iter().collect()
}

fn provider_label(provider_kind: ProviderKind) -> String {
    match provider_kind {
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
    .to_string()
}

fn dedupe_reasons(reason_codes: Vec<ReasonCode>) -> Vec<ReasonCode> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for reason in reason_codes {
        if seen.insert(format!("{reason:?}")) {
            deduped.push(reason);
        }
    }
    deduped
}
