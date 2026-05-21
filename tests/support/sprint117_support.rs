#![allow(dead_code)]

use std::path::PathBuf;

use soma_zero::{
    DeferredRealObservationExecutionBundle, DeferredRealObservationExecutionConfig,
    DeferredRealObservationExecutionRunner,
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

pub fn sprint117_config_from_example(
    name: &str,
    output_name: &str,
) -> DeferredRealObservationExecutionConfig {
    let mut config =
        DeferredRealObservationExecutionConfig::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint117 config");
    absolutize_paths(&mut config.sprint116_bundle_paths);
    absolutize_paths(&mut config.sprint116_truth_paths);
    absolutize_paths(&mut config.observation_backlog_paths);
    absolutize_paths(&mut config.no_run_plan_paths);
    absolutize_paths(&mut config.full_plan_paths);
    absolutize_paths(&mut config.cargo_json_plan_paths);
    config.output_root = named_output_dir("sprint117-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint117(name: &str, output_name: &str) -> DeferredRealObservationExecutionBundle {
    let config = sprint117_config_from_example(name, output_name);
    DeferredRealObservationExecutionRunner::default()
        .run(&config)
        .expect("run sprint117")
}

pub fn read_fixture<T: serde::de::DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn sprint117_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint117-tests", name)
}
