#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    WorkspaceTimeoutReductionQueueBundle, WorkspaceTimeoutReductionQueueConfig,
    WorkspaceTimeoutReductionQueueRunner,
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

pub fn sprint118_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceTimeoutReductionQueueConfig {
    let mut config =
        WorkspaceTimeoutReductionQueueConfig::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint118 config");
    absolutize_paths(&mut config.sprint117_bundle_paths);
    absolutize_paths(&mut config.sprint117_truth_paths);
    absolutize_paths(&mut config.cargo_json_execution_paths);
    absolutize_paths(&mut config.cargo_json_parse_paths);
    absolutize_paths(&mut config.no_run_execution_paths);
    absolutize_paths(&mut config.full_execution_paths);
    absolutize_paths(&mut config.timeout_cleanup_paths);
    config.output_root = named_output_dir("sprint118-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint118(name: &str, output_name: &str) -> WorkspaceTimeoutReductionQueueBundle {
    let config = sprint118_config_from_example(name, output_name);
    WorkspaceTimeoutReductionQueueRunner::default()
        .run(&config)
        .expect("run sprint118")
}

pub fn read_fixture<T: serde::de::DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint118_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint118-tests", name)
}
