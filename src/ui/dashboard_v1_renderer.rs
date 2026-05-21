use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::{ReasonCode, stable_reason_codes};

use super::control_tower_v1::{ControlTowerV1Config, ControlTowerV1State};
use super::owner_action_drafts::{OwnerActionDraftBundle, generate_owner_action_draft_bundle};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardV1RenderStatus {
    #[default]
    Rendered,
    RenderedWithWarnings,
    SecretRedactionFailed,
    UnsafeControlDetected,
    MissingState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardV1RenderReport {
    pub control_tower_id: String,
    #[serde(default)]
    pub html_path: Option<String>,
    #[serde(default)]
    pub json_path: Option<String>,
    #[serde(default)]
    pub text_path: Option<String>,
    #[serde(default)]
    pub owner_action_draft_dir: Option<String>,
    pub panel_count: usize,
    pub event_count: usize,
    pub candidate_count: usize,
    pub owner_pending_count: usize,
    pub secret_redaction_passed: bool,
    pub unsafe_control_detected: bool,
    pub render_status: DashboardV1RenderStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl DashboardV1RenderReport {
    pub fn to_text(&self) -> String {
        [
            "read_only_warning=dashboard v1 rendering is local-only static output".to_string(),
            format!("control_tower_id={}", self.control_tower_id),
            format!("html_path={}", self.html_path.clone().unwrap_or_default()),
            format!("json_path={}", self.json_path.clone().unwrap_or_default()),
            format!("text_path={}", self.text_path.clone().unwrap_or_default()),
            format!(
                "owner_action_draft_dir={}",
                self.owner_action_draft_dir.clone().unwrap_or_default()
            ),
            format!("panel_count={}", self.panel_count),
            format!("event_count={}", self.event_count),
            format!("candidate_count={}", self.candidate_count),
            format!("owner_pending_count={}", self.owner_pending_count),
            format!("secret_redaction_passed={}", self.secret_redaction_passed),
            format!("unsafe_control_detected={}", self.unsafe_control_detected),
            format!("render_status={:?}", self.render_status),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, Default)]
pub struct DashboardV1Renderer;

impl DashboardV1Renderer {
    pub fn render(
        &self,
        state: &ControlTowerV1State,
        config: &ControlTowerV1Config,
    ) -> Result<DashboardV1RenderReport, String> {
        let artifact_dir = config.artifact_dir();
        fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;

        let redacted_state = if config.redact_secrets {
            redact_state(state)?
        } else {
            state.clone()
        };
        let json = redacted_state.to_json_string()?;
        let text = render_text(&redacted_state);
        let mut html = render_html(&redacted_state, None);

        let secret_redaction_passed = !contains_secret_like_material(&json)
            && !contains_secret_like_material(&text)
            && !contains_secret_like_material(&html);
        let unsafe_control_detected = contains_unsafe_controls(&html);

        let mut html_path = None;
        let mut json_path = None;
        let mut text_path = None;

        if config.render_json {
            let path = artifact_dir.join("dashboard_state_v1.json");
            fs::write(&path, json).map_err(|err| err.to_string())?;
            json_path = Some(path.display().to_string());
        }
        if config.render_text {
            let path = artifact_dir.join("dashboard_v1.txt");
            fs::write(&path, text).map_err(|err| err.to_string())?;
            text_path = Some(path.display().to_string());
        }
        if config.render_html {
            let path = artifact_dir.join("dashboard_v1.html");
            html = html.replace(
                "__DRAFT_LINKS__",
                &artifact_dir
                    .join("owner_action_drafts")
                    .display()
                    .to_string(),
            );
            fs::write(&path, html).map_err(|err| err.to_string())?;
            html_path = Some(path.display().to_string());
        }

        let next_actions_path = artifact_dir.join("dashboard_next_actions.txt");
        fs::write(&next_actions_path, render_next_actions(&redacted_state))
            .map_err(|err| err.to_string())?;

        let owner_action_draft_bundle = if config.generate_owner_action_drafts {
            Some(generate_owner_action_draft_bundle(&redacted_state, config)?)
        } else {
            None
        };

        let render_status = if !secret_redaction_passed {
            DashboardV1RenderStatus::SecretRedactionFailed
        } else if unsafe_control_detected {
            DashboardV1RenderStatus::UnsafeControlDetected
        } else if !redacted_state.warnings.is_empty() || !redacted_state.blockers.is_empty() {
            DashboardV1RenderStatus::RenderedWithWarnings
        } else {
            DashboardV1RenderStatus::Rendered
        };

        Ok(DashboardV1RenderReport {
            control_tower_id: config.control_tower_id.clone(),
            html_path,
            json_path,
            text_path,
            owner_action_draft_dir: owner_action_draft_bundle
                .as_ref()
                .map(|bundle| bundle.draft_output_dir.clone()),
            panel_count: 17,
            event_count: redacted_state.audit_timeline.events.len(),
            candidate_count: redacted_state.candidate_panel.candidates.len(),
            owner_pending_count: redacted_state.owner_panel.pending_review_items.len(),
            secret_redaction_passed,
            unsafe_control_detected,
            render_status,
            reason_codes: stable_reason_codes(&[ReasonCode::DashboardRendered]),
        })
    }
}

fn render_text(state: &ControlTowerV1State) -> String {
    [
        state.to_text(),
        format!(
            "evidence_official_rows_before={}",
            state
                .evidence_panel
                .official_rows_before
                .unwrap_or_default()
        ),
        format!(
            "evidence_official_rows_after={}",
            state
                .evidence_panel
                .official_rows_after
                .unwrap_or(state.evidence_panel.official_rows)
        ),
        format!(
            "evidence_outcome_links_before={}",
            state
                .evidence_panel
                .outcome_links_before
                .unwrap_or_default()
        ),
        format!(
            "evidence_outcome_links_after={}",
            state
                .evidence_panel
                .outcome_links_after
                .unwrap_or(state.evidence_panel.outcome_links)
        ),
        format!(
            "operational_loop_status={}",
            state.operational_loop_panel.loop_status
        ),
        format!(
            "operational_loop_id={}",
            state
                .operational_loop_panel
                .last_loop_run_id
                .clone()
                .unwrap_or_default()
        ),
        format!(
            "candidate_lifecycle_views={}",
            state.candidate_lifecycle_panel.candidate_views.len()
        ),
        format!(
            "trinity_status_active={}",
            state.trinity_status_panel.active_count
        ),
        format!(
            "paper_lifecycle_closed={}",
            state.paper_lifecycle_panel.closed_positions
        ),
        format!("health_status={:?}", state.health_summary.health_status),
        format!(
            "primary_next_action_command={}",
            state
                .next_action_panel
                .primary_next_action
                .command_suggestion
                .clone()
                .unwrap_or_default()
        ),
        format!(
            "expected_output_artifact={}",
            state
                .next_action_panel
                .primary_next_action
                .expected_output_artifact
                .clone()
                .unwrap_or_default()
        ),
    ]
    .join("\n")
}

fn render_next_actions(state: &ControlTowerV1State) -> String {
    let mut lines = vec![
        "local_only_warning=run only local research and paper-only commands from this file"
            .to_string(),
        format!(
            "primary={:?}",
            state.next_action_panel.primary_next_action.action_kind
        ),
        format!(
            "command={}",
            state
                .next_action_panel
                .primary_next_action
                .command_suggestion
                .clone()
                .unwrap_or_default()
        ),
        format!(
            "expected_output={}",
            state
                .next_action_panel
                .primary_next_action
                .expected_output_artifact
                .clone()
                .unwrap_or_default()
        ),
    ];
    lines.extend(
        state
            .next_action_panel
            .recommended_actions
            .iter()
            .map(|action| {
                format!(
                    "recommended={:?};command={};expected={}",
                    action.action_kind,
                    action.command_suggestion.clone().unwrap_or_default(),
                    action.expected_output_artifact.clone().unwrap_or_default()
                )
            }),
    );
    lines.join("\n")
}

fn render_html(state: &ControlTowerV1State, _drafts: Option<&OwnerActionDraftBundle>) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>body{{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,sans-serif;margin:24px;background:#0f172a;color:#e2e8f0}}section{{border:1px solid #334155;border-radius:12px;padding:16px;margin:0 0 16px 0;background:#111827}}h1,h2{{margin:0 0 12px 0}}ul{{padding-left:18px}}code,pre{{background:#020617;color:#cbd5e1;padding:2px 4px;border-radius:4px;white-space:pre-wrap}}.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(320px,1fr));gap:16px}}a{{color:#93c5fd}}</style>\n</head>\n<body>\n<h1>{}</h1>\n<p>Local-only deterministic read-only dashboard. No broker, order, account, balance, holdings, position, or execution controls. Owner action drafts are files only and must be applied with owner CLI commands outside the browser.</p>\n<div class=\"grid\">\n<section><h2>KIS monitor</h2><p>auth_ready={}</p><p>base_url_ready={}</p><p>endpoint_policy={}</p><p>collection_plan={}</p><p>official_rows={}</p><p>outcome_links={}</p><p>latest_depth_run={}</p><p>latest_depth_status={}</p><p>latest_next_command=<code>{}</code></p><pre>{}</pre></section>\n<section><h2>Committee</h2><p>active_personas={}</p><p>recommendation={}</p><p>disagreement={:?}</p><p>groupthink={:?}</p></section>\n<section><h2>Operational loop</h2><p>status={}</p><p>loop_id={}</p><p>generated_candidates={}</p><p>risk_blocked={}</p><p>paper_open={}</p><p>next_action={}</p></section>\n<section><h2>Trinity status</h2><p>active={}</p><p>idle={}</p><p>blocked={}</p></section>\n<section><h2>Candidate lifecycle</h2><p>views={}</p><pre>{}</pre></section>\n<section><h2>Paper lifecycle</h2><p>open={}</p><p>closed={}</p><p>target_hits={}</p><p>stop_hits={}</p></section>\n<section><h2>Evidence depth</h2><p>official_rows_before={}</p><p>official_rows_after={}</p><p>outcome_links_before={}</p><p>outcome_links_after={}</p><p>counterfactuals_before={}</p><p>counterfactuals_after={}</p></section>\n<section><h2>Owner</h2><p>pending_reviews={}</p><p>blocked_inputs={}</p><p>paper_confirmed={}</p></section>\n<section><h2>Risk</h2><p>mode={}</p><p>denied_count={}</p><p>no_trade_count={}</p></section>\n<section><h2>Candidates</h2><p>count={}</p><pre>{}</pre></section>\n<section><h2>Paper positions</h2><p>open={}</p><p>closed={}</p></section>\n<section><h2>Human confirm</h2><p>pending={}</p><pre>{}</pre></section>\n<section><h2>Next actions</h2><p>primary={:?}</p><p><code>{}</code></p><p>expected_output={}</p></section>\n<section><h2>Health</h2><p>status={:?}</p><p>bottleneck={}</p><p>ui_health={}</p></section>\n<section><h2>Drafts</h2><p>Draft bundle path: <code>__DRAFT_LINKS__</code></p><p>Use <code>cargo run --quiet --bin soma_experiment -- dashboard-action-drafts --config examples/soma_dashboard_action_drafts.toml</code> to refresh local draft files.</p></section>\n</div>\n<section><h2>Audit timeline</h2><ul>{}</ul></section>\n</body>\n</html>",
        escape_html(&state.control_tower_id),
        escape_html(&state.control_tower_id),
        state.kis_monitor_panel.auth_ready,
        state.kis_monitor_panel.base_url_ready,
        escape_html(&state.kis_monitor_panel.endpoint_policy_status),
        escape_html(&state.kis_monitor_panel.collection_plan_status),
        state.kis_monitor_panel.official_row_count,
        state.kis_monitor_panel.outcome_links,
        escape_html(
            &state
                .kis_monitor_panel
                .latest_depth_run_id
                .clone()
                .unwrap_or_default()
        ),
        escape_html(
            &state
                .kis_monitor_panel
                .latest_depth_status
                .clone()
                .unwrap_or_default()
        ),
        escape_html(
            &state
                .kis_monitor_panel
                .latest_next_command
                .clone()
                .unwrap_or_default()
        ),
        escape_html(&state.kis_monitor_panel.next_kis_actions.join("\n")),
        state.committee_panel.active_personas,
        escape_html(&state.committee_panel.recommendation),
        state.committee_panel.disagreement_score,
        state.committee_panel.groupthink_risk,
        escape_html(&state.operational_loop_panel.loop_status),
        escape_html(
            &state
                .operational_loop_panel
                .last_loop_run_id
                .clone()
                .unwrap_or_default()
        ),
        state.operational_loop_panel.generated_candidates,
        state.operational_loop_panel.risk_blocked,
        state.operational_loop_panel.paper_open,
        escape_html(&state.operational_loop_panel.next_action),
        state.trinity_status_panel.active_count,
        state.trinity_status_panel.idle_count,
        state.trinity_status_panel.blocked_count,
        state.candidate_lifecycle_panel.candidate_views.len(),
        escape_html(
            &state
                .candidate_lifecycle_panel
                .candidate_views
                .iter()
                .map(|view| format!(
                    "{} {} {}",
                    view.candidate_id, view.symbol, view.lifecycle_status
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        state.paper_lifecycle_panel.open_positions,
        state.paper_lifecycle_panel.closed_positions,
        state.paper_lifecycle_panel.target_hit_count,
        state.paper_lifecycle_panel.stop_hit_count,
        state
            .evidence_panel
            .official_rows_before
            .unwrap_or_default(),
        state
            .evidence_panel
            .official_rows_after
            .unwrap_or(state.evidence_panel.official_rows),
        state
            .evidence_panel
            .outcome_links_before
            .unwrap_or_default(),
        state
            .evidence_panel
            .outcome_links_after
            .unwrap_or(state.evidence_panel.outcome_links),
        state
            .evidence_panel
            .counterfactuals_before
            .unwrap_or_default(),
        state.evidence_panel.counterfactuals_after.unwrap_or(
            state.evidence_panel.no_trade_counterfactuals
                + state.evidence_panel.risk_denied_counterfactuals
        ),
        state.owner_panel.pending_review_items.len(),
        state.owner_panel.blocked_owner_inputs.len(),
        state.owner_panel.paper_confirmed_items.len(),
        escape_html(&state.risk_panel.risk_governor_mode),
        state.risk_panel.denied_count,
        state.risk_panel.no_trade_count,
        state.candidate_panel.candidates.len(),
        escape_html(
            &state
                .candidate_panel
                .candidates
                .iter()
                .map(|candidate| format!(
                    "{} {:?} {} hold={} dismiss={} reanalysis={} paper_confirmed={}",
                    candidate.candidate_id,
                    candidate.status,
                    candidate.symbol,
                    candidate.owner_hold_active,
                    candidate.owner_dismissed,
                    candidate.owner_reanalysis_requested,
                    candidate.owner_paper_confirmed
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        state.paper_position_panel.open_positions.len(),
        state.paper_position_panel.closed_positions.len(),
        state.human_confirm_panel.pending_items.len(),
        escape_html(
            &state
                .human_confirm_panel
                .pending_items
                .iter()
                .map(|item| format!(
                    "{} paper_confirm_allowed={} why={}",
                    item.confirm_id, item.paper_confirm_allowed, item.paper_confirm_explanation
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        state.next_action_panel.primary_next_action.action_kind,
        escape_html(
            &state
                .next_action_panel
                .primary_next_action
                .command_suggestion
                .clone()
                .unwrap_or_default()
        ),
        escape_html(
            &state
                .next_action_panel
                .primary_next_action
                .expected_output_artifact
                .clone()
                .unwrap_or_default()
        ),
        state.health_summary.health_status,
        escape_html(&state.health_summary.current_primary_bottleneck),
        escape_html(&state.health_summary.ui_health),
        state
            .audit_timeline
            .events
            .iter()
            .map(|event| format!(
                "<li><strong>{}</strong> [{}] — {}</li>",
                escape_html(&event.title),
                escape_html(&format!("{:?}", event.severity)),
                escape_html(&event.summary)
            ))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn redact_state(state: &ControlTowerV1State) -> Result<ControlTowerV1State, String> {
    let mut value = serde_json::to_value(state).map_err(|err| err.to_string())?;
    redact_value(&mut value);
    let redacted =
        serde_json::from_value::<ControlTowerV1State>(value).map_err(|err| err.to_string())?;
    Ok(redacted.with_fingerprint())
}

fn redact_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let keys = map.keys().cloned().collect::<Vec<_>>();
            for key in keys {
                if let Some(child) = map.get_mut(&key) {
                    if is_sensitive_key(&key) && child.is_string() {
                        *child = Value::String("[REDACTED]".to_string());
                    } else {
                        redact_value(child);
                    }
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_value(item);
            }
        }
        Value::String(text) => {
            if contains_secret_like_material(text) {
                *text = "[REDACTED]".to_string();
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    matches!(
        key.as_str(),
        "kis_app_key"
            | "app_key"
            | "kis_app_secret"
            | "app_secret"
            | "kis_ws_approval_key"
            | "krx_api_key"
            | "base_url_preview_redacted"
            | "token"
            | "password"
    )
}

fn contains_secret_like_material(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    [
        "kis_app_key",
        "kis_app_secret",
        "kis_ws_approval_key",
        "krx_api_key",
        "token=",
        "password=",
        "secret=",
        "app_key=",
        "app_secret=",
        "kis_base_url=",
        "hunter2",
        "secret-value",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn contains_unsafe_controls(html: &str) -> bool {
    let normalized = html.to_ascii_lowercase();
    [
        "<form",
        "<button",
        "<input",
        "method=\"post\"",
        "fetch(",
        "xmlhttprequest",
        "onclick=",
        "executeorder",
        "placetrade",
        "account-balance",
        "command execution",
        "cdn.jsdelivr",
        "unpkg.com",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
