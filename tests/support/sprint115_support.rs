#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    ConsolidationStopResumeGovernanceBundle, ConsolidationStopResumeGovernanceConfig,
    ConsolidationStopResumeGovernanceRunner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
};

fn absolutize_paths(paths: &mut Option<Vec<String>>) {
    if let Some(paths) = paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize_fixture_path(path).display().to_string();
        }
    }
}

pub fn sprint115_config_from_example(
    name: &str,
    output_name: &str,
) -> ConsolidationStopResumeGovernanceConfig {
    let mut config =
        ConsolidationStopResumeGovernanceConfig::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint115 config");
    absolutize_paths(&mut config.sprint114_bundle_paths);
    absolutize_paths(&mut config.sprint114_truth_paths);
    absolutize_paths(&mut config.assertion_inventory_paths);
    absolutize_paths(&mut config.destination_capacity_paths);
    absolutize_paths(&mut config.evidence_blur_paths);
    absolutize_paths(&mut config.workspace_timeout_paths);
    config.output_root = named_output_dir("sprint115-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint115(name: &str, output_name: &str) -> ConsolidationStopResumeGovernanceBundle {
    let config = sprint115_config_from_example(name, output_name);
    ConsolidationStopResumeGovernanceRunner::default()
        .run(&config)
        .expect("run sprint115")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint115_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint115-tests", name)
}
