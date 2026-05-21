#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    Sprint98CommitteeOwnedCoreBundle, Sprint98CommitteeOwnedCoreConfig,
    Sprint98CommitteeOwnedCoreRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir};

pub fn sprint98_config_from_example(
    name: &str,
    output_name: &str,
) -> Sprint98CommitteeOwnedCoreConfig {
    let mut config = Sprint98CommitteeOwnedCoreConfig::from_toml_path(&example_path(name))
        .expect("parse sprint98 config");
    if let Some(path) = config.sprint97_summary_path.as_mut() {
        *path = absolutize(path).display().to_string();
    }
    if let Some(path) = config.workspace_acceptance_truth_path.as_mut() {
        *path = absolutize(path).display().to_string();
    }
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint98(name: &str, output_name: &str) -> Sprint98CommitteeOwnedCoreBundle {
    let config = sprint98_config_from_example(name, output_name);
    Sprint98CommitteeOwnedCoreRunner::default()
        .run_sprint98_committee_owned_core(&config)
        .expect("run sprint98")
}

pub fn sprint98_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}
