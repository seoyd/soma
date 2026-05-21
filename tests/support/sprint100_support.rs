#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    CommitteeQualityWarningClosureConfig, Sprint100CommitteeClosureBundle,
    Sprint100CommitteeClosureRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir};

pub fn sprint100_config_from_example(
    name: &str,
    output_name: &str,
) -> CommitteeQualityWarningClosureConfig {
    let mut config = CommitteeQualityWarningClosureConfig::from_toml_path(&example_path(name))
        .expect("parse sprint100 config");
    if let Some(paths) = config.sprint99_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    if let Some(paths) = config.workspace_acceptance_truth_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint100(name: &str, output_name: &str) -> Sprint100CommitteeClosureBundle {
    let config = sprint100_config_from_example(name, output_name);
    Sprint100CommitteeClosureRunner::default()
        .run_sprint100_committee_closure(&config)
        .expect("run sprint100")
}

pub fn sprint100_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}
