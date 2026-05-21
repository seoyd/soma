#![allow(dead_code)]

use std::path::PathBuf;

use serde::de::DeserializeOwned;
use soma_zero::{
    MixedFamilyIsolationV1Bundle, MixedFamilyIsolationV1Config, MixedFamilyIsolationV1Runner,
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

pub fn sprint114_config_from_example(
    name: &str,
    output_name: &str,
) -> MixedFamilyIsolationV1Config {
    let mut config = MixedFamilyIsolationV1Config::from_toml_path(&example_fixture_path(name))
        .expect("parse sprint114 config");
    absolutize_paths(&mut config.sprint113_bundle_paths);
    absolutize_paths(&mut config.sprint113_truth_paths);
    absolutize_paths(&mut config.suspect_target_paths);
    absolutize_paths(&mut config.assertion_inventory_paths);
    absolutize_paths(&mut config.cargo_json_progress_paths);
    absolutize_paths(&mut config.workspace_timeout_paths);
    config.output_root = named_output_dir("sprint114-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint114(name: &str, output_name: &str) -> MixedFamilyIsolationV1Bundle {
    let config = sprint114_config_from_example(name, output_name);
    MixedFamilyIsolationV1Runner::default()
        .run(&config)
        .expect("run sprint114")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint114_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint114-tests", name)
}
