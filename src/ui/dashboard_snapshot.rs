use std::fs;

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::core::{ReasonCode, stable_ordered_strings, stable_reason_codes};
use crate::experiment::{
    ProviderPriorityMode, ProviderSimplificationFinalStatus, ProviderSimplificationReport,
};
use crate::owner::{
    AllowedOwnerAction, OwnerDecisionImpactKind, OwnerDecisionImpactReport, OwnerInput,
    OwnerReviewQueue, OwnerThesisBook, build_owner_candidate_feedback,
    build_owner_decision_impact_report, build_owner_review_queue, load_owner_inputs_from_path,
    load_owner_thesis_notes_from_path, validate_owner_input,
};

use super::dashboard_events::{
    AuditTimelinePanel, DashboardEvent, DashboardEventKind, DashboardEventSeverity,
};
use super::dashboard_panels::{
    BottleneckPanel, CandidatePanel, CandidateStatus, ChairPanel, CommitteePanel,
    DashboardKisStatus, DashboardKrxStatus, DashboardNamedProviderStatus, EvidencePanel,
    HumanConfirmForbiddenAction, HumanConfirmItem, HumanConfirmPanel, HumanConfirmRequiredBy,
    HumanConfirmSafeAction, PaperPositionPanel, ProviderPanel, RiskPanel,
};
use super::dashboard_secret_redaction::redact_dashboard_state;
use super::dashboard_state::{DashboardSourceConfig, DashboardState, DashboardSystemMode};
use super::owner_panel::{OwnerPanel, OwnerReviewQueueSummary};

#[derive(Clone, Debug, Default)]
pub struct DashboardSnapshotBuilder;

impl DashboardSnapshotBuilder {
    pub fn build(&self, config: &DashboardSourceConfig) -> Result<DashboardState, String> {
        if !config.validate_local_paths().is_empty() {
            return Err("dashboard-snapshot config paths must be local".to_string());
        }

        let mut warnings = Vec::new();
        let mut blockers = Vec::new();
        let mut reason_codes = config.reason_codes.clone();
        reason_codes.extend([
            ReasonCode::DashboardStateBuilt,
            ReasonCode::DashboardSnapshotBuilt,
        ]);
        let mut generated_from_reports = Vec::new();

        let simplification_reports = load_values(
            &config.provider_simplification_report_paths,
            "provider simplification",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let kis_reports = load_values(
            &config.kis_activation_report_paths,
            "kis activation",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let evidence_reports = load_values(
            &config
                .official_evidence_scaleout_paths
                .iter()
                .chain(config.official_evidence_diversity_paths.iter())
                .chain(config.core_performance_scorecard_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
            "evidence",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let committee_reports = load_values(
            &config
                .committee_benchmark_paths
                .iter()
                .chain(config.committee_v1_paths.iter())
                .chain(config.committee_diagnostics_paths.iter())
                .cloned()
                .collect::<Vec<_>>(),
            "committee",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let risk_reports = load_values(
            &config.risk_reports,
            "risk",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let candidate_reports = load_values(
            &config.candidate_queue_paths,
            "candidate queue",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let paper_reports = load_values(
            &config.paper_position_paths,
            "paper position",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let human_reports = load_values(
            &config.human_confirm_paths,
            "human confirm",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let owner_input_reports = load_values(
            &config.owner_input_paths,
            "owner inputs",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let owner_thesis_reports = load_values(
            &config.owner_thesis_note_paths,
            "owner thesis",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let owner_review_queue_reports = load_values(
            &config.owner_review_queue_paths,
            "owner review queue",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let owner_impact_reports = load_values(
            &config.owner_impact_report_paths,
            "owner impact",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let audit_reports = load_values(
            &config.audit_ledger_paths,
            "audit timeline",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );
        let krx_reports = load_values(
            &config.krx_activation_report_paths,
            "krx activation",
            &mut warnings,
            &mut reason_codes,
            &mut generated_from_reports,
        );

        let provider_panel = build_provider_panel(
            &simplification_reports,
            &kis_reports,
            &krx_reports,
            &mut reason_codes,
        );
        let evidence_panel = build_panel_or_default::<EvidencePanel>(
            &evidence_reports,
            &["evidence_panel"],
            &mut reason_codes,
            ReasonCode::DashboardEvidencePanelBuilt,
        );
        let mut committee_panel = build_panel_or_default::<CommitteePanel>(
            &committee_reports,
            &["committee_panel"],
            &mut reason_codes,
            ReasonCode::DashboardCommitteePanelBuilt,
        );
        committee_panel
            .member_views
            .truncate(config.max_committee_rows);
        let chair_panel = build_panel_or_default::<ChairPanel>(
            &committee_reports,
            &["chair_panel"],
            &mut reason_codes,
            ReasonCode::DashboardChairPanelBuilt,
        );
        let risk_panel = build_panel_or_default::<RiskPanel>(
            &risk_reports,
            &["risk_panel"],
            &mut reason_codes,
            ReasonCode::DashboardRiskPanelBuilt,
        );
        let mut candidate_panel = build_panel_or_default::<CandidatePanel>(
            &candidate_reports,
            &["candidate_panel"],
            &mut reason_codes,
            ReasonCode::DashboardCandidatePanelBuilt,
        );
        candidate_panel.candidates.truncate(config.max_candidates);
        candidate_panel.stabilize();
        let owner_inputs = load_owner_inputs(&config.owner_input_paths);
        let owner_thesis_book = load_owner_thesis_book(&config.owner_thesis_note_paths);
        enrich_candidate_panel(&mut candidate_panel, &owner_inputs, &owner_thesis_book);
        let paper_position_panel = build_panel_or_default::<PaperPositionPanel>(
            &paper_reports,
            &["paper_position_panel"],
            &mut reason_codes,
            ReasonCode::DashboardPaperPanelBuilt,
        );
        let owner_review_queue = build_panel_optional::<OwnerReviewQueue>(
            &owner_review_queue_reports,
            &["owner_review_queue"],
        )
        .unwrap_or_else(|| {
            build_owner_review_queue(
                &format!("{}-owner-review", config.dashboard_id),
                &candidate_panel,
                &owner_inputs,
                &Default::default(),
            )
        });
        let owner_impact_report = build_panel_optional::<OwnerDecisionImpactReport>(
            &owner_impact_reports,
            &["owner_impact_report"],
        )
        .unwrap_or_else(|| {
            build_owner_decision_impact_report(
                &format!("{}-owner-impact", config.dashboard_id),
                &candidate_panel,
                &owner_inputs,
                &Default::default(),
            )
        });
        let mut human_confirm_panel = if let Some(panel) =
            build_panel_optional::<HumanConfirmPanel>(&human_reports, &["human_confirm_panel"])
        {
            let mut panel = panel;
            panel.stabilize();
            reason_codes.push(ReasonCode::DashboardHumanConfirmPanelBuilt);
            panel
        } else {
            derive_human_confirm_panel(&candidate_panel, &mut reason_codes)
        };
        enrich_human_confirm_panel(&mut human_confirm_panel, &owner_review_queue);
        let mut owner_panel = derive_owner_panel(
            &owner_inputs,
            &owner_thesis_book,
            &owner_review_queue,
            &owner_impact_report,
            &owner_input_reports,
            &owner_thesis_reports,
        );
        owner_panel.stabilize();
        reason_codes.push(ReasonCode::DashboardOwnerPanelBuilt);
        let mut bottleneck_panel = build_panel_or_default::<BottleneckPanel>(
            &evidence_reports,
            &["bottleneck_panel"],
            &mut reason_codes,
            ReasonCode::DashboardBottleneckPanelBuilt,
        );
        if bottleneck_panel.primary_bottleneck.is_empty() {
            bottleneck_panel.primary_bottleneck = if evidence_panel.current_bottleneck.is_empty() {
                "NeedMoreEvidence".to_string()
            } else {
                evidence_panel.current_bottleneck.clone()
            };
            bottleneck_panel.next_action = if evidence_panel.next_recommended_action.is_empty() {
                "Keep the dashboard in diagnostic/read-only mode and expand official evidence."
                    .to_string()
            } else {
                evidence_panel.next_recommended_action.clone()
            };
            bottleneck_panel.core_status = "ResearchOnly".to_string();
            bottleneck_panel.evidence_status = evidence_panel.sufficiency_status.clone();
            bottleneck_panel.risk_status = risk_panel
                .last_risk_decision
                .clone()
                .unwrap_or_else(|| "NoTrade".to_string());
            bottleneck_panel.committee_status = committee_panel.recommendation.clone();
            bottleneck_panel.operator_actions = provider_panel.operator_actions.clone();
        }
        bottleneck_panel.stabilize();

        let mut audit_timeline = build_audit_timeline(&audit_reports, &mut reason_codes);
        if audit_timeline.events.is_empty() {
            audit_timeline.events.push(DashboardEvent {
                event_id: "evt-derived-provider".to_string(),
                kind: DashboardEventKind::ProviderStatus,
                timestamp_ms: None,
                title: "Provider simplification loaded".to_string(),
                summary: format!(
                    "KIS primary for Korean/US equity, KRX reference fallback, yfinance research-only. status={}",
                    provider_panel
                        .active_primary_provider_by_market
                        .iter()
                        .map(|(market, provider)| format!("{market}:{provider}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                source_report: None,
                severity: DashboardEventSeverity::Info,
                reason_codes: vec![ReasonCode::DashboardProviderPanelBuilt],
            });
        }
        append_owner_events(
            &mut audit_timeline,
            &owner_inputs,
            &owner_review_queue,
            &owner_impact_report,
        );
        audit_timeline.events.truncate(config.max_events);
        audit_timeline.stabilize();

        if !provider_panel.kis_status.auth_ready {
            blockers.push("KIS auth not ready".to_string());
        }
        if provider_panel
            .kis_status
            .endpoint_policy_status
            .to_ascii_lowercase()
            .contains("block")
        {
            blockers.push("KIS endpoint policy blocked".to_string());
        }
        if !config.include_research_only && evidence_panel.source_breakdown.research_only > 0 {
            warnings.push(
                "Research-only evidence is displayed but not promoted to official readiness."
                    .to_string(),
            );
        }
        if paper_position_panel.open_positions.is_empty()
            && human_confirm_panel.pending_items.is_empty()
        {
            warnings.push(
                "Dashboard has no active paper positions or pending human confirmations."
                    .to_string(),
            );
        }

        let system_mode = derive_system_mode(
            &paper_position_panel,
            &candidate_panel,
            &risk_panel,
            config.include_diagnostics,
        );

        let mut state = DashboardState {
            dashboard_id: config.dashboard_id.clone(),
            generated_from_reports,
            system_mode,
            provider_panel,
            evidence_panel,
            committee_panel,
            chair_panel,
            risk_panel,
            candidate_panel,
            paper_position_panel,
            human_confirm_panel,
            owner_panel,
            bottleneck_panel,
            audit_timeline,
            warnings,
            blockers,
            reason_codes: stable_reason_codes(&reason_codes),
            fingerprint: String::new(),
        }
        .with_fingerprint();

        if config.redact_secrets {
            state = redact_dashboard_state(&state)?.0;
        }

        Ok(state.with_fingerprint())
    }

    pub fn build_and_write(
        &self,
        config: &DashboardSourceConfig,
    ) -> Result<DashboardState, String> {
        let state = self.build(config)?;
        state.write_to_dir(&config.artifact_dir())?;
        Ok(state)
    }
}

fn load_values(
    paths: &[String],
    label: &str,
    warnings: &mut Vec<String>,
    reason_codes: &mut Vec<ReasonCode>,
    generated_from_reports: &mut Vec<String>,
) -> Vec<Value> {
    let mut values = Vec::new();
    for path in stable_ordered_strings(paths) {
        match fs::read_to_string(&path) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => {
                    generated_from_reports.push(path);
                    values.push(value);
                }
                Err(err) => {
                    warnings.push(format!("failed to parse {label} report: {err}"));
                    reason_codes.push(ReasonCode::DataLoadFailed);
                }
            },
            Err(_) => {
                warnings.push(format!("missing {label} report"));
                reason_codes.push(ReasonCode::MissingFile);
                reason_codes.push(ReasonCode::DashboardReportMissing);
            }
        }
    }
    values
}

fn build_provider_panel(
    simplification_reports: &[Value],
    kis_reports: &[Value],
    krx_reports: &[Value],
    reason_codes: &mut Vec<ReasonCode>,
) -> ProviderPanel {
    let mut panel = ProviderPanel {
        active_primary_provider_by_market: Default::default(),
        kis_status: DashboardKisStatus {
            auth_ready: false,
            endpoint_policy_status: "Unknown".to_string(),
            domestic_market_data_ready: false,
            overseas_market_data_ready: false,
            realtime_ready: false,
            blocked_reasons: Vec::new(),
        },
        krx_status: DashboardKrxStatus {
            reference_enabled: true,
            fallback_enabled: true,
            blocked_reasons: Vec::new(),
        },
        alpha_vantage_status: DashboardNamedProviderStatus {
            provider_label: "AlphaVantage".to_string(),
            mode: ProviderPriorityMode::AlphaVantageFallback,
            enabled: true,
            notes: vec!["US equity fallback".to_string()],
            blocked_reasons: Vec::new(),
        },
        yfinance_status: DashboardNamedProviderStatus {
            provider_label: "yfinance".to_string(),
            mode: ProviderPriorityMode::YFinanceResearchOnly,
            enabled: true,
            notes: vec!["research-only supplemental data".to_string()],
            blocked_reasons: Vec::new(),
        },
        upbit_status: DashboardNamedProviderStatus {
            provider_label: "Upbit".to_string(),
            mode: ProviderPriorityMode::UpbitCryptoOnly,
            enabled: true,
            notes: vec!["optional crypto-only lane".to_string()],
            blocked_reasons: Vec::new(),
        },
        operator_actions: Vec::new(),
        reason_codes: vec![ReasonCode::DashboardProviderPanelBuilt],
    };

    if let Some(report) = simplification_reports.iter().find_map(|value| {
        serde_json::from_value::<ProviderSimplificationReport>(value.clone())
            .ok()
            .or_else(|| {
                extract_nested::<ProviderSimplificationReport>(
                    value,
                    &["provider_simplification_report"],
                )
            })
    }) {
        panel.active_primary_provider_by_market.insert(
            "KoreanEquity".to_string(),
            report.korean_equity_primary.provider_label.clone(),
        );
        panel.active_primary_provider_by_market.insert(
            "USEquity".to_string(),
            report.us_equity_primary.provider_label.clone(),
        );
        panel
            .operator_actions
            .extend(report.operator_actions.clone());
        panel.reason_codes.extend(report.reason_codes.clone());
        if matches!(
            report.final_status,
            ProviderSimplificationFinalStatus::KISPrimaryBlockedByAuth
                | ProviderSimplificationFinalStatus::NeedProviderAuth
        ) {
            panel
                .kis_status
                .blocked_reasons
                .push("Missing auth".to_string());
        }
        if matches!(
            report.final_status,
            ProviderSimplificationFinalStatus::KISPrimaryBlockedByEndpointPolicy
        ) {
            panel
                .kis_status
                .blocked_reasons
                .push("Endpoint policy blocked".to_string());
            panel.kis_status.endpoint_policy_status = "Blocked".to_string();
        }
        panel.krx_status.reference_enabled =
            report.fallback_providers.iter().any(|item| item == "KRX");
        panel.krx_status.fallback_enabled = panel.krx_status.reference_enabled;
        panel.alpha_vantage_status.enabled = report
            .fallback_providers
            .iter()
            .any(|item| item == "AlphaVantage");
        panel.yfinance_status.enabled = report
            .research_only_providers
            .iter()
            .any(|item| item.eq_ignore_ascii_case("yfinance"));
        panel.upbit_status.enabled = !report.disabled_providers.iter().any(|item| item == "Upbit");
    } else {
        panel
            .active_primary_provider_by_market
            .insert("KoreanEquity".to_string(), "KIS".to_string());
        panel
            .active_primary_provider_by_market
            .insert("USEquity".to_string(), "KIS".to_string());
    }

    for value in kis_reports {
        panel.kis_status.auth_ready |= bool_field(value, &["auth_ready"]).unwrap_or(false);
        panel.kis_status.domestic_market_data_ready |= bool_field(
            value,
            &[
                "domestic_market_data_ready",
                "safe_to_collect_rest_market_data",
            ],
        )
        .unwrap_or(false);
        panel.kis_status.overseas_market_data_ready |= bool_field(
            value,
            &[
                "overseas_market_data_ready",
                "safe_to_collect_rest_market_data",
            ],
        )
        .unwrap_or(false);
        panel.kis_status.realtime_ready |= bool_field(value, &["realtime_ready"]).unwrap_or(false);
        if let Some(status) = string_field(value, &["endpoint_policy_status"]) {
            panel.kis_status.endpoint_policy_status = status;
        }
        panel
            .kis_status
            .blocked_reasons
            .extend(array_string_field(value, &["blocked_reasons"]));
    }

    if panel.kis_status.endpoint_policy_status.is_empty() {
        panel.kis_status.endpoint_policy_status = "Unknown".to_string();
    }
    if panel.kis_status.endpoint_policy_status == "Unknown" {
        panel.kis_status.endpoint_policy_status = "MarketDataOnly".to_string();
    }

    if krx_reports.is_empty() {
        panel.krx_status.reference_enabled = true;
        panel.krx_status.fallback_enabled = true;
    } else {
        for value in krx_reports {
            panel.krx_status.reference_enabled |=
                bool_field(value, &["reference_enabled"]).unwrap_or(true);
            panel.krx_status.fallback_enabled |=
                bool_field(value, &["fallback_enabled"]).unwrap_or(true);
            panel
                .krx_status
                .blocked_reasons
                .extend(array_string_field(value, &["blocked_reasons"]));
        }
    }

    panel.stabilize();
    reason_codes.push(ReasonCode::DashboardProviderPanelBuilt);
    panel
}

fn build_panel_or_default<T: DeserializeOwned + Default>(
    values: &[Value],
    nested_keys: &[&str],
    reason_codes: &mut Vec<ReasonCode>,
    reason_code: ReasonCode,
) -> T {
    let panel = build_panel_optional(values, nested_keys).unwrap_or_default();
    reason_codes.push(reason_code);
    panel
}

fn build_panel_optional<T: DeserializeOwned>(values: &[Value], nested_keys: &[&str]) -> Option<T> {
    values.iter().find_map(|value| {
        extract_nested::<T>(value, nested_keys)
            .or_else(|| serde_json::from_value::<T>(value.clone()).ok())
    })
}

fn extract_nested<T: DeserializeOwned>(value: &Value, keys: &[&str]) -> Option<T> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .cloned()
            .and_then(|item| serde_json::from_value(item).ok())
    })
}

fn derive_human_confirm_panel(
    candidate_panel: &CandidatePanel,
    reason_codes: &mut Vec<ReasonCode>,
) -> HumanConfirmPanel {
    let mut panel = HumanConfirmPanel::default();
    for candidate in &candidate_panel.candidates {
        if matches!(candidate.status, CandidateStatus::HumanConfirmRequired) {
            panel.pending_items.push(HumanConfirmItem {
                confirm_id: format!("confirm-{}", candidate.candidate_id),
                candidate_id: candidate.candidate_id.clone(),
                reason: "Human confirmation required before paper approval.".to_string(),
                required_by: HumanConfirmRequiredBy::Chair,
                paper_confirm_allowed: false,
                paper_confirm_explanation:
                    "PaperConfirm is unavailable until owner review queue policy is derived."
                        .to_string(),
                safe_actions: vec![
                    HumanConfirmSafeAction::ViewOnly,
                    HumanConfirmSafeAction::MarkReviewed,
                    HumanConfirmSafeAction::Defer,
                ],
                allowed_owner_actions: Vec::new(),
                forbidden_actions: vec![
                    HumanConfirmForbiddenAction::ExecuteOrder,
                    HumanConfirmForbiddenAction::PlaceTrade,
                    HumanConfirmForbiddenAction::ModifyAccount,
                ],
                forbidden_owner_actions: Vec::new(),
                reason_codes: candidate.reason_codes.clone(),
            });
        }
    }
    panel
        .reason_codes
        .push(ReasonCode::DashboardHumanConfirmPanelBuilt);
    panel.stabilize();
    reason_codes.push(ReasonCode::DashboardHumanConfirmPanelBuilt);
    panel
}

fn build_audit_timeline(
    values: &[Value],
    reason_codes: &mut Vec<ReasonCode>,
) -> AuditTimelinePanel {
    if let Some(mut panel) = build_panel_optional::<AuditTimelinePanel>(values, &["audit_timeline"])
    {
        panel.stabilize();
        reason_codes.push(ReasonCode::DashboardAuditTimelineBuilt);
        return panel;
    }
    let events = values
        .iter()
        .filter_map(|value| {
            value
                .get("events")
                .cloned()
                .and_then(|events| serde_json::from_value::<Vec<DashboardEvent>>(events).ok())
        })
        .flatten()
        .collect::<Vec<_>>();
    let mut panel = AuditTimelinePanel {
        events,
        warnings: 0,
        errors: 0,
        critical_count: 0,
        fingerprint: String::new(),
        reason_codes: vec![ReasonCode::DashboardAuditTimelineBuilt],
    };
    panel.stabilize();
    reason_codes.push(ReasonCode::DashboardAuditTimelineBuilt);
    panel
}

fn load_owner_inputs(paths: &[String]) -> Vec<OwnerInput> {
    let mut inputs = paths
        .iter()
        .filter_map(|path| load_owner_inputs_from_path(std::path::Path::new(path)).ok())
        .flatten()
        .collect::<Vec<_>>();
    inputs.sort_by(|left, right| left.owner_input_id.cmp(&right.owner_input_id));
    inputs.dedup_by(|left, right| left.owner_input_id == right.owner_input_id);
    for input in &mut inputs {
        input.stabilize();
    }
    inputs
}

fn load_owner_thesis_book(paths: &[String]) -> OwnerThesisBook {
    let notes = paths
        .iter()
        .filter_map(|path| load_owner_thesis_notes_from_path(std::path::Path::new(path)).ok())
        .flatten()
        .collect::<Vec<_>>();
    OwnerThesisBook::from_notes(&notes, None)
}

fn enrich_candidate_panel(
    candidate_panel: &mut CandidatePanel,
    owner_inputs: &[OwnerInput],
    owner_thesis_book: &OwnerThesisBook,
) {
    for candidate in &mut candidate_panel.candidates {
        let relevant_inputs = owner_inputs
            .iter()
            .filter(|input| {
                input.target_id.as_deref() == Some(candidate.candidate_id.as_str())
                    || input.symbol.as_deref() == Some(candidate.symbol.as_str())
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut feedback_history = candidate.owner_feedback_history.clone();
        for input in &relevant_inputs {
            let validation = validate_owner_input(input);
            if let Some(feedback) = build_owner_candidate_feedback(input, &validation) {
                feedback_history.push(feedback);
            }
        }
        feedback_history.sort_by(|left, right| left.feedback_id.cmp(&right.feedback_id));
        feedback_history.dedup_by(|left, right| left.feedback_id == right.feedback_id);
        candidate.owner_feedback_history = feedback_history;
        candidate.owner_hold_active = relevant_inputs.iter().any(|input| {
            matches!(
                input.input_kind,
                crate::owner::OwnerInputKind::CandidateHold
            )
        });
        candidate.owner_dismissed = relevant_inputs.iter().any(|input| {
            matches!(
                input.input_kind,
                crate::owner::OwnerInputKind::CandidateDismiss
            )
        });
        candidate.owner_reanalysis_requested = relevant_inputs.iter().any(|input| {
            matches!(
                input.input_kind,
                crate::owner::OwnerInputKind::CandidateReanalysisRequest
            )
        });
        candidate.owner_paper_confirmed = relevant_inputs.iter().any(|input| {
            matches!(input.input_kind, crate::owner::OwnerInputKind::PaperConfirm)
                && validate_owner_input(input).allowed
        });
        let mut notes = candidate.linked_thesis_notes.clone();
        if let Some(linked) = owner_thesis_book.notes_by_symbol.get(&candidate.symbol) {
            notes.extend(linked.clone());
        }
        notes.sort_by(|left, right| left.thesis_id.cmp(&right.thesis_id));
        notes.dedup_by(|left, right| left.thesis_id == right.thesis_id);
        candidate.linked_thesis_notes = notes;
    }
    candidate_panel.stabilize();
}

fn enrich_human_confirm_panel(
    human_confirm_panel: &mut HumanConfirmPanel,
    owner_review_queue: &OwnerReviewQueue,
) {
    for items in [
        &mut human_confirm_panel.pending_items,
        &mut human_confirm_panel.reviewed_items,
        &mut human_confirm_panel.deferred_items,
    ] {
        for item in items.iter_mut() {
            if let Some(review_item) = owner_review_queue
                .pending_items
                .iter()
                .chain(owner_review_queue.reviewed_items.iter())
                .chain(owner_review_queue.deferred_items.iter())
                .chain(owner_review_queue.dismissed_items.iter())
                .chain(owner_review_queue.paper_confirmed_items.iter())
                .chain(owner_review_queue.blocked_items.iter())
                .chain(owner_review_queue.expired_items.iter())
                .find(|review| review.candidate_id.as_deref() == Some(item.candidate_id.as_str()))
            {
                item.allowed_owner_actions = review_item.allowed_owner_actions.clone();
                item.forbidden_owner_actions = review_item.forbidden_owner_actions.clone();
                item.paper_confirm_allowed = review_item
                    .allowed_owner_actions
                    .contains(&AllowedOwnerAction::PaperConfirm);
                item.paper_confirm_explanation = if item.paper_confirm_allowed {
                    "PaperConfirm allowed for a paper-only path because risk permits human confirmation."
                        .to_string()
                } else if review_item
                    .reason_codes
                    .contains(&ReasonCode::OwnerResearchOnlyConfirmBlocked)
                {
                    "PaperConfirm forbidden because the candidate remains research-only."
                        .to_string()
                } else if review_item
                    .reason_codes
                    .contains(&ReasonCode::OwnerDiagnosticOnlyConfirmBlocked)
                {
                    "PaperConfirm forbidden because the candidate remains diagnostic-only."
                        .to_string()
                } else if review_item.reason_codes.contains(&ReasonCode::RiskDenied) {
                    "PaperConfirm forbidden because Risk Governor denied the candidate.".to_string()
                } else if review_item
                    .reason_codes
                    .contains(&ReasonCode::NoTradeDefault)
                {
                    "PaperConfirm forbidden because the candidate remains NoTrade.".to_string()
                } else {
                    "PaperConfirm forbidden because the paper-only review protocol does not allow it for this item."
                        .to_string()
                };
                item.reason_codes.extend(review_item.reason_codes.clone());
            }
        }
    }
    human_confirm_panel.stabilize();
}

fn derive_owner_panel(
    owner_inputs: &[OwnerInput],
    owner_thesis_book: &OwnerThesisBook,
    owner_review_queue: &OwnerReviewQueue,
    owner_impact_report: &OwnerDecisionImpactReport,
    owner_input_reports: &[Value],
    owner_thesis_reports: &[Value],
) -> OwnerPanel {
    let blocked_input_ids = owner_impact_report
        .records
        .iter()
        .filter(|record| {
            matches!(
                record.impact_kind,
                OwnerDecisionImpactKind::BlockedByPolicy
                    | OwnerDecisionImpactKind::BlockedByRiskGovernor
            )
        })
        .map(|record| record.owner_input_id.clone())
        .collect::<Vec<_>>();
    let mut panel = OwnerPanel {
        review_queue_summary: OwnerReviewQueueSummary {
            pending_review_items: owner_review_queue.pending_items.len(),
            reviewed_items: owner_review_queue.reviewed_items.len(),
            deferred_items: owner_review_queue.deferred_items.len(),
            dismissed_items: owner_review_queue.dismissed_items.len(),
            paper_confirmed_items: owner_review_queue.paper_confirmed_items.len(),
            blocked_items: owner_review_queue.blocked_items.len(),
            expired_items: owner_review_queue.expired_items.len(),
        },
        pending_review_items: owner_review_queue.pending_items.clone(),
        recent_owner_inputs: owner_inputs.to_vec(),
        active_thesis_notes: owner_thesis_book.active_notes.clone(),
        blocked_owner_inputs: owner_inputs
            .iter()
            .filter(|input| blocked_input_ids.contains(&input.owner_input_id))
            .cloned()
            .collect(),
        paper_confirmed_items: owner_review_queue.paper_confirmed_items.clone(),
        reanalysis_requests: owner_inputs
            .iter()
            .filter(|input| {
                matches!(
                    input.input_kind,
                    crate::owner::OwnerInputKind::CandidateReanalysisRequest
                )
            })
            .cloned()
            .collect(),
        owner_policy_warnings: Vec::new(),
        reason_codes: vec![ReasonCode::DashboardOwnerPanelBuilt],
    };
    if owner_input_reports.is_empty() {
        panel
            .owner_policy_warnings
            .push("No owner input reports were provided.".to_string());
    }
    if owner_thesis_reports.is_empty() {
        panel
            .owner_policy_warnings
            .push("No owner thesis notes were provided.".to_string());
    }
    if !panel.blocked_owner_inputs.is_empty() {
        panel
            .owner_policy_warnings
            .push("Some owner inputs were blocked by policy or risk.".to_string());
    }
    panel.stabilize();
    panel
}

fn append_owner_events(
    audit_timeline: &mut AuditTimelinePanel,
    owner_inputs: &[OwnerInput],
    owner_review_queue: &OwnerReviewQueue,
    owner_impact_report: &OwnerDecisionImpactReport,
) {
    for input in owner_inputs {
        let validation = validate_owner_input(input);
        audit_timeline.events.push(DashboardEvent {
            event_id: format!("owner-input-submitted-{}", input.owner_input_id),
            kind: DashboardEventKind::OwnerInputSubmitted,
            timestamp_ms: input.timestamp_ms,
            title: "Owner input submitted".to_string(),
            summary: format!(
                "{:?} submitted for target {}",
                input.input_kind,
                input.target_id.clone().unwrap_or_default()
            ),
            source_report: None,
            severity: DashboardEventSeverity::Info,
            reason_codes: validation.reason_codes.clone(),
        });
    }
    for record in &owner_impact_report.records {
        let (kind, title, severity) = match record.impact_kind {
            OwnerDecisionImpactKind::BlockedByPolicy
            | OwnerDecisionImpactKind::BlockedByRiskGovernor => (
                DashboardEventKind::OwnerInputBlocked,
                "Owner input blocked",
                DashboardEventSeverity::Warning,
            ),
            OwnerDecisionImpactKind::CandidateHeld => (
                DashboardEventKind::CandidateHeld,
                "Candidate held by owner",
                DashboardEventSeverity::Info,
            ),
            OwnerDecisionImpactKind::CandidateDismissed => (
                DashboardEventKind::CandidateDismissed,
                "Candidate dismissed by owner",
                DashboardEventSeverity::Info,
            ),
            OwnerDecisionImpactKind::PaperConfirmed => (
                DashboardEventKind::PaperConfirmed,
                "Candidate paper-confirmed",
                DashboardEventSeverity::Info,
            ),
            OwnerDecisionImpactKind::ReanalysisRequested => (
                DashboardEventKind::ReanalysisRequested,
                "Owner requested reanalysis",
                DashboardEventSeverity::Info,
            ),
            _ => (
                DashboardEventKind::OwnerInputApplied,
                "Owner input applied",
                DashboardEventSeverity::Info,
            ),
        };
        audit_timeline.events.push(DashboardEvent {
            event_id: format!("owner-impact-{}", record.owner_input_id),
            kind,
            timestamp_ms: None,
            title: title.to_string(),
            summary: format!(
                "impact={:?} target={} after={}",
                record.impact_kind,
                record.target_id.clone().unwrap_or_default(),
                record.after_status.clone().unwrap_or_default()
            ),
            source_report: None,
            severity,
            reason_codes: record.reason_codes.clone(),
        });
    }
    for item in &owner_review_queue.paper_confirmed_items {
        audit_timeline.events.push(DashboardEvent {
            event_id: format!("owner-human-confirm-{}", item.review_id),
            kind: DashboardEventKind::HumanConfirmTransition,
            timestamp_ms: None,
            title: "Human confirm transitioned to paper-confirmed".to_string(),
            summary: format!(
                "candidate={} paper-confirmed through audited owner review",
                item.candidate_id.clone().unwrap_or_default()
            ),
            source_report: None,
            severity: DashboardEventSeverity::Info,
            reason_codes: item.reason_codes.clone(),
        });
    }
}

fn derive_system_mode(
    paper_positions: &PaperPositionPanel,
    candidates: &CandidatePanel,
    risk_panel: &RiskPanel,
    include_diagnostics: bool,
) -> DashboardSystemMode {
    if !paper_positions.open_positions.is_empty()
        || candidates.candidates.iter().any(|candidate| {
            matches!(
                candidate.status,
                CandidateStatus::PaperApproved | CandidateStatus::PaperPositionOpen
            )
        })
    {
        DashboardSystemMode::Paper
    } else if include_diagnostics && risk_panel.default_deny_active {
        DashboardSystemMode::DiagnosticsOnly
    } else {
        DashboardSystemMode::Research
    }
}

fn bool_field(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(key).and_then(|item| item.as_bool()))
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(key)
            .and_then(|item| item.as_str())
            .map(|item| item.to_string())
    })
}

fn array_string_field(value: &Value, keys: &[&str]) -> Vec<String> {
    keys.iter()
        .find_map(|key| {
            value.get(key).and_then(|item| {
                item.as_array().map(|items| {
                    items
                        .iter()
                        .filter_map(|entry| entry.as_str().map(|item| item.to_string()))
                        .collect::<Vec<_>>()
                })
            })
        })
        .unwrap_or_default()
}
