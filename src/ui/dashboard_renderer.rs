use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

use super::dashboard_secret_redaction::redact_dashboard_state;
use super::dashboard_snapshot::DashboardSnapshotBuilder;
use super::dashboard_state::{DashboardSourceConfig, DashboardState};

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardRenderConfig {
    pub dashboard_id: String,
    #[serde(default)]
    pub dashboard_state_path: Option<String>,
    #[serde(default)]
    pub source_config_path: Option<String>,
    pub output_root: String,
    #[serde(default = "default_true")]
    pub render_html: bool,
    #[serde(default = "default_true")]
    pub render_json: bool,
    #[serde(default = "default_true")]
    pub render_text: bool,
    #[serde(default)]
    pub auto_refresh_seconds: Option<u64>,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl Default for DashboardRenderConfig {
    fn default() -> Self {
        Self {
            dashboard_id: "soma_control_tower".to_string(),
            dashboard_state_path: None,
            source_config_path: None,
            output_root: "target/soma_control_tower_render".to_string(),
            render_html: true,
            render_json: true,
            render_text: true,
            auto_refresh_seconds: None,
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl DashboardRenderConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&contents).map_err(|err| err.to_string())
    }

    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn validate_local_paths(&self) -> Vec<ReasonCode> {
        let remote = self.output_root.contains("://")
            || self
                .dashboard_state_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .source_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"));
        if remote {
            vec![
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]
        } else {
            Vec::new()
        }
    }

    pub fn artifact_dir(&self) -> PathBuf {
        Path::new(&self.output_root).join(&self.dashboard_id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardRenderStatus {
    Rendered,
    RenderedJsonOnly,
    RenderedTextOnly,
    MissingState,
    SecretRedactionFailed,
    DiagnosticOnly,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashboardRenderReport {
    pub dashboard_id: String,
    #[serde(default)]
    pub html_path: Option<String>,
    #[serde(default)]
    pub json_path: Option<String>,
    #[serde(default)]
    pub text_path: Option<String>,
    pub panel_count: usize,
    pub event_count: usize,
    pub candidate_count: usize,
    pub secret_redaction_passed: bool,
    pub render_status: DashboardRenderStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl DashboardRenderReport {
    pub fn to_text(&self) -> String {
        [
            "read_only_warning=dashboard rendering is read-only and local-only".to_string(),
            format!("dashboard_id={}", self.dashboard_id),
            format!("html_path={}", self.html_path.clone().unwrap_or_default()),
            format!("json_path={}", self.json_path.clone().unwrap_or_default()),
            format!("text_path={}", self.text_path.clone().unwrap_or_default()),
            format!("panel_count={}", self.panel_count),
            format!("event_count={}", self.event_count),
            format!("candidate_count={}", self.candidate_count),
            format!("secret_redaction_passed={}", self.secret_redaction_passed),
            format!("render_status={:?}", self.render_status),
            format!(
                "reason_codes={}",
                self.reason_codes
                    .iter()
                    .map(|reason| format!("{reason:?}"))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        ]
        .join("\n")
    }
}

#[derive(Clone, Debug, Default)]
pub struct DashboardRenderer;

impl DashboardRenderer {
    pub fn render(&self, config: &DashboardRenderConfig) -> Result<DashboardRenderReport, String> {
        if !config.validate_local_paths().is_empty() {
            return Err("dashboard-render config paths must be local".to_string());
        }

        let mut reason_codes = config.reason_codes.clone();
        reason_codes.push(ReasonCode::DashboardRendered);

        let state = if let Some(path) = &config.dashboard_state_path {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str::<DashboardState>(&text).map_err(|err| err.to_string())?
        } else if let Some(path) = &config.source_config_path {
            let source = DashboardSourceConfig::from_toml_path(Path::new(path))?;
            DashboardSnapshotBuilder.build(&source)?
        } else {
            return Ok(DashboardRenderReport {
                dashboard_id: config.dashboard_id.clone(),
                html_path: None,
                json_path: None,
                text_path: None,
                panel_count: 11,
                event_count: 0,
                candidate_count: 0,
                secret_redaction_passed: true,
                render_status: DashboardRenderStatus::MissingState,
                reason_codes: stable_reason_codes(&reason_codes),
            });
        };

        let (state, redaction_report) = redact_dashboard_state(&state)?;
        let artifact_dir = config.artifact_dir();
        fs::create_dir_all(&artifact_dir).map_err(|err| err.to_string())?;

        let mut html_path = None;
        let mut json_path = None;
        let mut text_path = None;

        if redaction_report.passed {
            if config.render_json {
                let path = artifact_dir.join("dashboard_state.json");
                fs::write(&path, state.to_json_string()?).map_err(|err| err.to_string())?;
                json_path = Some(path.display().to_string());
            }
            if config.render_text {
                let path = artifact_dir.join("dashboard_state.txt");
                fs::write(&path, state.to_text()).map_err(|err| err.to_string())?;
                text_path = Some(path.display().to_string());
            }
            if config.render_html {
                let path = artifact_dir.join("dashboard.html");
                fs::write(&path, render_html(&state, config.auto_refresh_seconds))
                    .map_err(|err| err.to_string())?;
                html_path = Some(path.display().to_string());
            }
        }

        let render_status = if !redaction_report.passed {
            DashboardRenderStatus::SecretRedactionFailed
        } else if config.render_html {
            DashboardRenderStatus::Rendered
        } else if config.render_json {
            DashboardRenderStatus::RenderedJsonOnly
        } else if config.render_text {
            DashboardRenderStatus::RenderedTextOnly
        } else {
            DashboardRenderStatus::DiagnosticOnly
        };

        Ok(DashboardRenderReport {
            dashboard_id: config.dashboard_id.clone(),
            html_path,
            json_path,
            text_path,
            panel_count: 11,
            event_count: state.audit_timeline.events.len(),
            candidate_count: state.candidate_panel.candidates.len(),
            secret_redaction_passed: redaction_report.passed,
            render_status,
            reason_codes: stable_reason_codes(&reason_codes),
        })
    }
}

fn render_html(state: &DashboardState, auto_refresh_seconds: Option<u64>) -> String {
    let refresh = auto_refresh_seconds
        .map(|seconds| format!("<meta http-equiv=\"refresh\" content=\"{seconds}\">"))
        .unwrap_or_default();
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n{refresh}\n<title>{}</title>\n<style>body{{font-family:-apple-system,BlinkMacSystemFont,Segoe UI,sans-serif;margin:24px;background:#0f172a;color:#e2e8f0}}section{{border:1px solid #334155;border-radius:12px;padding:16px;margin:0 0 16px 0;background:#111827}}h1,h2{{margin:0 0 12px 0}}ul{{padding-left:18px}}code,pre{{background:#020617;color:#cbd5e1;padding:2px 4px;border-radius:4px;white-space:pre-wrap}}.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px}}</style>\n</head>\n<body>\n<h1>{}</h1>\n<p>Read-only local dashboard. No broker, order, account, balance, holdings, or live execution controls.</p>\n<div class=\"grid\">\n{}\n</div>\n<section><h2>Audit timeline</h2><ul>{}</ul></section>\n</body>\n</html>",
        escape_html(&state.dashboard_id),
        escape_html(&state.dashboard_id),
        render_sections(state),
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

fn render_sections(state: &DashboardState) -> String {
    [
        format!(
            "<section><h2>Provider</h2><p>KoreanEquity: {}</p><p>USEquity: {}</p><p>KIS auth ready: {}</p><p>KIS endpoint policy: {}</p><p>KRX reference: {}</p><p>yfinance: {:?}</p></section>",
            escape_html(
                state
                    .provider_panel
                    .active_primary_provider_by_market
                    .get("KoreanEquity")
                    .map(String::as_str)
                    .unwrap_or("Unknown")
            ),
            escape_html(
                state
                    .provider_panel
                    .active_primary_provider_by_market
                    .get("USEquity")
                    .map(String::as_str)
                    .unwrap_or("Unknown")
            ),
            state.provider_panel.kis_status.auth_ready,
            escape_html(&state.provider_panel.kis_status.endpoint_policy_status),
            state.provider_panel.krx_status.reference_enabled,
            state.provider_panel.yfinance_status.mode,
        ),
        format!(
            "<section><h2>Evidence</h2><p>Official rows: {}</p><p>Outcome links: {}</p><p>Bottleneck: {}</p><p>Next action: {}</p></section>",
            state.evidence_panel.official_rows,
            state.evidence_panel.outcome_links,
            escape_html(&state.evidence_panel.current_bottleneck),
            escape_html(&state.evidence_panel.next_recommended_action),
        ),
        format!(
            "<section><h2>Committee</h2><p>Active personas: {}</p><p>Recommendation: {}</p><p>Disagreement: {:?}</p></section>",
            state.committee_panel.active_personas,
            escape_html(&state.committee_panel.recommendation),
            state.committee_panel.disagreement_score,
        ),
        format!(
            "<section><h2>Chair</h2><p>Decision: {}</p><p>Human confirm required: {}</p><p>Selected speakers: {}</p></section>",
            escape_html(&state.chair_panel.final_decision),
            state.chair_panel.human_confirm_required,
            escape_html(&state.chair_panel.selected_speakers.join(", ")),
        ),
        format!(
            "<section><h2>Risk</h2><p>Mode: {}</p><p>Default deny: {}</p><p>Emergency stop: {}</p><p>Last decision: {}</p></section>",
            escape_html(&state.risk_panel.risk_governor_mode),
            state.risk_panel.default_deny_active,
            state.risk_panel.emergency_stop_active,
            escape_html(state.risk_panel.last_risk_decision.as_deref().unwrap_or("None")),
        ),
        format!(
            "<section><h2>Candidates</h2><p>Total: {}</p><p>Human confirm: {}</p><p>Paper approved: {}</p><pre>{}</pre></section>",
            state.candidate_panel.candidates.len(),
            state.candidate_panel.human_confirm_candidates,
            state.candidate_panel.paper_approved_candidates,
            escape_html(
                &state
                    .candidate_panel
                    .candidates
                    .iter()
                    .map(|candidate| format!(
                        "{} {:?} {} hold={} dismiss={} reanalysis={} paper_confirmed={} feedbacks={} thesis_notes={}",
                        candidate.candidate_id,
                        candidate.status,
                        candidate.symbol,
                        candidate.owner_hold_active,
                        candidate.owner_dismissed,
                        candidate.owner_reanalysis_requested,
                        candidate.owner_paper_confirmed,
                        candidate.owner_feedback_history.len(),
                        candidate.linked_thesis_notes.len()
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ),
        format!(
            "<section><h2>Paper positions</h2><p>Open: {}</p><p>Closed: {}</p></section>",
            state.paper_position_panel.open_positions.len(),
            state.paper_position_panel.closed_positions.len(),
        ),
        format!(
            "<section><h2>Human confirm</h2><p>Pending: {}</p><pre>{}</pre></section>",
            state.human_confirm_panel.pending_items.len(),
            escape_html(
                &state
                    .human_confirm_panel
                    .pending_items
                    .iter()
                    .map(|item| format!(
                        "{} {:?} {:?} owner_allowed={:?} owner_forbidden={:?} paper_confirm_allowed={} why={}",
                        item.confirm_id,
                        item.safe_actions,
                        item.forbidden_actions,
                        item.allowed_owner_actions,
                        item.forbidden_owner_actions,
                        item.paper_confirm_allowed,
                        item.paper_confirm_explanation
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ),
        format!(
            "<section><h2>Owner</h2><p>Pending review: {}</p><p>Blocked inputs: {}</p><p>Paper confirmed: {}</p><pre>{}</pre></section>",
            state.owner_panel.pending_review_items.len(),
            state.owner_panel.blocked_owner_inputs.len(),
            state.owner_panel.paper_confirmed_items.len(),
            escape_html(
                &state
                    .owner_panel
                    .recent_owner_inputs
                    .iter()
                    .map(|input| {
                        format!("{} {:?} {:?}", input.owner_input_id, input.input_kind, input.status)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ),
        format!(
            "<section><h2>Bottleneck</h2><p>Primary: {}</p><p>Next action: {}</p></section>",
            escape_html(&state.bottleneck_panel.primary_bottleneck),
            escape_html(&state.bottleneck_panel.next_action),
        ),
        format!(
            "<section><h2>Overview</h2><p>System mode: {:?}</p><p>Warnings: {}</p><p>Fingerprint: {}</p></section>",
            state.system_mode,
            escape_html(&state.warnings.join(" | ")),
            escape_html(&state.fingerprint),
        ),
    ]
    .join("\n")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
