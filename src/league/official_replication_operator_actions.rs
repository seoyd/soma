use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};
use crate::data::{ProviderEntitlementStatusKind, ProviderKind, ProviderMarket};
use crate::experiment::{
    OfficialProviderReadinessReport, ProviderRealityReport, ProviderRealitySummary,
};

use super::official_candle_coverage::{OfficialCandleCoverageReport, OfficialCandleCoverageStatus};
use super::official_evidence_replication::OfficialEvidenceReplicationConfig;
use super::official_replication_inventory::{
    OfficialReplicationArtifactInventory, OfficialReplicationArtifactKind,
};
use super::official_sufficiency_replication::{
    OfficialSufficiencyReplicationReport, OfficialSufficiencyReplicationStatus,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OfficialReplicationActionPriority {
    Required,
    Recommended,
    Optional,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReplicationOperatorAction {
    pub action_id: String,
    pub priority: OfficialReplicationActionPriority,
    #[serde(default)]
    pub provider_kind: Option<ProviderKind>,
    #[serde(default)]
    pub market: Option<ProviderMarket>,
    pub description: String,
    pub env_var_names: Vec<String>,
    #[serde(default)]
    pub command_suggestion: Option<String>,
    #[serde(default)]
    pub expected_output_artifact: Option<String>,
    pub safe_to_run: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficialReplicationOperatorActionPlan {
    pub actions: Vec<OfficialReplicationOperatorAction>,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OfficialReplicationOperatorActionPlanner;

impl OfficialReplicationOperatorActionPlanner {
    pub fn build(
        &self,
        _config: &OfficialEvidenceReplicationConfig,
        inventory: &OfficialReplicationArtifactInventory,
        candle_coverage: Option<&OfficialCandleCoverageReport>,
        sufficiency: Option<&OfficialSufficiencyReplicationReport>,
    ) -> OfficialReplicationOperatorActionPlan {
        let readiness_reports = inventory
            .descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.artifact_kind == OfficialReplicationArtifactKind::ProviderReadinessReport
            })
            .filter_map(|descriptor| {
                OfficialProviderReadinessReport::from_json_path(std::path::Path::new(
                    &descriptor.path,
                ))
                .ok()
            })
            .collect::<Vec<_>>();
        let reality_reports = inventory
            .descriptors
            .iter()
            .filter(|descriptor| {
                descriptor.artifact_kind == OfficialReplicationArtifactKind::ProviderRealityReport
            })
            .filter_map(|descriptor| {
                ProviderRealityReport::from_json_path(std::path::Path::new(&descriptor.path)).ok()
            })
            .collect::<Vec<_>>();
        let mut actions = Vec::new();
        let mut blockers = Vec::new();

        if readiness_reports.is_empty() {
            actions.push(action(
                "RunProviderReadiness",
                OfficialReplicationActionPriority::Recommended,
                None,
                None,
                "Run provider readiness to enumerate bounded official prerequisites.",
                &[],
                Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                Some("provider_readiness_report.json"),
                vec![ReasonCode::ProviderReadinessReportBuilt],
            ));
        }
        if reality_reports.is_empty() {
            actions.push(action(
                "RunProviderReality",
                OfficialReplicationActionPriority::Recommended,
                None,
                None,
                "Run provider reality to confirm entitlement, approval, and market coverage gaps.",
                &[],
                Some("cargo run --bin soma_experiment -- provider-reality --config examples/soma_provider_reality.toml"),
                Some("provider_reality_report.json"),
                vec![ReasonCode::ProviderRealityReportBuilt],
            ));
        }

        for report in &readiness_reports {
            for line in &report.missing_auth_actions {
                let lowered = line.to_ascii_lowercase();
                if lowered.contains("krx") {
                    actions.push(action(
                        "SetKrxApiKey",
                        OfficialReplicationActionPriority::Required,
                        Some(ProviderKind::KrxOpenApi),
                        Some(ProviderMarket::KoreanEquity),
                        "Set the bounded KRX API key env var locally before claiming Korean official evidence.",
                        &["KRX_API_KEY"],
                        Some("cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"),
                        Some("provider_auth_preflight_report.json"),
                        vec![ReasonCode::MissingAuth],
                    ));
                }
                if lowered.contains("alphavantage") {
                    actions.push(action(
                        "SetAlphaVantageApiKey",
                        OfficialReplicationActionPriority::Required,
                        Some(ProviderKind::AlphaVantage),
                        Some(ProviderMarket::USEquity),
                        "Set the bounded AlphaVantage API key env var locally before claiming US official evidence.",
                        &["ALPHAVANTAGE_API_KEY"],
                        Some("cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"),
                        Some("provider_auth_preflight_report.json"),
                        vec![ReasonCode::MissingAuth],
                    ));
                }
                if lowered.contains("data-go-kr") {
                    actions.push(action(
                        "SetDataGoKrServiceKey",
                        OfficialReplicationActionPriority::Recommended,
                        Some(ProviderKind::DataGoKrFscStockPrice),
                        Some(ProviderMarket::KoreanEquity),
                        "Set the data.go.kr service key locally for bounded Korean fallback collection.",
                        &["DATA_GO_KR_SERVICE_KEY"],
                        Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                        Some("provider_readiness_report.json"),
                        vec![ReasonCode::MissingAuth],
                    ));
                }
            }
            if matches!(
                report.final_status,
                crate::experiment::OfficialProviderReadinessStatus::MissingProviderEndpointProfile
            ) {
                actions.push(action(
                    "SetKrxEndpointTemplate",
                    OfficialReplicationActionPriority::Required,
                    Some(ProviderKind::KrxOpenApi),
                    Some(ProviderMarket::KoreanEquity),
                    "Set the bounded KRX endpoint template env var locally before official collection.",
                    &["KRX_ENDPOINT_TEMPLATE"],
                    Some("cargo run --bin soma_experiment -- provider-auth-check --config examples/soma_provider_auth_preflight.toml"),
                    Some("provider_auth_preflight_report.json"),
                    vec![ReasonCode::MissingEndpointTemplate],
                ));
            }
        }

        for report in &reality_reports {
            if report
                .final_summary
                .iter()
                .any(|summary| *summary == ProviderRealitySummary::KRXApprovalPending)
                || report.entitlement_statuses.iter().any(|status| {
                    status.provider_subject
                        == crate::data::ProviderDataSubject::Provider(ProviderKind::KrxOpenApi)
                        && !status.approval_ready
                })
            {
                actions.push(action(
                    "WaitForKrxApproval",
                    OfficialReplicationActionPriority::Required,
                    Some(ProviderKind::KrxOpenApi),
                    Some(ProviderMarket::KoreanEquity),
                    "Wait for KRX approval before claiming Korean official evidence readiness.",
                    &[],
                    Some("cargo run --bin soma_experiment -- provider-reality --config examples/soma_provider_reality.toml"),
                    Some("provider_reality_report.json"),
                    vec![ReasonCode::MissingApproval],
                ));
            }
            if report.entitlement_statuses.iter().any(|status| {
                status.provider_subject
                    == crate::data::ProviderDataSubject::Provider(ProviderKind::Alpaca)
                    && matches!(status.status, ProviderEntitlementStatusKind::MissingAuth)
            }) {
                actions.push(action(
                    "SetAlpacaKeys",
                    OfficialReplicationActionPriority::Optional,
                    Some(ProviderKind::Alpaca),
                    Some(ProviderMarket::USEquity),
                    "Set bounded Alpaca market-data env vars locally for research-only US realtime comparisons.",
                    &["ALPACA_API_KEY_ID", "ALPACA_API_SECRET_KEY"],
                    Some("cargo run --bin soma_experiment -- provider-readiness --config examples/soma_provider_readiness.toml"),
                    Some("provider_readiness_report.json"),
                    vec![ReasonCode::MissingAuth],
                ));
            }
        }

        if inventory.non_crypto_official_artifact_count == 0 {
            blockers.push("no true non-crypto official artifacts are available".to_string());
            actions.push(action(
                "RunOfficialAcquire",
                OfficialReplicationActionPriority::Required,
                None,
                None,
                "Run bounded official acquisition or provide already-collected official canonical CSV artifacts.",
                &[],
                Some("cargo run --bin soma_experiment -- official-acquire --config examples/soma_official_evidence_acquisition.toml"),
                Some("official_collection_report.json"),
                vec![ReasonCode::MissingOfficialData],
            ));
        }
        if inventory.missing_provenance_count > 0 {
            blockers.push("official provenance artifacts are missing".to_string());
            actions.push(action(
                "RunEvidenceExecute",
                OfficialReplicationActionPriority::Required,
                None,
                None,
                "Run bounded evidence execution or provide operator-supplied provenance manifests for official rows.",
                &[],
                Some("cargo run --bin soma_experiment -- evidence-execute --config examples/soma_evidence_plan_eod_official.toml"),
                Some("official_provenance.json"),
                vec![ReasonCode::MissingOfficialProvenance],
            ));
        }
        if inventory.missing_preflight_count > 0 {
            blockers.push("official preflight artifacts are missing".to_string());
            actions.push(action(
                "RunCommitteeOutcomeCoverage",
                OfficialReplicationActionPriority::Recommended,
                None,
                None,
                "Run conservative outcome coverage / preflight validation before claiming official closure.",
                &[],
                Some("cargo run --bin soma_experiment -- committee-outcome-coverage --config examples/soma_committee_outcome_coverage_controlled.toml"),
                Some("preflight_report.json"),
                vec![ReasonCode::MissingOfficialPreflight],
            ));
        }
        if candle_coverage.is_some_and(|report| {
            matches!(
                report.coverage_status,
                OfficialCandleCoverageStatus::MissingOfficialCandles
                    | OfficialCandleCoverageStatus::MissingFutureWindow
                    | OfficialCandleCoverageStatus::InsufficientCoverage
            )
        }) || inventory.missing_candle_count > 0
        {
            blockers.push("local official candles are missing or insufficient".to_string());
            actions.push(action(
                "ProvideOfficialCandleSeries",
                OfficialReplicationActionPriority::Required,
                None,
                None,
                "Provide bounded local official candle series or canonical CSV files for every official row under review.",
                &[],
                Some("cargo run --bin soma_experiment -- official-artifact-inventory --config examples/soma_official_artifact_inventory.toml"),
                Some("*_candles.json"),
                vec![ReasonCode::MissingOfficialCandles],
            ));
        }
        if !inventory.descriptors.iter().any(|descriptor| {
            descriptor.artifact_kind == OfficialReplicationArtifactKind::OfficialCommitteePack
        }) {
            actions.push(action(
                "RunCommitteePackOfficial",
                OfficialReplicationActionPriority::Recommended,
                None,
                None,
                "Materialize a bounded official committee pack once official rows are locally available.",
                &[],
                Some("cargo run --bin soma_experiment -- committee-pack-official --config examples/soma_committee_pack_controlled_official.toml"),
                Some("official_scenario_pack.json"),
                vec![ReasonCode::OfficialCommitteePackBuilt],
            ));
        }
        if !inventory.descriptors.iter().any(|descriptor| {
            descriptor.artifact_kind == OfficialReplicationArtifactKind::GeneratedReferencePack
        }) {
            actions.push(action(
                "RunCommitteeBuildReferences",
                OfficialReplicationActionPriority::Recommended,
                None,
                None,
                "Build bounded committee references after official rows and local candles are ready.",
                &[],
                Some("cargo run --bin soma_experiment -- committee-build-references --config examples/soma_committee_build_references_controlled.toml"),
                Some("generated_reference_pack.json"),
                vec![ReasonCode::CommitteeReferencePackBuilt],
            ));
        }
        if sufficiency.is_some_and(|report| {
            matches!(
                report.final_status,
                OfficialSufficiencyReplicationStatus::OfficialSufficiencyPassed
            )
        }) {
            actions.push(action(
                "RunCommitteeOfficialBenchmark",
                OfficialReplicationActionPriority::Recommended,
                None,
                None,
                "Rerun the bounded official committee benchmark only after official sufficiency is satisfied.",
                &[],
                Some("cargo run --bin soma_experiment -- committee-official-benchmark --config examples/soma_committee_official_benchmark_controlled.toml"),
                Some("committee_official_benchmark_bundle.json"),
                vec![ReasonCode::CommitteeOfficialBenchmarkBuilt],
            ));
        }
        if inventory.official_artifact_count > 0 && inventory.descriptors.len() > 5 {
            actions.push(action(
                "ReduceScope",
                OfficialReplicationActionPriority::Optional,
                None,
                None,
                "Reduce rows/symbols to the smallest bounded official scope before escalating more collection.",
                &[],
                None,
                None,
                vec![ReasonCode::RowLimitApplied],
            ));
        }

        actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
        actions.dedup_by(|left, right| left.action_id == right.action_id);
        OfficialReplicationOperatorActionPlan {
            actions,
            blockers: dedup_strings(blockers),
            warnings: vec![
                "All Sprint 39 operator actions remain local-only, research-only, and never include secret values."
                    .to_string(),
            ],
            reason_codes: stable_reason_codes(&[
                ReasonCode::OfficialReplicationOperatorActionsBuilt,
                ReasonCode::OperatorActionPlanBuilt,
            ]),
        }
    }
}

impl OfficialReplicationOperatorActionPlan {
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("blockers={}", self.blockers.join(" | ")),
            format!("warnings={}", self.warnings.join(" | ")),
        ];
        lines.extend(self.actions.iter().map(|action| {
            format!(
                "action_id={};priority={:?};provider={};market={};env_var_names={};safe_to_run={};expected_output_artifact={};command={}",
                action.action_id,
                action.priority,
                action.provider_kind.map(|value| format!("{value:?}")).unwrap_or_default(),
                action.market.map(|value| format!("{value:?}")).unwrap_or_default(),
                action.env_var_names.join("|"),
                action.safe_to_run,
                action.expected_output_artifact.clone().unwrap_or_default(),
                action.command_suggestion.clone().unwrap_or_default(),
            )
        }));
        lines.join("\n")
    }
}

fn action(
    action_id: &str,
    priority: OfficialReplicationActionPriority,
    provider_kind: Option<ProviderKind>,
    market: Option<ProviderMarket>,
    description: &str,
    env_var_names: &[&str],
    command_suggestion: Option<&str>,
    expected_output_artifact: Option<&str>,
    reason_codes: Vec<ReasonCode>,
) -> OfficialReplicationOperatorAction {
    OfficialReplicationOperatorAction {
        action_id: action_id.to_string(),
        priority,
        provider_kind,
        market,
        description: description.to_string(),
        env_var_names: env_var_names
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        command_suggestion: command_suggestion.map(|value| value.to_string()),
        expected_output_artifact: expected_output_artifact.map(|value| value.to_string()),
        safe_to_run: true,
        reason_codes: stable_reason_codes(
            &reason_codes
                .into_iter()
                .chain([ReasonCode::OfficialReplicationOperatorActionsBuilt])
                .collect::<Vec<_>>(),
        ),
    }
}

fn dedup_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = BTreeSet::new();
    for value in values {
        deduped.insert(value);
    }
    deduped.into_iter().collect()
}
