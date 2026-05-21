use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_reason_codes};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardOpenStatus {
    #[default]
    PathPrinted,
    RejectedRemotePath,
    RejectedOutsideOutputRoot,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardOpenReport {
    pub html_path: String,
    #[serde(default)]
    pub local_open_command: Option<String>,
    pub status: DashboardOpenStatus,
    pub launched: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}

impl DashboardOpenReport {
    pub fn to_text(&self) -> String {
        [
            "local_file_warning=dashboard-open only resolves local generated html paths"
                .to_string(),
            format!("html_path={}", self.html_path),
            format!(
                "local_open_command={}",
                self.local_open_command.clone().unwrap_or_default()
            ),
            format!("status={:?}", self.status),
            format!("launched={}", self.launched),
        ]
        .join("\n")
    }
}

pub fn prepare_dashboard_open(
    output_root: &Path,
    html_path: &Path,
    allow_outside_output_root: bool,
) -> Result<DashboardOpenReport, String> {
    let html = html_path.display().to_string();
    if html.contains("://") {
        return Ok(DashboardOpenReport {
            html_path: html,
            local_open_command: None,
            status: DashboardOpenStatus::RejectedRemotePath,
            launched: false,
            reason_codes: stable_reason_codes(&[
                ReasonCode::LocalPathRejected,
                ReasonCode::RemotePathRejected,
            ]),
        });
    }

    let output_root = normalize(output_root);
    let html_path = normalize(html_path);
    if !allow_outside_output_root && !html_path.starts_with(&output_root) {
        return Ok(DashboardOpenReport {
            html_path: html,
            local_open_command: None,
            status: DashboardOpenStatus::RejectedOutsideOutputRoot,
            launched: false,
            reason_codes: stable_reason_codes(&[ReasonCode::LocalPathRejected]),
        });
    }

    let command = if cfg!(target_os = "macos") {
        format!("open {}", shell_escape(&html_path.display().to_string()))
    } else if cfg!(target_os = "windows") {
        format!("start {}", shell_escape(&html_path.display().to_string()))
    } else {
        format!(
            "xdg-open {}",
            shell_escape(&html_path.display().to_string())
        )
    };

    Ok(DashboardOpenReport {
        html_path: html_path.display().to_string(),
        local_open_command: Some(command),
        status: DashboardOpenStatus::PathPrinted,
        launched: false,
        reason_codes: stable_reason_codes(&[ReasonCode::LocalFileOnly]),
    })
}

fn normalize(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn shell_escape(path: &str) -> String {
    format!("'{}'", path.replace('\'', "'\\''"))
}
