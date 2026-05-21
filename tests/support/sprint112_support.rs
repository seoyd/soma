#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    WorkspaceDiagnosticPilotV1Bundle, WorkspaceDiagnosticPilotV1Config,
    WorkspaceDiagnosticPilotV1Runner,
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

pub fn sprint112_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceDiagnosticPilotV1Config {
    let mut config = WorkspaceDiagnosticPilotV1Config::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint112 config");
    absolutize_paths(&mut config.sprint111_bundle_paths);
    absolutize_paths(&mut config.sprint111_truth_paths);
    absolutize_paths(&mut config.cargo_json_progress_paths);
    absolutize_paths(&mut config.timing_capture_paths);
    absolutize_paths(&mut config.nextest_paths);
    absolutize_paths(&mut config.sccache_paths);
    config.output_root = named_output_dir("sprint112-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint112(name: &str, output_name: &str) -> WorkspaceDiagnosticPilotV1Bundle {
    let config = sprint112_config_from_example(name, output_name);
    WorkspaceDiagnosticPilotV1Runner::default()
        .run(&config)
        .expect("run sprint112")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint112_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint112-tests", name)
}
