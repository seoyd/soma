#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    WorkspaceTimeoutTrackExecutionBundle, WorkspaceTimeoutTrackExecutionConfig,
    WorkspaceTimeoutTrackExecutionRunner,
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

pub fn sprint116_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceTimeoutTrackExecutionConfig {
    let mut config =
        WorkspaceTimeoutTrackExecutionConfig::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint116 config");
    absolutize_paths(&mut config.sprint115_bundle_paths);
    absolutize_paths(&mut config.sprint115_truth_paths);
    absolutize_paths(&mut config.workspace_timeout_backlog_paths);
    absolutize_paths(&mut config.no_run_observation_plan_paths);
    absolutize_paths(&mut config.full_observation_plan_paths);
    absolutize_paths(&mut config.cargo_json_observation_plan_paths);
    config.output_root = named_output_dir("sprint116-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint116(name: &str, output_name: &str) -> WorkspaceTimeoutTrackExecutionBundle {
    let config = sprint116_config_from_example(name, output_name);
    WorkspaceTimeoutTrackExecutionRunner::default()
        .run(&config)
        .expect("run sprint116")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint116_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint116-tests", name)
}
