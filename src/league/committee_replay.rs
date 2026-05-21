use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::{CoreCheckConfig, CoreCheckRunner, ReasonCode, stable_hash_string};

use super::chair_v0::ChairV0;
use super::committee_decision::{ChairCommitteeConfig, CommitteeDecisionRecord, CommitteeInput};
use super::committee_risk_bridge::{CommitteeFinalAction, CommitteeOutcome, CommitteeRiskBridge};
use super::committee_scenario_loader::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioRow,
    CommitteeScenarioSet,
};
use super::committee_smoke::CommitteeSmokeTestConfig;
use super::persona_vote::PersonaVote;
use super::trinity_personas::active_trinity_scorers;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitteeReplayConfig {
    pub replay_id: String,
    #[serde(default)]
    pub scenario_set_path: Option<String>,
    #[serde(default)]
    pub committee_smoke_config_path: Option<String>,
    pub output_root: String,
    #[serde(default = "default_max_decisions")]
    pub max_decisions: usize,
    #[serde(default)]
    pub require_core_check: bool,
    #[serde(default)]
    pub deterministic_seed: Option<u64>,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeReplayRecord {
    pub scenario_row: CommitteeScenarioRow,
    pub persona_votes: Vec<PersonaVote>,
    pub chair_decision_record: CommitteeDecisionRecord,
    pub risk_bridge_outcome: CommitteeOutcome,
    pub final_action: CommitteeFinalAction,
    pub replay_fingerprint: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitteeReplayReport {
    pub replay_id: String,
    pub records: Vec<CommitteeReplayRecord>,
    pub record_count: usize,
    pub source_summary: String,
    pub final_action_counts: BTreeMap<String, usize>,
    pub risk_denial_counts: BTreeMap<String, usize>,
    pub chair_decision_counts: BTreeMap<String, usize>,
    pub deterministic_fingerprint: String,
    pub reason_codes: Vec<ReasonCode>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeDebateReplay;

impl Default for CommitteeReplayConfig {
    fn default() -> Self {
        Self {
            replay_id: "committee_replay".to_string(),
            scenario_set_path: None,
            committee_smoke_config_path: None,
            output_root: "target/soma_committee_replay".to_string(),
            max_decisions: default_max_decisions(),
            require_core_check: false,
            deterministic_seed: None,
            reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
        }
    }
}

impl CommitteeReplayConfig {
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

    pub fn validate_local_paths(&self) -> Result<(), String> {
        if self.output_root.contains("://")
            || self
                .scenario_set_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
            || self
                .committee_smoke_config_path
                .as_deref()
                .is_some_and(|path| path.contains("://"))
        {
            return Err("committee replay paths must be local".to_string());
        }
        Ok(())
    }

    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.replay_id)
    }
}

impl CommitteeReplayReport {
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|err| err.to_string())
    }

    pub fn from_json_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        serde_json::from_str(&text).map_err(|err| err.to_string())
    }

    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("replay_id={}", self.replay_id),
            format!("record_count={}", self.record_count),
            format!("source_summary={}", self.source_summary),
            format!(
                "deterministic_fingerprint={}",
                self.deterministic_fingerprint
            ),
        ];
        for (action, count) in &self.final_action_counts {
            lines.push(format!("final_action={action};count={count}"));
        }
        for (decision, count) in &self.chair_decision_counts {
            lines.push(format!("chair_decision={decision};count={count}"));
        }
        for (reason, count) in &self.risk_denial_counts {
            lines.push(format!("risk_denial={reason};count={count}"));
        }
        lines.join("\n")
    }

    pub fn write_to_dir(&self, output_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let json_path = output_dir.join("committee_replay_report.json");
        fs::write(&json_path, self.to_json_string()?).map_err(|err| err.to_string())?;
        fs::write(
            output_dir.join("committee_replay_report.txt"),
            self.to_text(),
        )
        .map_err(|err| err.to_string())?;
        Ok(json_path)
    }
}

impl CommitteeDebateReplay {
    pub fn run(&self, config: &CommitteeReplayConfig) -> Result<CommitteeReplayReport, String> {
        config.validate_local_paths()?;
        if config.require_core_check {
            let _ = CoreCheckRunner::default().run(&CoreCheckConfig::default())?;
        }
        let scenario_set = load_scenario_set(config)?;
        self.run_for_scenario_set(&config.replay_id, &scenario_set, config.max_decisions)
    }

    pub fn run_for_scenario_set(
        &self,
        replay_id: &str,
        scenario_set: &CommitteeScenarioSet,
        max_decisions: usize,
    ) -> Result<CommitteeReplayReport, String> {
        let scorers = active_trinity_scorers();
        let chair = ChairV0 {
            config: ChairCommitteeConfig::default(),
        };
        let bridge = CommitteeRiskBridge::default();
        let mut records = Vec::new();
        for row in scenario_set.rows.iter().take(max_decisions) {
            let scoring_input = row.to_scoring_input();
            let persona_votes = scorers
                .iter()
                .map(|scorer| scorer.score(&scoring_input))
                .collect::<Vec<_>>();
            let committee_input = CommitteeInput {
                scoring_input: scoring_input.clone(),
                persona_votes: persona_votes.clone(),
                target_horizon: scoring_input.target_horizon,
                source_kind: scoring_input.source_kind,
                regime: scoring_input.regime,
                reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
            };
            let chair_decision_record = chair.evaluate(&committee_input);
            let risk_bridge_outcome = bridge.evaluate(
                &row.to_market_snapshot(),
                &row.to_risk_snapshot(),
                &scoring_input,
                chair_decision_record.clone(),
            );
            let replay_fingerprint = stable_hash_string(&format!(
                "{}|{:?}|{:?}|{:?}",
                row.scenario_row_id,
                persona_votes,
                chair_decision_record.final_decision,
                risk_bridge_outcome.final_action
            ));
            records.push(CommitteeReplayRecord {
                scenario_row: row.clone(),
                persona_votes,
                chair_decision_record,
                final_action: risk_bridge_outcome.final_action,
                risk_bridge_outcome,
                replay_fingerprint,
                reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
            });
        }
        let mut final_action_counts = BTreeMap::new();
        let mut risk_denial_counts = BTreeMap::new();
        let mut chair_decision_counts = BTreeMap::new();
        for record in &records {
            *final_action_counts
                .entry(format!("{:?}", record.final_action))
                .or_insert(0) += 1;
            *chair_decision_counts
                .entry(format!("{:?}", record.chair_decision_record.final_decision))
                .or_insert(0) += 1;
            for reason in &record.risk_bridge_outcome.risk_decision.reason_codes {
                *risk_denial_counts.entry(format!("{reason:?}")).or_insert(0) += 1;
            }
        }
        let deterministic_fingerprint = stable_hash_string(
            &records
                .iter()
                .map(|record| record.replay_fingerprint.clone())
                .collect::<Vec<_>>()
                .join("|"),
        );
        Ok(CommitteeReplayReport {
            replay_id: replay_id.to_string(),
            record_count: records.len(),
            source_summary: scenario_set.source_summary.clone(),
            final_action_counts,
            risk_denial_counts,
            chair_decision_counts,
            deterministic_fingerprint,
            records,
            reason_codes: vec![ReasonCode::CommitteeReplayBuilt],
        })
    }
}

fn load_scenario_set(config: &CommitteeReplayConfig) -> Result<CommitteeScenarioSet, String> {
    if let Some(path) = &config.scenario_set_path {
        CommitteeScenarioSet::from_json_path(Path::new(path))
    } else if let Some(path) = &config.committee_smoke_config_path {
        let smoke = CommitteeSmokeTestConfig::from_toml_path(Path::new(path))?;
        let loader_config = CommitteeScenarioLoadConfig::from_committee_smoke_config(&smoke);
        CommitteeScenarioLoader::default().load(&loader_config)
    } else {
        Err("committee replay requires --scenario-set or --committee-smoke-config".to_string())
    }
}

fn default_max_decisions() -> usize {
    50
}
