use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{ReasonCode, stable_hash};

use super::campaign::{ResearchCampaignConfig, ResearchCampaignReport};
use super::render::campaign_summary_to_text;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceStoreConfig {
    pub root_path: String,
    pub campaign_id: String,
    pub allow_overwrite: bool,
    pub created_at_ms: Option<u64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSnapshot {
    pub snapshot_id: String,
    pub campaign_id: String,
    pub matrix_id: Option<String>,
    pub created_at_ms: Option<u64>,
    pub input_fingerprint: String,
    pub report_fingerprint: String,
    pub config_fingerprint: String,
    pub report_path: String,
    pub summary_path: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EvidenceStore;

impl EvidenceStore {
    pub fn compute_fingerprint(input: &str) -> String {
        format!("{:016x}", stable_hash(input))
    }

    pub fn save_campaign_report(
        &self,
        store_config: &EvidenceStoreConfig,
        campaign_config: &ResearchCampaignConfig,
        matrix_inputs: &[String],
        report: &ResearchCampaignReport,
    ) -> Result<EvidenceSnapshot, String> {
        if store_config.root_path.contains("://") {
            return Err("remote evidence store path rejected".to_string());
        }
        let config_text = campaign_config.to_toml_string()?;
        let report_text = report.to_json_string()?;
        let summary_text = campaign_summary_to_text(report);
        let input_fingerprint = Self::compute_fingerprint(&matrix_inputs.join("\n---\n"));
        let config_fingerprint = Self::compute_fingerprint(&config_text);
        let report_fingerprint = Self::compute_fingerprint(&report_text);
        let snapshot_id = format!("{}-{report_fingerprint}", store_config.campaign_id);
        let snapshot_dir = Path::new(&store_config.root_path).join(&store_config.campaign_id);
        fs::create_dir_all(&snapshot_dir).map_err(|err| err.to_string())?;
        let report_path = snapshot_dir.join(format!("{snapshot_id}.report.json"));
        let summary_path = snapshot_dir.join(format!("{snapshot_id}.summary.txt"));
        let snapshot_path = snapshot_dir.join(format!("{snapshot_id}.snapshot.json"));
        if !store_config.allow_overwrite
            && (report_path.exists() || summary_path.exists() || snapshot_path.exists())
        {
            return Err("evidence snapshot already exists".to_string());
        }
        fs::write(&report_path, report_text).map_err(|err| err.to_string())?;
        fs::write(&summary_path, summary_text).map_err(|err| err.to_string())?;
        let snapshot = EvidenceSnapshot {
            snapshot_id,
            campaign_id: store_config.campaign_id.clone(),
            matrix_id: None,
            created_at_ms: store_config.created_at_ms,
            input_fingerprint,
            report_fingerprint,
            config_fingerprint,
            report_path: report_path.display().to_string(),
            summary_path: summary_path.display().to_string(),
            reason_codes: vec![ReasonCode::DeterministicPath],
        };
        fs::write(
            &snapshot_path,
            serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        Ok(snapshot)
    }

    pub fn list_snapshots(
        &self,
        store_config: &EvidenceStoreConfig,
    ) -> Result<Vec<EvidenceSnapshot>, String> {
        if store_config.root_path.contains("://") {
            return Err("remote evidence store path rejected".to_string());
        }
        let snapshot_dir = Path::new(&store_config.root_path).join(&store_config.campaign_id);
        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }
        let mut snapshots = fs::read_dir(&snapshot_dir)
            .map_err(|err| err.to_string())?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".snapshot.json"))
            })
            .map(|path| {
                let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
                serde_json::from_str::<EvidenceSnapshot>(&text).map_err(|err| err.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        snapshots.sort_by(|left, right| left.snapshot_id.cmp(&right.snapshot_id));
        Ok(snapshots)
    }

    pub fn load_snapshot_summary(&self, path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|err| err.to_string())
    }
}
