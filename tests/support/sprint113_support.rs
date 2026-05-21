#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    RealWorkspaceObservationDrilldownBundle, RealWorkspaceObservationDrilldownConfig,
    RealWorkspaceObservationDrilldownRunner,
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

pub fn sprint113_config_from_example(
    name: &str,
    output_name: &str,
) -> RealWorkspaceObservationDrilldownConfig {
    let mut config =
        RealWorkspaceObservationDrilldownConfig::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint113 config");
    absolutize_paths(&mut config.sprint112_bundle_paths);
    absolutize_paths(&mut config.sprint112_truth_paths);
    absolutize_paths(&mut config.sprint112_verification_patch_paths);
    absolutize_paths(&mut config.cargo_json_progress_paths);
    absolutize_paths(&mut config.nextest_observation_paths);
    absolutize_paths(&mut config.sccache_observation_paths);
    absolutize_paths(&mut config.suspect_target_paths);
    config.output_root = named_output_dir("sprint113-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint113(name: &str, output_name: &str) -> RealWorkspaceObservationDrilldownBundle {
    let config = sprint113_config_from_example(name, output_name);
    RealWorkspaceObservationDrilldownRunner::default()
        .run(&config)
        .expect("run sprint113")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint113_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint113-tests", name)
}
