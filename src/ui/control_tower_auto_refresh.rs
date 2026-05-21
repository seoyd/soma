use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash_string, stable_reason_codes};
use crate::experiment::kis_market_data_smoke::KISMarketDataEvidenceSmokeReport;
use crate::security::{
    SecretRedactionAuditConfig, SecretRedactionAuditReport, SecretRedactionAuditRunner,
    SecretRedactionStatus,
};

use super::control_tower_refresh::{
    ControlTowerRefreshConfig, ControlTowerRefreshRunner, ControlTowerRefreshStatus,
};
use super::control_tower_v1::ControlTowerV1Config;
use super::dashboard_v1_renderer::{DashboardV1RenderStatus, DashboardV1Renderer};

fn default_output_root() -> String {
    "target/sprint58/control_tower_auto_refresh".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlTowerAutoRefreshConfig {
    pub refresh_id: String,
    #[serde(default)]
    pub control_tower_refresh_config_path: Option<String>,
    #[serde(default)]
    pub source_smoke_report_paths: Vec<String>,
    #[serde(default)]
    pub secret_redaction_audit_config_path: Option<String>,
    #[serde(default)]
    pub secret_redaction_audit_report_paths: Vec<String>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlTowerAutoRefreshStatus {
    AutoRefreshed,
    AutoRefreshedWithWarnings,
    MissingSourceArtifacts,
    SecretRedactionFailed,
    UnsafeControlDetected,
    #[default]
    DiagnosticOnly,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlTowerAutoRefreshReport {
    pub refresh_id: String,
    pub source_smoke_report: KISMarketDataEvidenceSmokeReport,
    pub dashboard_state_path: String,
    #[serde(default)]
    pub dashboard_html_path: Option<String>,
    #[serde(default)]
    pub dashboard_text_path: Option<String>,
    #[serde(default)]
    pub next_actions_path: Option<String>,
    #[serde(default)]
    pub owner_drafts_dir: Option<String>,
    pub secret_redaction_report: SecretRedactionAuditReport,
    pub refresh_status: ControlTowerAutoRefreshStatus,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ControlTowerAutoRefreshRunner;

impl Default for ControlTowerAutoRefreshConfig {
    fn default() -> Self {
        Self {
            refresh_id: "sprint58-control-tower-auto-refresh".to_string(),
            control_tower_refresh_config_path: None,
            source_smoke_report_paths: Vec::new(),
            secret_redaction_audit_config_path: None,
            secret_redaction_audit_report_paths: Vec::new(),
            output_root: default_output_root(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        }
    }
}

impl ControlTowerAutoRefreshConfig {
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        toml::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.refresh_id.trim().is_empty() {
            return Err("control tower auto-refresh id must not be empty".to_string());
        }
        if self
            .control_tower_refresh_config_path
            .iter()
            .chain(self.source_smoke_report_paths.iter())
            .chain(self.secret_redaction_audit_config_path.iter())
            .chain(self.secret_redaction_audit_report_paths.iter())
            .chain(std::iter::once(&self.output_root))
            .any(|path| path.contains("://"))
        {
            return Err("control tower auto-refresh paths must be local".to_string());
        }
        Ok(())
    }

    pub fn artifact_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.refresh_id)
    }
}

impl ControlTowerAutoRefreshReport {
    pub fn to_text(&self) -> String {
        [
            "read_only_warning=control tower auto-refresh is local-only, deterministic, and read-only".to_string(),
            format!("refresh_id={}", self.refresh_id),
            format!("dashboard_state_path={}", self.dashboard_state_path),
            format!(
                "dashboard_html_path={}",
                self.dashboard_html_path.clone().unwrap_or_default()
            ),
            format!(
                "dashboard_text_path={}",
                self.dashboard_text_path.clone().unwrap_or_default()
            ),
            format!("next_actions_path={}", self.next_actions_path.clone().unwrap_or_default()),
            format!("owner_drafts_dir={}", self.owner_drafts_dir.clone().unwrap_or_default()),
            format!("smoke_status={:?}", self.source_smoke_report.final_status),
            format!(
                "secret_redaction_status={:?}",
                self.secret_redaction_report.redaction_status
            ),
            format!("refresh_status={:?}", self.refresh_status),
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

    pub fn fingerprint(&self) -> String {
        stable_hash_string(&self.to_text())
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let text_path = output_dir.join("control_tower_auto_refresh.txt");
        fs::write(&text_path, self.to_text()).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("control_tower_auto_refresh.json"),
            serde_json::to_string_pretty(self).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(text_path)
    }
}

impl ControlTowerAutoRefreshRunner {
    pub fn run(
        &self,
        config: &ControlTowerAutoRefreshConfig,
    ) -> Result<ControlTowerAutoRefreshReport, String> {
        config.validate()?;
        let source_smoke_report =
            load_latest::<KISMarketDataEvidenceSmokeReport>(&config.source_smoke_report_paths)?
                .ok_or_else(|| "missing source smoke report".to_string())?;
        let secret_redaction_report = if let Some(path) = &config.secret_redaction_audit_config_path
        {
            let audit_config = SecretRedactionAuditConfig::from_toml_path(Path::new(path))?;
            SecretRedactionAuditRunner::default().run(&audit_config)?
        } else {
            load_latest::<SecretRedactionAuditReport>(&config.secret_redaction_audit_report_paths)?
                .unwrap_or(SecretRedactionAuditReport {
                    audit_id: format!("{}-diagnostic", config.refresh_id),
                    redaction_status: SecretRedactionStatus::DiagnosticOnly,
                    ..SecretRedactionAuditReport::default()
                })
        };
        let mut base_config = if let Some(path) = &config.control_tower_refresh_config_path {
            ControlTowerRefreshConfig::from_toml_path(Path::new(path))?
        } else {
            ControlTowerRefreshConfig::default()
        };
        base_config.refresh_id = config.refresh_id.clone();
        base_config.output_root = config.output_root.clone();
        let mut output = ControlTowerRefreshRunner::default().run(
            &base_config,
            config
                .control_tower_refresh_config_path
                .as_deref()
                .map(Path::new),
            None,
            None,
        )?;
        output.state.kis_monitor_panel.collection_plan_status =
            format!("{:?}", source_smoke_report.collection_plan_status);
        output.state.kis_monitor_panel.last_collection_status =
            Some(format!("{:?}", source_smoke_report.final_status));
        output.state.kis_monitor_panel.latest_depth_status =
            source_smoke_report.evidence_depth_status.clone();
        output.state.kis_monitor_panel.next_kis_actions = vec![
            format!("auth_closure={:?}", source_smoke_report.auth_closure_status),
            format!("dry_run={:?}", source_smoke_report.dry_run_status),
            format!("smoke={:?}", source_smoke_report.final_status),
        ];
        let mut render_config = if let Some(path) = &base_config.control_tower_v1_config_path {
            ControlTowerV1Config::from_toml_path(Path::new(path))?
        } else {
            ControlTowerV1Config::default()
        };
        render_config.control_tower_id = base_config.refresh_id.clone();
        render_config.output_root = base_config.output_root.clone();
        render_config.render_html = base_config.render_html;
        render_config.render_json = base_config.render_json;
        render_config.render_text = base_config.render_text;
        render_config.generate_owner_action_drafts = base_config.generate_owner_action_drafts;
        let render_report = DashboardV1Renderer::default()
            .render(&output.state, &render_config)
            .map_err(|err| format!("failed to render control tower auto-refresh outputs: {err}"))?;
        let refresh_status = if matches!(
            secret_redaction_report.redaction_status,
            SecretRedactionStatus::FailedSecretLeak
                | SecretRedactionStatus::FailedTokenLeak
                | SecretRedactionStatus::FailedAccountField
                | SecretRedactionStatus::FailedOrderField
        ) {
            ControlTowerAutoRefreshStatus::SecretRedactionFailed
        } else if render_report.unsafe_control_detected {
            ControlTowerAutoRefreshStatus::UnsafeControlDetected
        } else if matches!(
            output.report.refresh_status,
            ControlTowerRefreshStatus::MissingSourceReports
        ) {
            ControlTowerAutoRefreshStatus::MissingSourceArtifacts
        } else if matches!(
            render_report.render_status,
            DashboardV1RenderStatus::Rendered
        ) {
            ControlTowerAutoRefreshStatus::AutoRefreshed
        } else {
            ControlTowerAutoRefreshStatus::AutoRefreshedWithWarnings
        };
        let report = ControlTowerAutoRefreshReport {
            refresh_id: config.refresh_id.clone(),
            source_smoke_report,
            dashboard_state_path: render_report.json_path.clone().unwrap_or_default(),
            dashboard_html_path: render_report.html_path.clone(),
            dashboard_text_path: render_report.text_path.clone(),
            next_actions_path: output.report.next_actions_path.clone(),
            owner_drafts_dir: render_report.owner_action_draft_dir.clone(),
            secret_redaction_report,
            refresh_status,
            reason_codes: stable_reason_codes(
                &[
                    config.reason_codes.clone(),
                    vec![ReasonCode::ControlTowerAutoRefreshBuilt],
                ]
                .concat(),
            ),
        };
        report.write_to_dir(&config.artifact_dir())?;
        Ok(report)
    }
}

fn load_latest<T: for<'de> Deserialize<'de>>(paths: &[String]) -> Result<Option<T>, String> {
    let mut latest = None;
    for path in paths {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        latest = Some(serde_json::from_str(&text).map_err(|err| err.to_string())?);
    }
    Ok(latest)
}
