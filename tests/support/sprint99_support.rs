#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    CommitteeQualityHardeningConfig, Sprint99CommitteeQualityHardeningBundle,
    Sprint99CommitteeQualityHardeningRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir};

pub fn sprint99_config_from_example(
    name: &str,
    output_name: &str,
) -> CommitteeQualityHardeningConfig {
    let mut config = CommitteeQualityHardeningConfig::from_toml_path(&example_path(name))
        .expect("parse sprint99 config");
    if let Some(paths) = config.workspace_acceptance_truth_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    if let Some(paths) = config.sprint98_bundle_paths.as_mut() {
        for path in paths.iter_mut() {
            *path = absolutize(path).display().to_string();
        }
    }
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint99(name: &str, output_name: &str) -> Sprint99CommitteeQualityHardeningBundle {
    let config = sprint99_config_from_example(name, output_name);
    Sprint99CommitteeQualityHardeningRunner::default()
        .run_sprint99_committee_quality_hardening(&config)
        .expect("run sprint99")
}

pub fn sprint99_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}
