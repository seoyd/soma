mod common;

use std::path::PathBuf;

use soma_zero::{KRXOfficialEvidenceActivationConfig, KRXOfficialEvidenceActivationRunner};

#[test]
fn sprint49_activation_example_is_deterministic() {
    let mut config = KRXOfficialEvidenceActivationConfig::from_toml_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("soma_krx_official_activate_local_import.toml"),
    )
    .expect("parse config");
    config.output_root = common::output_dir("krx-determinism-example")
        .display()
        .to_string();
    let first = KRXOfficialEvidenceActivationRunner::default()
        .run(&config)
        .expect("first run");
    let second = KRXOfficialEvidenceActivationRunner::default()
        .run(&config)
        .expect("second run");
    assert_eq!(first, second);
}
