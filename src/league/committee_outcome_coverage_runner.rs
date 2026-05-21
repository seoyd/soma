use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::core::ReasonCode;

use super::committee_counterfactual_audit::{
    CommitteeCounterfactualAuditConfig, CommitteeCounterfactualAuditRunner,
};
use super::committee_evidence_sufficiency::{
    CommitteeEvidenceSufficiencyGateConfig, evaluate_committee_evidence_sufficiency,
};
use super::committee_official_benchmark::{
    CommitteeOfficialBenchmarkConfig, CommitteeOfficialBenchmarkReport,
    CommitteeOfficialBenchmarkRunner,
};
use super::committee_outcome_coverage::{
    CommitteeOutcomeCoverageConfig, build_committee_outcome_coverage_report,
};
use super::committee_outcome_coverage_bundle::CommitteeOutcomeCoverageBundle;
use super::committee_outcome_linker::{
    CommitteeOutcomeLinkerConfig, OutcomeLinkedCommitteeScenarioPack,
};
use super::committee_performance_matrix::build_committee_performance_evidence_matrix;
use super::committee_replay::CommitteeReplayReport;
use super::official_committee_benchmark_bundle::CommitteeOfficialBenchmarkBundle;
use super::official_committee_pack::{
    OfficialCommitteeScenarioPack, OfficialCommitteeScenarioPackBuilder,
    OfficialCommitteeScenarioPackConfig,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommitteeOutcomeCoverageRunner;

#[derive(Default)]
struct LoadedCoverageInputs {
    packs: BTreeMap<String, OfficialCommitteeScenarioPack>,
    linked_packs: BTreeMap<String, OutcomeLinkedCommitteeScenarioPack>,
    replay_reports: Vec<CommitteeReplayReport>,
    storage_paths: BTreeSet<String>,
}

impl CommitteeOutcomeCoverageRunner {
    pub fn run(
        &self,
        config: &CommitteeOutcomeCoverageConfig,
    ) -> Result<CommitteeOutcomeCoverageBundle, String> {
        config.validate()?;
        let inputs = self.load_inputs(config)?;
        let total_rows = inputs
            .packs
            .values()
            .map(|pack| pack.rows.len())
            .sum::<usize>();
        if total_rows > config.max_rows {
            return Err(format!(
                "committee outcome coverage loaded {} rows which exceeds max_rows {}",
                total_rows, config.max_rows
            ));
        }
        let unique_symbols = inputs
            .packs
            .values()
            .flat_map(|pack| pack.rows.iter().map(|row| row.symbol.clone()))
            .collect::<BTreeSet<_>>();
        if unique_symbols.len() > config.max_symbols {
            return Err(format!(
                "committee outcome coverage loaded {} symbols which exceeds max_symbols {}",
                unique_symbols.len(),
                config.max_symbols
            ));
        }
        let storage_bytes = inputs
            .storage_paths
            .iter()
            .filter_map(|path| fs::metadata(path).ok())
            .map(|metadata| metadata.len() as usize)
            .sum::<usize>();
        if storage_bytes > config.max_bytes {
            return Err(format!(
                "committee outcome coverage input size {} exceeds max_bytes {}",
                storage_bytes, config.max_bytes
            ));
        }
        let packs = inputs.packs.values().cloned().collect::<Vec<_>>();
        let linked_packs = inputs.linked_packs.values().cloned().collect::<Vec<_>>();
        let counterfactual_audit_report = if !config.candle_series_paths.is_empty() {
            Some(
                CommitteeCounterfactualAuditRunner::default().run(
                    &CommitteeCounterfactualAuditConfig {
                        audit_id: config.coverage_id.clone(),
                        scenario_pack_paths: config.scenario_pack_paths.clone(),
                        candle_series_paths: config.candle_series_paths.clone(),
                        outcome_reference_paths: linked_packs
                            .iter()
                            .flat_map(|pack| {
                                pack.linked_rows
                                    .iter()
                                    .filter_map(|row| row.outcome_reference.as_ref())
                                    .map(|reference| reference.outcome_id.clone())
                                    .collect::<Vec<_>>()
                            })
                            .take(0)
                            .collect(),
                        output_root: config.output_root.clone(),
                        allow_estimated_when_missing_candles: config
                            .allow_estimated_counterfactuals,
                        reason_codes: vec![ReasonCode::CommitteeOutcomeCoverageRunnerBuilt],
                        ..CommitteeCounterfactualAuditConfig::default()
                    },
                )?,
            )
        } else {
            None
        };
        let counterfactual_records = counterfactual_audit_report
            .as_ref()
            .map(|report| report.records.clone())
            .unwrap_or_default();
        let coverage_report = build_committee_outcome_coverage_report(
            config,
            &packs,
            &linked_packs,
            &counterfactual_records,
        );
        let performance_matrix = build_committee_performance_evidence_matrix(
            &config.coverage_id,
            &coverage_report,
            &linked_packs,
            &inputs.replay_reports,
            &counterfactual_records,
            config.allow_estimated_counterfactuals,
        );
        let sufficiency_gate_result = evaluate_committee_evidence_sufficiency(
            &CommitteeEvidenceSufficiencyGateConfig::default(),
            &coverage_report,
            counterfactual_audit_report.as_ref(),
            &performance_matrix,
        );
        let bundle = CommitteeOutcomeCoverageBundle::new(
            coverage_report,
            counterfactual_audit_report,
            performance_matrix,
            sufficiency_gate_result,
            format!(
                "output_dir={};input_paths={}",
                config.output_dir().display(),
                inputs
                    .storage_paths
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("|")
            ),
        );
        bundle.write_to_dir(&config.output_dir())?;
        Ok(bundle)
    }

    fn load_inputs(
        &self,
        config: &CommitteeOutcomeCoverageConfig,
    ) -> Result<LoadedCoverageInputs, String> {
        let mut inputs = LoadedCoverageInputs::default();
        for path in &config.committee_benchmark_bundle_paths {
            self.load_bundle_path(path, &mut inputs)?;
        }
        for path in &config.official_benchmark_report_paths {
            self.load_report_path(path, &mut inputs)?;
        }
        for path in &config.outcome_linked_pack_paths {
            self.load_linked_pack_path(path, &mut inputs)?;
        }
        for path in &config.scenario_pack_paths {
            self.load_pack_path(path, &mut inputs)?;
        }
        Ok(inputs)
    }

    fn load_bundle_path(
        &self,
        path: &str,
        inputs: &mut LoadedCoverageInputs,
    ) -> Result<(), String> {
        inputs.storage_paths.insert(path.to_string());
        let bundle = if path.ends_with(".toml") {
            CommitteeOfficialBenchmarkRunner::default().run_bundle(
                &CommitteeOfficialBenchmarkConfig::from_toml_path(Path::new(path))?,
            )?
        } else {
            let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
            serde_json::from_str::<CommitteeOfficialBenchmarkBundle>(&text)
                .map_err(|err| err.to_string())?
        };
        inputs.packs.insert(
            bundle.official_scenario_pack.pack_id.clone(),
            bundle.official_scenario_pack.clone(),
        );
        inputs.linked_packs.insert(
            bundle.outcome_linked_pack.pack.pack_id.clone(),
            bundle.outcome_linked_pack.clone(),
        );
        inputs
            .replay_reports
            .push(bundle.committee_benchmark_report.replay_report.clone());
        Ok(())
    }

    fn load_report_path(
        &self,
        path: &str,
        inputs: &mut LoadedCoverageInputs,
    ) -> Result<(), String> {
        if path.ends_with(".toml") {
            return self.load_bundle_path(path, inputs);
        }
        inputs.storage_paths.insert(path.to_string());
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        if let Ok(bundle) = serde_json::from_str::<CommitteeOfficialBenchmarkBundle>(&text) {
            inputs.packs.insert(
                bundle.official_scenario_pack.pack_id.clone(),
                bundle.official_scenario_pack.clone(),
            );
            inputs.linked_packs.insert(
                bundle.outcome_linked_pack.pack.pack_id.clone(),
                bundle.outcome_linked_pack.clone(),
            );
            inputs
                .replay_reports
                .push(bundle.committee_benchmark_report.replay_report.clone());
            return Ok(());
        }
        let report = serde_json::from_str::<CommitteeOfficialBenchmarkReport>(&text)
            .map_err(|err| err.to_string())?;
        inputs
            .replay_reports
            .push(report.committee_benchmark_report.replay_report);
        Ok(())
    }

    fn load_linked_pack_path(
        &self,
        path: &str,
        inputs: &mut LoadedCoverageInputs,
    ) -> Result<(), String> {
        inputs.storage_paths.insert(path.to_string());
        let linked_pack = if path.ends_with(".toml") {
            let linker_config = CommitteeOutcomeLinkerConfig::from_toml_path(Path::new(path))?;
            super::committee_outcome_linker::CommitteeOutcomeLinker::default()
                .link_from_config(&linker_config)?
        } else {
            OutcomeLinkedCommitteeScenarioPack::from_json_path(Path::new(path))?
        };
        inputs
            .packs
            .insert(linked_pack.pack.pack_id.clone(), linked_pack.pack.clone());
        inputs
            .linked_packs
            .insert(linked_pack.pack.pack_id.clone(), linked_pack);
        Ok(())
    }

    fn load_pack_path(&self, path: &str, inputs: &mut LoadedCoverageInputs) -> Result<(), String> {
        inputs.storage_paths.insert(path.to_string());
        let pack = if path.ends_with(".toml") {
            OfficialCommitteeScenarioPackBuilder::default().build(
                &OfficialCommitteeScenarioPackConfig::from_toml_path(Path::new(path))?,
            )?
        } else {
            OfficialCommitteeScenarioPack::from_json_path(Path::new(path))?
        };
        inputs.packs.insert(pack.pack_id.clone(), pack);
        Ok(())
    }
}
