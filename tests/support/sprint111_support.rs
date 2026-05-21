#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    WorkspaceTimeoutRootCauseBundle, WorkspaceTimeoutRootCauseConfig,
    WorkspaceTimeoutRootCauseRunner,
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

pub fn sprint111_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceTimeoutRootCauseConfig {
    let mut config = WorkspaceTimeoutRootCauseConfig::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint111 config");
    absolutize_paths(&mut config.sprint110_bundle_paths);
    absolutize_paths(&mut config.sprint110_validation_paths);
    absolutize_paths(&mut config.previous_ledger_paths);
    absolutize_paths(&mut config.previous_retired_target_manifest_paths);
    absolutize_paths(&mut config.cargo_json_progress_paths);
    absolutize_paths(&mut config.workspace_timeout_paths);
    config.output_root = named_output_dir("sprint111-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint111(name: &str, output_name: &str) -> WorkspaceTimeoutRootCauseBundle {
    let config = sprint111_config_from_example(name, output_name);
    WorkspaceTimeoutRootCauseRunner::default()
        .run(&config)
        .expect("run sprint111")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint111_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint111-tests", name)
}
