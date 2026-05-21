use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use soma_zero::{Sprint14Runner, sprint14_report_to_text};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn sprint14_report_rendering_is_deterministic() {
    let ablation_path = repo_root()
        .join("target")
        .join("soma_ablations")
        .join("ablation_feature_lab")
        .join("ablation_report.json");
    if !ablation_path.exists() {
        let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
            .args([
                "ablation",
                "--config",
                "examples/soma_ablation_feature_lab.toml",
            ])
            .output()
            .expect("run ablation");
        assert!(output.status.success());
    }
    let report_a = Sprint14Runner::default()
        .run_from_ablation_report_path(&ablation_path)
        .expect("sprint14 report");
    let report_b = Sprint14Runner::default()
        .run_from_ablation_report_path(&ablation_path)
        .expect("sprint14 report");
    assert_eq!(
        sprint14_report_to_text(&report_a),
        sprint14_report_to_text(&report_b)
    );
}

#[test]
fn no_runtime_llm_path_exists() {
    let mut files = Vec::new();
    collect_rs_files(&repo_root().join("src"), &mut files);
    let combined = files
        .into_iter()
        .map(|path| fs::read_to_string(path).expect("source"))
        .collect::<Vec<_>>()
        .join("\n");
    for token in ["openai", "anthropic", "langchain", "llm_client"] {
        assert!(!combined.to_lowercase().contains(token));
    }
}

#[test]
fn no_live_network_api_path_exists() {
    let cargo_toml = fs::read_to_string(repo_root().join("Cargo.toml")).expect("cargo toml");
    for token in ["reqwest", "hyper", "tungstenite", "ureq"] {
        assert!(!cargo_toml.contains(token));
    }
}

#[test]
fn no_real_broker_or_order_execution_path_exists() {
    let output = Command::new(env!("CARGO_BIN_EXE_soma_experiment"))
        .arg("--help")
        .output()
        .expect("cli help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("\n  broker"));
    assert!(!stdout.contains("\n  live"));
    assert!(!stdout.contains("\n  execute"));
}

#[test]
fn no_new_persona_implementation_exists() {
    let league_mod = fs::read_to_string(repo_root().join("src/league/mod.rs")).expect("league mod");
    for token in [
        "darvas_box_breakout",
        "turtle_system_trend",
        "ptj_tactical_risk",
    ] {
        assert!(!league_mod.contains(token));
    }
}
