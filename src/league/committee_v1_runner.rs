use std::path::Path;

use crate::core::ReasonCode;
use crate::data::{EvidenceSourceKind, ProviderMarket};

use super::chair_calibration::build_chair_calibration_report;
use super::chair_diagnostics::build_chair_diagnostics;
use super::committee_decision::CommitteeInput;
use super::committee_decision_quality::build_committee_decision_quality_report;
use super::committee_diagnostics::CommitteeDiagnosticsConfig;
use super::committee_evidence_quality::build_committee_evidence_quality_report;
use super::committee_replay::{CommitteeDebateReplay, CommitteeReplayConfig};
use super::committee_risk_bridge::CommitteeRiskBridge;
use super::committee_scenario_loader::{
    CommitteeScenarioLoadConfig, CommitteeScenarioLoader, CommitteeScenarioSet,
    CommitteeScenarioSourceKind,
};
use super::committee_smoke::CommitteeSmokeTestConfig;
use super::committee_v1::CommitteeV1RunConfig;
use super::committee_v1_bundle::{
    ChairDiagnosticsSummary, CommitteeV1FinalStatus, CommitteeV1ReportBundle,
    RiskDiagnosticsSummary,
};
use super::committee_v1_readiness::{
    CommitteeV1ReadinessStatus, build_committee_v1_readiness_report,
};
use super::persona_conflict_matrix::build_persona_conflict_matrix;
use super::risk_bridge_diagnostics::build_risk_bridge_diagnostics;
use super::risk_calibration::build_risk_calibration_report;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeV1Runner;

impl CommitteeV1Runner {
    pub fn run(&self, config: &CommitteeV1RunConfig) -> Result<CommitteeV1ReportBundle, String> {
        config.validate()?;
        let scenario_set = resolve_scenario_set(config)?;
        let replay_report = if let Some(path) = &config.replay_config_path {
            let replay_config = CommitteeReplayConfig::from_toml_path(Path::new(path))?;
            CommitteeDebateReplay::default().run(&replay_config)?
        } else {
            CommitteeDebateReplay::default().run_for_scenario_set(
                &format!("{}-replay", config.run_id),
                &scenario_set,
                config.max_decisions,
            )?
        };

        let bridge = CommitteeRiskBridge::default();
        let chair_reports = replay_report
            .records
            .iter()
            .map(|record| {
                build_chair_diagnostics(
                    &CommitteeInput {
                        scoring_input: record.scenario_row.to_scoring_input(),
                        persona_votes: record.persona_votes.clone(),
                        target_horizon: record.scenario_row.target_horizon,
                        source_kind: record.scenario_row.evidence_source_kind,
                        regime: record.scenario_row.regime,
                        reason_codes: vec![ReasonCode::CommitteeV1RunnerBuilt],
                    },
                    &record.chair_decision_record,
                )
            })
            .collect::<Vec<_>>();
        let risk_reports = replay_report
            .records
            .iter()
            .map(|record| {
                build_risk_bridge_diagnostics(
                    &bridge,
                    &record.scenario_row.to_market_snapshot(),
                    &record.scenario_row.to_scoring_input(),
                    &record.chair_decision_record,
                    &record.risk_bridge_outcome,
                )
            })
            .collect::<Vec<_>>();
        let conflict_matrix = build_persona_conflict_matrix(&replay_report);
        let evidence_quality_report = build_committee_evidence_quality_report(&scenario_set);
        let decision_quality_report = build_committee_decision_quality_report(
            &replay_report,
            &chair_reports,
            &risk_reports,
            &conflict_matrix,
            &evidence_quality_report,
        );
        let chair_calibration_report =
            build_chair_calibration_report(&chair_reports, &evidence_quality_report);
        let risk_calibration_report = build_risk_calibration_report(
            &risk_reports,
            &evidence_quality_report,
            &decision_quality_report,
        );
        let v1_readiness_report = build_committee_v1_readiness_report(
            &evidence_quality_report,
            &decision_quality_report,
            &chair_calibration_report,
            &risk_calibration_report,
            &conflict_matrix,
        );
        let chair_diagnostics_summary = ChairDiagnosticsSummary::from_reports(chair_reports);
        let risk_diagnostics_summary = RiskDiagnosticsSummary::from_reports(risk_reports);
        let final_status = map_final_status(&v1_readiness_report.status);
        let final_recommendation = v1_readiness_report.next_recommendation;
        let mut warnings = evidence_quality_report.warnings.clone();
        warnings.extend(v1_readiness_report.warnings.clone());
        if !config.allow_crypto_only
            && evidence_quality_report.crypto_only_count == evidence_quality_report.scenario_count
            && evidence_quality_report.scenario_count > 0
        {
            warnings.push(
                "crypto-only committee evidence is excluded by Committee V1 config".to_string(),
            );
        }
        warnings.sort();
        warnings.dedup();
        let mut bundle = CommitteeV1ReportBundle {
            run_id: config.run_id.clone(),
            scenario_set: Some(scenario_set),
            replay_report,
            chair_diagnostics_summary,
            risk_diagnostics_summary,
            conflict_matrix,
            evidence_quality_report,
            decision_quality_report,
            chair_calibration_report,
            risk_calibration_report,
            v1_readiness_report,
            audit_summary: String::new(),
            storage_summary: Some(format!("output_dir={}", config.output_dir().display())),
            final_status,
            final_recommendation,
            warnings,
            reason_codes: vec![ReasonCode::CommitteeV1RunnerBuilt],
        };
        bundle.audit_summary = bundle.build_audit_summary();
        Ok(bundle)
    }
}

fn resolve_scenario_set(config: &CommitteeV1RunConfig) -> Result<CommitteeScenarioSet, String> {
    if let Some(path) = &config.scenario_load_config_path {
        let load_config = CommitteeScenarioLoadConfig::from_toml_path(Path::new(path))?;
        return CommitteeScenarioLoader::default().load(&load_config);
    }
    if let Some(path) = &config.replay_config_path {
        let replay_config = CommitteeReplayConfig::from_toml_path(Path::new(path))?;
        return load_scenario_set_from_replay_config(&replay_config);
    }
    if let Some(path) = &config.diagnostics_config_path {
        let diagnostics_config = CommitteeDiagnosticsConfig::from_toml_path(Path::new(path))?;
        if let Some(replay_path) = diagnostics_config.replay_config_path {
            let replay_config = CommitteeReplayConfig::from_toml_path(Path::new(&replay_path))?;
            return load_scenario_set_from_replay_config(&replay_config);
        }
        if let Some(load_path) = diagnostics_config.scenario_load_config_path {
            let load_config = CommitteeScenarioLoadConfig::from_toml_path(Path::new(&load_path))?;
            return CommitteeScenarioLoader::default().load(&load_config);
        }
        if let Some(smoke_path) = diagnostics_config.committee_smoke_config_path {
            let smoke = CommitteeSmokeTestConfig::from_toml_path(Path::new(&smoke_path))?;
            let load_config = CommitteeScenarioLoadConfig::from_committee_smoke_config(&smoke);
            return CommitteeScenarioLoader::default().load(&load_config);
        }
    }
    build_scenario_set_from_inputs(config)
}

fn load_scenario_set_from_replay_config(
    replay_config: &CommitteeReplayConfig,
) -> Result<CommitteeScenarioSet, String> {
    if let Some(path) = &replay_config.scenario_set_path {
        CommitteeScenarioSet::from_json_path(Path::new(path))
    } else if let Some(path) = &replay_config.committee_smoke_config_path {
        let smoke = CommitteeSmokeTestConfig::from_toml_path(Path::new(path))?;
        CommitteeScenarioLoader::default().load(
            &CommitteeScenarioLoadConfig::from_committee_smoke_config(&smoke),
        )
    } else {
        Err("committee-v1 replay config missing scenario source".to_string())
    }
}

fn build_scenario_set_from_inputs(
    config: &CommitteeV1RunConfig,
) -> Result<CommitteeScenarioSet, String> {
    let mut sets = Vec::new();
    for (index, path) in config.source_report_paths.iter().enumerate() {
        sets.push(CommitteeScenarioLoader::default().load(&load_config(
            config,
            &format!("{}-source-{index}", config.run_id),
            CommitteeScenarioSourceKind::SourceAwareBenchmarkReport,
            vec![path.clone()],
        ))?);
    }
    for (index, path) in config.yfinance_report_paths.iter().enumerate() {
        sets.push(CommitteeScenarioLoader::default().load(&load_config(
            config,
            &format!("{}-yfinance-{index}", config.run_id),
            CommitteeScenarioSourceKind::YahooResearchEvidenceReport,
            vec![path.clone()],
        ))?);
    }
    for (index, path) in config.evidence_lane_report_paths.iter().enumerate() {
        sets.push(CommitteeScenarioLoader::default().load(&load_config(
            config,
            &format!("{}-evidence-{index}", config.run_id),
            CommitteeScenarioSourceKind::EvidenceLaneReport,
            vec![path.clone()],
        ))?);
    }
    if !config.fixture_paths.is_empty() {
        for (index, path) in config.fixture_paths.iter().enumerate() {
            sets.push(CommitteeScenarioLoader::default().load(&load_config(
                config,
                &format!("{}-fixture-{index}", config.run_id),
                CommitteeScenarioSourceKind::Fixture,
                vec![path.clone()],
            ))?);
        }
    } else if sets.is_empty() && config.allow_fixture {
        sets.push(CommitteeScenarioLoader::default().load(&load_config(
            config,
            &format!("{}-fixture", config.run_id),
            CommitteeScenarioSourceKind::Fixture,
            Vec::new(),
        ))?);
    }
    merge_scenario_sets(config, sets)
}

fn load_config(
    config: &CommitteeV1RunConfig,
    scenario_id: &str,
    source_kind: CommitteeScenarioSourceKind,
    input_paths: Vec<String>,
) -> CommitteeScenarioLoadConfig {
    CommitteeScenarioLoadConfig {
        scenario_id: scenario_id.to_string(),
        source_kind,
        input_paths,
        output_root: config.output_root.clone(),
        max_scenarios: config.max_scenarios,
        require_core_check: config.require_core_check,
        allow_yfinance_research: config.allow_yfinance_research,
        allow_fixture: config.allow_fixture,
        allow_synthetic_test: false,
        min_data_quality: if config.allow_summary_derived_rows {
            0.70
        } else {
            0.80
        },
        reason_codes: vec![ReasonCode::CommitteeV1Built],
    }
}

fn merge_scenario_sets(
    config: &CommitteeV1RunConfig,
    sets: Vec<CommitteeScenarioSet>,
) -> Result<CommitteeScenarioSet, String> {
    let mut rows = sets
        .into_iter()
        .flat_map(|set| set.rows.into_iter())
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.scenario_row_id.cmp(&right.scenario_row_id));
    if rows.len() > config.max_scenarios {
        rows.truncate(config.max_scenarios);
    }
    if !config.allow_crypto_only {
        rows.retain(|row| row.market != ProviderMarket::Crypto);
    }
    let row_count = rows.len();
    let official_row_count = rows
        .iter()
        .filter(|row| row.evidence_source_kind.readiness_eligible())
        .count();
    let research_only_row_count = rows
        .iter()
        .filter(|row| row.evidence_source_kind == EvidenceSourceKind::YFinanceResearch)
        .count();
    let fixture_row_count = rows
        .iter()
        .filter(|row| matches!(row.source_kind, CommitteeScenarioSourceKind::Fixture))
        .count();
    let mut source_summary = rows
        .iter()
        .map(|row| format!("{:?}", row.source_kind))
        .collect::<Vec<_>>();
    source_summary.sort();
    source_summary.dedup();
    Ok(CommitteeScenarioSet {
        scenario_id: config.run_id.clone(),
        rows,
        source_summary: source_summary.join("|"),
        row_count,
        official_row_count,
        research_only_row_count,
        fixture_row_count,
        skipped_row_count: 0,
        reason_codes: vec![ReasonCode::CommitteeV1Built],
    })
}

fn map_final_status(status: &CommitteeV1ReadinessStatus) -> CommitteeV1FinalStatus {
    match status {
        CommitteeV1ReadinessStatus::ReadyForMoreEvidence
        | CommitteeV1ReadinessStatus::ReadyForCommitteeBenchmark
        | CommitteeV1ReadinessStatus::ReadyForSixPersonaDesignReviewOnly => {
            CommitteeV1FinalStatus::CommitteeV1ResearchReady
        }
        CommitteeV1ReadinessStatus::ReadyForChairTuning
        | CommitteeV1ReadinessStatus::NotReadyGroupthink => {
            CommitteeV1FinalStatus::CommitteeV1NeedsChairTuning
        }
        CommitteeV1ReadinessStatus::ReadyForRiskReview
        | CommitteeV1ReadinessStatus::NotReadyRiskUnstable => {
            CommitteeV1FinalStatus::CommitteeV1NeedsRiskReview
        }
        CommitteeV1ReadinessStatus::NotReadyResearchOnly => {
            CommitteeV1FinalStatus::CommitteeV1ResearchOnly
        }
        CommitteeV1ReadinessStatus::NotReadyFixtureOnly => {
            CommitteeV1FinalStatus::CommitteeV1FixtureOnly
        }
        CommitteeV1ReadinessStatus::NotReadyEvidenceTooWeak
        | CommitteeV1ReadinessStatus::NotReadyTooFewSamples => {
            CommitteeV1FinalStatus::CommitteeV1NeedsEvidence
        }
    }
}
