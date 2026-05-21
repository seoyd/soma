#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde::de::DeserializeOwned;
use soma_zero::{
    DualAgentWorkflowConfig, Sprint103PaperRotationClosureBundle,
    Sprint104DualAgentPaperLifecycleBundle, Sprint104DualAgentPaperLifecycleRunner,
};

use super::sprint69_support::{absolutize, example_path, output_dir, read_json};
use super::sprint103_support::run_sprint103;

pub fn sprint104_config_from_example(name: &str, output_name: &str) -> DualAgentWorkflowConfig {
    let mut config = DualAgentWorkflowConfig::from_toml_path(&example_path(name))
        .expect("parse sprint104 config");
    for paths in [
        config.sprint103_bundle_paths.as_mut(),
        config.implementation_report_paths.as_mut(),
        config.verification_report_paths.as_mut(),
        config.changed_file_manifest_paths.as_mut(),
        config.focused_test_report_paths.as_mut(),
        config.cli_smoke_report_paths.as_mut(),
        config.workspace_truth_paths.as_mut(),
    ] {
        if let Some(paths) = paths {
            for path in paths.iter_mut() {
                *path = absolutize(path).display().to_string();
            }
        }
    }
    config.output_root = output_dir(output_name).display().to_string();
    config
}

pub fn run_sprint104(name: &str, output_name: &str) -> Sprint104DualAgentPaperLifecycleBundle {
    let config = sprint104_config_from_example(name, output_name);
    Sprint104DualAgentPaperLifecycleRunner::default()
        .run(&config)
        .expect("run sprint104")
}

pub fn sprint104_output_dir(name: &str) -> PathBuf {
    output_dir(name)
}

pub fn read_fixture<T: DeserializeOwned>(name: &str) -> T {
    read_json(example_path(name))
}

pub fn write_support_json<T: Serialize>(output_name: &str, file_name: &str, value: &T) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    let text = serde_json::to_string_pretty(value).expect("serialize sprint104 json");
    fs::write(&path, text).expect("write sprint104 json");
    path.display().to_string()
}

pub fn write_manifest(output_name: &str, file_name: &str, entries: &[&str]) -> String {
    write_support_json(output_name, file_name, &entries)
}

pub fn write_sprint103_bundle(
    output_name: &str,
    file_name: &str,
    bundle: &Sprint103PaperRotationClosureBundle,
) -> String {
    write_support_json(output_name, file_name, bundle)
}

pub fn run_default_sprint103_fixture() -> Sprint103PaperRotationClosureBundle {
    run_sprint103(
        "soma_sprint103_paper_rotation_close.toml",
        "sprint104-generated-sprint103-fixture",
    )
}

pub fn write_text(output_name: &str, file_name: &str, value: &str) -> String {
    let dir = output_dir(output_name);
    let path = dir.join(file_name);
    fs::write(&path, value).expect("write sprint104 text");
    path.display().to_string()
}

pub fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}
