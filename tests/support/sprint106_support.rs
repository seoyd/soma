#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use soma_zero::{
    Sprint105VerificationPatchClosureBundle, WorkspaceAcceptanceRecoveryV7Bundle,
    WorkspaceAcceptanceRecoveryV7Config, WorkspaceAcceptanceRecoveryV7Runner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
    write_json_output, write_text_output,
};
use super::sprint105_support::run_sprint105;

pub fn sprint106_config_from_example(
    name: &str,
    output_name: &str,
) -> WorkspaceAcceptanceRecoveryV7Config {
    let mut config =
        WorkspaceAcceptanceRecoveryV7Config::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint106 config");
    for paths in [
        config.sprint105_bundle_paths.as_mut(),
        config.workspace_truth_paths.as_mut(),
        config.compile_cost_paths.as_mut(),
        config.cargo_json_paths.as_mut(),
        config.target_inventory_paths.as_mut(),
        config.test_manifest_paths.as_mut(),
    ] {
        if let Some(paths) = paths {
            for path in paths.iter_mut() {
                *path = absolutize_fixture_path(path).display().to_string();
            }
        }
    }
    config.output_root = named_output_dir("sprint106-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint106(name: &str, output_name: &str) -> WorkspaceAcceptanceRecoveryV7Bundle {
    let config = sprint106_config_from_example(name, output_name);
    WorkspaceAcceptanceRecoveryV7Runner::default()
        .run(&config)
        .expect("run sprint106")
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    write_json_output(
        named_output_dir("sprint106-tests", output_name),
        file_name,
        value,
    )
}

pub fn write_text(output_name: &str, file_name: &str, value: &str) -> String {
    write_text_output(
        named_output_dir("sprint106-tests", output_name),
        file_name,
        value,
    )
}

pub fn write_sprint105_bundle(
    output_name: &str,
    file_name: &str,
    bundle: &Sprint105VerificationPatchClosureBundle,
) -> String {
    write_support_json(output_name, file_name, bundle)
}

pub fn run_default_sprint105_fixture() -> Sprint105VerificationPatchClosureBundle {
    run_sprint105(
        "soma_sprint105_verification_patch_close.toml",
        "sprint106-generated-sprint105-fixture",
    )
}

pub fn sprint106_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint106-tests", name)
}

pub fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
