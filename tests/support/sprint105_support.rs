#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use soma_zero::{
    Sprint104DualAgentPaperLifecycleBundle, Sprint105VerificationPatchClosureBundle,
    Sprint105VerificationPatchClosureConfig, Sprint105VerificationPatchClosureRunner,
};

use super::shared_fixture_harness::{
    absolutize_fixture_path, example_fixture_path, load_json_fixture, named_output_dir,
    write_json_output, write_text_output,
};
use super::sprint104_support::run_sprint104;

pub fn sprint105_config_from_example(
    name: &str,
    output_name: &str,
) -> Sprint105VerificationPatchClosureConfig {
    let mut config =
        Sprint105VerificationPatchClosureConfig::from_toml_path(&example_fixture_path(name))
            .expect("parse sprint105 config");
    for paths in [
        config.sprint104_bundle_paths.as_mut(),
        config.verification_finding_paths.as_mut(),
        config.review_patch_summary_paths.as_mut(),
        config.paper_lifecycle_paths.as_mut(),
        config.risk_governor_batch_paths.as_mut(),
        config.lower_confidence_paths.as_mut(),
        config.workspace_truth_paths.as_mut(),
    ] {
        if let Some(paths) = paths {
            for path in paths.iter_mut() {
                *path = absolutize_fixture_path(path).display().to_string();
            }
        }
    }
    config.output_root = named_output_dir("sprint105-tests", output_name)
        .display()
        .to_string();
    config
}

pub fn run_sprint105(name: &str, output_name: &str) -> Sprint105VerificationPatchClosureBundle {
    let config = sprint105_config_from_example(name, output_name);
    Sprint105VerificationPatchClosureRunner::default()
        .run(&config)
        .expect("run sprint105")
}

pub fn sprint105_output_dir(name: &str) -> PathBuf {
    named_output_dir("sprint105-tests", name)
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    load_json_fixture(example_fixture_path(name))
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    write_json_output(
        named_output_dir("sprint105-tests", output_name),
        file_name,
        value,
    )
}

pub fn write_sprint104_bundle(
    output_name: &str,
    file_name: &str,
    bundle: &Sprint104DualAgentPaperLifecycleBundle,
) -> String {
    write_support_json(output_name, file_name, bundle)
}

pub fn run_default_sprint104_fixture() -> Sprint104DualAgentPaperLifecycleBundle {
    run_sprint104(
        "soma_sprint104_dual_agent_paper_lifecycle.toml",
        "sprint105-generated-sprint104-fixture",
    )
}

pub fn write_text(output_name: &str, file_name: &str, value: &str) -> String {
    write_text_output(
        named_output_dir("sprint105-tests", output_name),
        file_name,
        value,
    )
}

pub fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
