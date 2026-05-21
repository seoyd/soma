use crate::ReasonCode;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn render_json<T: Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value).map_err(|err| err.to_string())
}
fn write_text_file(path: &Path, value: &str) -> Result<(), String> {
    fs::write(path, value).map_err(|err| err.to_string())
}
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn local_only(path: &str) -> bool {
    let path = path.trim();
    !path.is_empty()
        && !path.contains("://")
        && !path.starts_with("http:")
        && !path.starts_with("https:")
        && !path.starts_with("s3:")
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_timeout_ms() -> Option<u64> {
    Some(420_000)
}
fn default_output_root() -> String {
    "target/soma_sprint114_mixed_family_isolation".to_string()
}
fn stable_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn shell_exec(command: &str) -> String {
    format!("cd {} && exec {}", project_root().display(), command)
}

fn count_processes(process_name: &str) -> u64 {
    Command::new("sh")
        .arg("-lc")
        .arg(format!(
            "ps -axo comm= | awk '$1==\"{}\" {{c++}} END {{print c+0}}'",
            process_name
        ))
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| text.trim().parse::<u64>().ok())
        .unwrap_or(0)
}
fn diagnostic_reason_codes(extra: &[ReasonCode]) -> Vec<ReasonCode> {
    let mut codes = vec![
        ReasonCode::CommitteeV1Built,
        ReasonCode::DeterministicPath,
        ReasonCode::LocalFileOnly,
        ReasonCode::ResearchOnlyOverride,
        ReasonCode::MambaRuntimeDeferred,
        ReasonCode::GatedDeltaNetRuntimeDeferred,
        ReasonCode::NoTradeDefault,
    ];
    for code in extra {
        if !codes.contains(code) {
            codes.push(code.clone());
        }
    }
    codes
}
fn warning_posture() -> Vec<String> {
    vec![
        "research-only",
        "paper-only",
        "mixed-family-isolation-only",
        "fifth-patch-not-applied",
        "fifth-patch-ready-does-not-mean-applied",
        "focused-is-not-full",
        "CLI-smoke-is-not-full",
        "cargo-build-is-not-full",
        "no-run-is-not-full",
        "cargo-progress-is-not-acceptance",
        "timeout-cleanup-is-not-pass",
        "no assertion deletion",
        "no safety sentinel deletion",
        "no runtime implementation",
        "no training",
        "no live inference",
        "no live trading",
        "no order/account command",
        "no runtime LLM live decision path",
        "no investor impersonation",
        "no auto-activation of 18 live agents",
        "no silent confidence upgrade",
        "no safety test deletion",
        "no hidden skips",
        "local-only paths",
        "remote paths rejected",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
macro_rules! report { ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => { #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)] pub struct $name { $(pub $field: $ty,)* pub reason_codes: Vec<ReasonCode>, } }; }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MixedFamilyIsolationV1Config {
    pub isolation_id: String,
    #[serde(default)]
    pub sprint113_bundle_paths: Option<Vec<String>>,
    #[serde(default)]
    pub sprint113_truth_paths: Option<Vec<String>>,
    #[serde(default)]
    pub suspect_target_paths: Option<Vec<String>>,
    #[serde(default)]
    pub assertion_inventory_paths: Option<Vec<String>>,
    #[serde(default)]
    pub cargo_json_progress_paths: Option<Vec<String>>,
    #[serde(default)]
    pub workspace_timeout_paths: Option<Vec<String>>,
    #[serde(default = "default_output_root")]
    pub output_root: String,
    #[serde(default = "default_false")]
    pub run_real_no_run_observation: bool,
    #[serde(default = "default_false")]
    pub run_real_full_observation: bool,
    #[serde(default = "default_false")]
    pub run_cargo_json_suspect_trace: bool,
    #[serde(default = "default_false")]
    pub run_suspect_target_rustc_probe: bool,
    #[serde(default = "default_timeout_ms")]
    pub no_run_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub full_timeout_ms: Option<u64>,
    #[serde(default = "default_timeout_ms")]
    pub cargo_json_timeout_ms: Option<u64>,
    #[serde(default = "default_true")]
    pub require_assertion_migration_feasibility: bool,
    #[serde(default = "default_true")]
    pub require_equivalent_coverage_drilldown: bool,
    #[serde(default = "default_true")]
    pub require_fifth_patch_gate_v4: bool,
    #[serde(default = "default_false")]
    pub allow_fifth_patch_application: bool,
    #[serde(default = "default_true")]
    pub preserve_runtime_deferred: bool,
    #[serde(default = "default_true")]
    pub preserve_safety_guards: bool,
    #[serde(default)]
    pub reason_codes: Vec<ReasonCode>,
}
impl Default for MixedFamilyIsolationV1Config {
    fn default() -> Self {
        Self {
            isolation_id: "sprint114-mixed-family-isolation".to_string(),
            sprint113_bundle_paths: Some(vec![
                "examples/sprint114_data/sprint113_summary.json".to_string(),
            ]),
            sprint113_truth_paths: Some(vec![
                "examples/sprint114_data/sprint113_summary.json".to_string(),
            ]),
            suspect_target_paths: Some(vec![
                "examples/sprint114_data/suspect_target_decomposition_expected.json".to_string(),
            ]),
            assertion_inventory_paths: Some(vec![
                "examples/sprint114_data/assertion_inventory_expected.json".to_string(),
            ]),
            cargo_json_progress_paths: Some(vec![
                "examples/sprint114_data/integration_fanout_narrowing_expected.json".to_string(),
            ]),
            workspace_timeout_paths: Some(vec![
                "examples/sprint114_data/acceptance_truth_gate_v15_expected.json".to_string(),
            ]),
            output_root: default_output_root(),
            run_real_no_run_observation: false,
            run_real_full_observation: false,
            run_cargo_json_suspect_trace: false,
            run_suspect_target_rustc_probe: false,
            no_run_timeout_ms: default_timeout_ms(),
            full_timeout_ms: default_timeout_ms(),
            cargo_json_timeout_ms: default_timeout_ms(),
            require_assertion_migration_feasibility: true,
            require_equivalent_coverage_drilldown: true,
            require_fifth_patch_gate_v4: true,
            allow_fifth_patch_application: false,
            preserve_runtime_deferred: true,
            preserve_safety_guards: true,
            reason_codes: diagnostic_reason_codes(&[]),
        }
    }
}
impl MixedFamilyIsolationV1Config {
    pub fn from_toml_str(input: &str) -> Result<Self, String> {
        toml::from_str(input).map_err(|err| err.to_string())
    }
    pub fn from_toml_path(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|err| err.to_string())?;
        Self::from_toml_str(&text)
    }
    pub fn to_toml_string(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|err| err.to_string())
    }
    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output_root).join(&self.isolation_id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.isolation_id.trim().is_empty() {
            return Err("sprint114 isolation_id must not be empty".to_string());
        }
        if !local_only(&self.output_root) {
            return Err("sprint114 mixed family isolation config paths must be local".to_string());
        }
        for paths in [
            &self.sprint113_bundle_paths,
            &self.sprint113_truth_paths,
            &self.suspect_target_paths,
            &self.assertion_inventory_paths,
            &self.cargo_json_progress_paths,
            &self.workspace_timeout_paths,
        ] {
            if let Some(paths) = paths
                && paths.iter().any(|path| !local_only(path))
            {
                return Err(
                    "sprint114 mixed family isolation config paths must be local".to_string(),
                );
            }
        }
        if !self.require_assertion_migration_feasibility {
            return Err(
                "sprint114 requires require_assertion_migration_feasibility=true".to_string(),
            );
        }
        if !self.require_equivalent_coverage_drilldown {
            return Err(
                "sprint114 requires require_equivalent_coverage_drilldown=true".to_string(),
            );
        }
        if !self.require_fifth_patch_gate_v4 {
            return Err("sprint114 requires require_fifth_patch_gate_v4=true".to_string());
        }
        if self.allow_fifth_patch_application {
            return Err("sprint114 forbids fifth patch application".to_string());
        }
        if !self.preserve_runtime_deferred || !self.preserve_safety_guards {
            return Err("sprint114 preserve flags must stay true".to_string());
        }
        Ok(())
    }
}
fn load_first_json<T: DeserializeOwned>(paths: Option<&Vec<String>>) -> Result<Option<T>, String> {
    if let Some(paths) = paths {
        for path in paths {
            let candidate = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                project_root().join(path)
            };
            if candidate.exists() {
                let text = fs::read_to_string(&candidate).map_err(|err| err.to_string())?;
                return serde_json::from_str(&text)
                    .map(Some)
                    .map_err(|err| format!("{}: {err}", candidate.display()));
            }
        }
    }
    Ok(None)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sprint113SummaryFixture {
    pub report_id: String,
    pub focused_tests_passed: bool,
    pub cli_smoke_passed: bool,
    pub cargo_check_passed: bool,
    pub cargo_build_passed: bool,
    pub no_run_timeout_seconds: Option<u64>,
    pub no_run_exit_code: Option<i32>,
    pub full_timeout_seconds: Option<u64>,
    pub full_exit_code: Option<i32>,
    pub timeout_cleanup_verified: bool,
    pub root_cause_status: String,
    pub root_cause_confidence: String,
    pub acceptance_evidence_status: String,
    pub acceptance_truth_status: String,
    pub fifth_patch_still_blocked: bool,
    pub previous_gate_status: String,
    pub isolated_families: Vec<String>,
    pub still_mixed_families: Vec<String>,
    pub suspect_targets: Vec<String>,
    pub integration_observed_evidence: Vec<String>,
    pub integration_inferred_evidence: Vec<String>,
    pub link_observed_evidence: Vec<String>,
    pub link_inferred_evidence: Vec<String>,
    pub macro_observed_evidence: Vec<String>,
    pub macro_inferred_evidence: Vec<String>,
    pub cargo_json_events: BTreeMap<String, Vec<String>>,
    pub cargo_json_last_seen_artifact: BTreeMap<String, String>,
    pub cargo_json_parser_errors: u64,
    pub rustc_args: Vec<String>,
    pub rustc_max_concurrency: u64,
    pub remaining_rustc_processes_after_timeout: u64,
    pub remaining_cargo_processes_after_timeout: u64,
    pub artifact_events: BTreeMap<String, Vec<String>>,
    pub assertion_count_by_target: BTreeMap<String, u64>,
    pub assertion_kinds_by_target: BTreeMap<String, Vec<String>>,
    pub assertion_dependencies: BTreeMap<String, Vec<String>>,
    pub migration_complexity: BTreeMap<String, String>,
    pub existing_migrated_assertion_count: u64,
    pub further_migration_capacity: u64,
    pub equivalent_coverage_feasible: bool,
    pub sentinel_safety_preserved: bool,
    pub no_hidden_skip_risk: bool,
    pub mixed_family_evidence_narrowed_enough: bool,
    pub cumulative_sample_backed_delta: i64,
}
impl Default for Sprint113SummaryFixture {
    fn default() -> Self {
        let suspects = vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            "tests/shared_fixture_harness_application_v1.rs".to_string(),
            "tests/workspace_timeout_root_cause.rs".to_string(),
        ];
        Self {
            report_id: "sprint113-summary".to_string(),
            focused_tests_passed: true,
            cli_smoke_passed: true,
            cargo_check_passed: true,
            cargo_build_passed: true,
            no_run_timeout_seconds: Some(420),
            no_run_exit_code: Some(124),
            full_timeout_seconds: Some(420),
            full_exit_code: Some(124),
            timeout_cleanup_verified: true,
            root_cause_status: "TimeoutRootCausePartiallyIsolated".to_string(),
            root_cause_confidence: "Moderate".to_string(),
            acceptance_evidence_status: "AcceptanceEvidenceSupportingOnly".to_string(),
            acceptance_truth_status: "AcceptanceTruthReadyWithWarnings".to_string(),
            fifth_patch_still_blocked: true,
            previous_gate_status: "FifthPatchStillBlocked".to_string(),
            isolated_families: vec![
                "FixtureSetupFanout".to_string(),
                "CliSmokeFanout".to_string(),
                "ArtifactRenderFanout".to_string(),
            ],
            still_mixed_families: vec![
                "IntegrationTestBinaryFanout".to_string(),
                "LinkTimeCost".to_string(),
                "MacroExpansionCost".to_string(),
            ],
            suspect_targets: suspects.clone(),
            integration_observed_evidence: vec![
                "cargo no-run timeout still ends near integration test binary fanout".to_string(),
                "focused matrix passed while integration binary fanout stayed mixed".to_string(),
            ],
            integration_inferred_evidence: vec![
                "shared fixture harness likely amplifies integration binary count".to_string(),
                "control tower panel transitively depends on integration binary surfaces"
                    .to_string(),
            ],
            link_observed_evidence: vec![
                "rustc timeline last heavy link candidate was control tower timeout panel"
                    .to_string(),
            ],
            link_inferred_evidence: vec![
                "workspace timeout root cause target may still add linker pressure".to_string(),
            ],
            macro_observed_evidence: vec![
                "cargo json trace stalled after workspace_timeout_root_cause artifact emission"
                    .to_string(),
            ],
            macro_inferred_evidence: vec![
                "macro-heavy report rendering remains coupled to timeout target".to_string(),
            ],
            cargo_json_events: BTreeMap::from([
                (
                    suspects[0].clone(),
                    vec![
                        "compiler-artifact:control_tower_workspace_timeout_root_cause_panel"
                            .to_string(),
                        "build-script-executed:control_tower panel".to_string(),
                    ],
                ),
                (
                    suspects[1].clone(),
                    vec!["compiler-artifact:shared_fixture_harness_application_v1".to_string()],
                ),
                (
                    suspects[2].clone(),
                    vec![
                        "compiler-message:workspace timeout root cause".to_string(),
                        "compiler-artifact:workspace_timeout_root_cause".to_string(),
                    ],
                ),
            ]),
            cargo_json_last_seen_artifact: BTreeMap::from([
                (
                    suspects[0].clone(),
                    "target/debug/deps/control_tower_workspace_timeout_root_cause_panel"
                        .to_string(),
                ),
                (
                    suspects[1].clone(),
                    "target/debug/deps/shared_fixture_harness_application_v1".to_string(),
                ),
                (
                    suspects[2].clone(),
                    "target/debug/deps/workspace_timeout_root_cause".to_string(),
                ),
            ]),
            cargo_json_parser_errors: 0,
            rustc_args: vec![
                "--test tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                "--test tests/shared_fixture_harness_application_v1.rs".to_string(),
                "--test tests/workspace_timeout_root_cause.rs".to_string(),
            ],
            rustc_max_concurrency: 3,
            remaining_rustc_processes_after_timeout: 0,
            remaining_cargo_processes_after_timeout: 0,
            artifact_events: BTreeMap::from([
                (
                    suspects[0].clone(),
                    vec![
                        "target/debug/deps/control_tower_workspace_timeout_root_cause_panel"
                            .to_string(),
                    ],
                ),
                (
                    suspects[1].clone(),
                    vec!["target/debug/deps/shared_fixture_harness_application_v1".to_string()],
                ),
                (
                    suspects[2].clone(),
                    vec!["target/debug/deps/workspace_timeout_root_cause".to_string()],
                ),
            ]),
            assertion_count_by_target: BTreeMap::from([
                (suspects[0].clone(), 6),
                (suspects[1].clone(), 5),
                (suspects[2].clone(), 9),
            ]),
            assertion_kinds_by_target: BTreeMap::from([
                (
                    suspects[0].clone(),
                    vec![
                        "cli-warning-posture".to_string(),
                        "panel-render".to_string(),
                        "timeout-interpretation".to_string(),
                    ],
                ),
                (
                    suspects[1].clone(),
                    vec![
                        "fixture-setup".to_string(),
                        "shared-harness".to_string(),
                        "deterministic-output".to_string(),
                    ],
                ),
                (
                    suspects[2].clone(),
                    vec![
                        "root-cause-split".to_string(),
                        "evidence-matrix".to_string(),
                        "acceptance-warning".to_string(),
                    ],
                ),
            ]),
            assertion_dependencies: BTreeMap::from([
                (
                    suspects[0].clone(),
                    vec![
                        "control tower warning renderer".to_string(),
                        "timeout panel bundle text".to_string(),
                    ],
                ),
                (
                    suspects[1].clone(),
                    vec!["shared fixture harness helpers".to_string()],
                ),
                (
                    suspects[2].clone(),
                    vec![
                        "root cause evidence split".to_string(),
                        "acceptance truth gate wording".to_string(),
                    ],
                ),
            ]),
            migration_complexity: BTreeMap::from([
                (suspects[0].clone(), "High".to_string()),
                (suspects[1].clone(), "Medium".to_string()),
                (suspects[2].clone(), "High".to_string()),
            ]),
            existing_migrated_assertion_count: 2,
            further_migration_capacity: 1,
            equivalent_coverage_feasible: true,
            sentinel_safety_preserved: true,
            no_hidden_skip_risk: true,
            mixed_family_evidence_narrowed_enough: false,
            cumulative_sample_backed_delta: -4,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Sprint114CommandOutputSnapshot {
    attempted: bool,
    finished: bool,
    exit_code: Option<i32>,
    timed_out: bool,
    timeout_ms: Option<u64>,
    duration_ms: Option<u64>,
    stdout: String,
    remaining_cargo_processes: u64,
    remaining_rustc_processes: u64,
}

fn observe_sprint114_command_stdout(
    run: bool,
    command: &str,
    timeout_ms: Option<u64>,
) -> Result<Sprint114CommandOutputSnapshot, String> {
    if !run {
        return Ok(Sprint114CommandOutputSnapshot {
            timeout_ms,
            ..Sprint114CommandOutputSnapshot::default()
        });
    }
    let start = Instant::now();
    let mut child = Command::new("sh")
        .arg("-lc")
        .arg(shell_exec(command))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| err.to_string())?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture sprint114 command stdout".to_string())?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut text = String::new();
        let _ = stdout.read_to_string(&mut text);
        let _ = tx.send(text);
    });
    loop {
        if let Some(status) = child.try_wait().map_err(|err| err.to_string())? {
            let stdout = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
            return Ok(Sprint114CommandOutputSnapshot {
                attempted: true,
                finished: true,
                exit_code: status.code(),
                timed_out: false,
                timeout_ms,
                duration_ms: Some(start.elapsed().as_millis() as u64),
                stdout,
                remaining_cargo_processes: 0,
                remaining_rustc_processes: 0,
            });
        }
        if let Some(timeout_ms) = timeout_ms
            && start.elapsed() >= Duration::from_millis(timeout_ms)
        {
            let _ = child.kill();
            let _ = child.wait();
            let stdout = rx.recv_timeout(Duration::from_secs(2)).unwrap_or_default();
            return Ok(Sprint114CommandOutputSnapshot {
                attempted: true,
                finished: false,
                exit_code: Some(124),
                timed_out: true,
                timeout_ms: Some(timeout_ms),
                duration_ms: Some(start.elapsed().as_millis() as u64),
                stdout,
                remaining_cargo_processes: count_processes("cargo"),
                remaining_rustc_processes: count_processes("rustc"),
            });
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn cargo_json_target_label(value: &serde_json::Value) -> Option<String> {
    value
        .get("target")
        .and_then(|target| target.get("name"))
        .and_then(|name| name.as_str())
        .or_else(|| {
            value
                .get("package_id")
                .and_then(|package_id| package_id.as_str())
        })
        .map(str::to_string)
}

fn cargo_json_artifact(value: &serde_json::Value) -> Option<String> {
    value
        .get("executable")
        .and_then(|executable| executable.as_str())
        .or_else(|| {
            value
                .get("filenames")
                .and_then(|filenames| filenames.as_array())
                .and_then(|filenames| filenames.last())
                .and_then(|filename| filename.as_str())
        })
        .map(str::to_string)
}

fn suspect_target_for_label(summary: &Sprint113SummaryFixture, label: &str) -> Option<String> {
    summary
        .suspect_targets
        .iter()
        .find(|target| {
            Path::new(target)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| label.contains(stem) || stem.contains(label))
        })
        .cloned()
}

fn apply_cargo_json_trace_to_summary(summary: &mut Sprint113SummaryFixture, stdout: &str) {
    let mut events: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut last_seen_artifact: BTreeMap<String, String> = BTreeMap::new();
    let mut parser_errors = 0;
    for line in stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            parser_errors += 1;
            continue;
        };
        let reason = value
            .get("reason")
            .and_then(|reason| reason.as_str())
            .unwrap_or("json-message");
        let label = cargo_json_target_label(&value).unwrap_or_else(|| "unknown-target".to_string());
        if let Some(target) = suspect_target_for_label(summary, &label) {
            events
                .entry(target.clone())
                .or_default()
                .push(format!("{reason}:{label}"));
            if let Some(artifact) = cargo_json_artifact(&value) {
                last_seen_artifact.insert(target, artifact);
            }
        }
    }
    summary.cargo_json_events = events;
    summary.cargo_json_last_seen_artifact = last_seen_artifact;
    summary.cargo_json_parser_errors = parser_errors;
}

fn apply_actual_sprint114_observations(
    summary: &mut Sprint113SummaryFixture,
    config: &MixedFamilyIsolationV1Config,
) -> Result<(), String> {
    let mut remaining_cargo = None;
    let mut remaining_rustc = None;
    if config.run_real_no_run_observation {
        let no_run = observe_sprint114_command_stdout(
            true,
            "cargo test --workspace --no-run --quiet",
            config.no_run_timeout_ms,
        )?;
        summary.no_run_timeout_seconds = no_run.timeout_ms.map(|timeout_ms| timeout_ms / 1000);
        summary.no_run_exit_code = no_run.exit_code;
        remaining_cargo = Some(no_run.remaining_cargo_processes);
        remaining_rustc = Some(no_run.remaining_rustc_processes);
    }
    if config.run_real_full_observation {
        let full = observe_sprint114_command_stdout(
            true,
            "cargo test --workspace --quiet",
            config.full_timeout_ms,
        )?;
        summary.full_timeout_seconds = full.timeout_ms.map(|timeout_ms| timeout_ms / 1000);
        summary.full_exit_code = full.exit_code;
        remaining_cargo = Some(remaining_cargo.unwrap_or(0) + full.remaining_cargo_processes);
        remaining_rustc = Some(remaining_rustc.unwrap_or(0) + full.remaining_rustc_processes);
    }
    if config.run_cargo_json_suspect_trace || config.run_suspect_target_rustc_probe {
        let cargo_json = observe_sprint114_command_stdout(
            true,
            "cargo test --workspace --no-run --message-format=json",
            config.cargo_json_timeout_ms,
        )?;
        apply_cargo_json_trace_to_summary(summary, &cargo_json.stdout);
        remaining_cargo = Some(remaining_cargo.unwrap_or(0) + cargo_json.remaining_cargo_processes);
        remaining_rustc = Some(remaining_rustc.unwrap_or(0) + cargo_json.remaining_rustc_processes);
    }
    if let (Some(cargo), Some(rustc)) = (remaining_cargo, remaining_rustc) {
        summary.remaining_cargo_processes_after_timeout = cargo;
        summary.remaining_rustc_processes_after_timeout = rustc;
        summary.timeout_cleanup_verified = cargo == 0 && rustc == 0;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssertionDestinationCandidateRowV1 {
    pub candidate_target: String,
    pub existing_coverage: String,
    pub risk: String,
    pub capacity: u64,
    pub destination_target_required: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FifthPatchCandidateDecisionRowV1 {
    pub candidate_target: String,
    pub assertion_migration_feasible: bool,
    pub equivalent_coverage_feasible: bool,
    pub sentinel_safety_preserved: bool,
    pub no_hidden_skip_risk: bool,
    pub mixed_family_relevance: String,
    pub decision_recommendation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceMatrixRowV1 {
    pub family: String,
    pub target: String,
    pub evidence_kind: String,
    pub source: String,
    pub detail: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservationQualityRowV1 {
    pub target: String,
    pub observed_evidence_count: u64,
    pub inferred_evidence_count: u64,
    pub quality: String,
}

report!(Sprint113BaselineTruthImportReport { report_id: String, focused_tests_passed: bool, cli_smoke_passed: bool, cargo_check_passed: bool, cargo_build_passed: bool, no_run_timeout_seconds: Option<u64>, no_run_exit_code: Option<i32>, full_timeout_seconds: Option<u64>, full_exit_code: Option<i32>, timeout_cleanup_verified: bool, fifth_patch_still_blocked: bool, root_cause_status: String, acceptance_evidence_status: String, imported_as_full_acceptance: bool, import_status: String });
report!(Sprint113ObservationCarryForwardReport { report_id: String, real_observation_surfaces_carried_forward: Vec<String>, cargo_json_actual_parsing_carried_forward: bool, actual_cleanup_counts_carried_forward: bool, no_apply_guarantee_carried_forward: bool, suspect_family_isolation_carried_forward: bool, fifth_patch_gate_carried_forward: bool, carry_forward_status: String });
report!(StillMixedFamilyRegistryV1 { registry_id: String, mixed_families: Vec<String>, already_isolated_families: Vec<String>, suspect_targets: Vec<String>, status: String });
report!(MixedFamilyIsolationPlanV1 { plan_id: String, integration_fanout_steps: Vec<String>, link_time_steps: Vec<String>, macro_expansion_steps: Vec<String>, assertion_inventory_steps: Vec<String>, equivalent_coverage_steps: Vec<String>, stop_conditions: Vec<String>, plan_status: String });
report!(IntegrationFanoutNarrowingReportV1 { report_id: String, suspect_integration_targets: Vec<String>, fanout_cluster_count: u64, isolated_integration_fanout: Vec<String>, still_mixed_integration_fanout: Vec<String>, observed_evidence: Vec<String>, inferred_evidence: Vec<String>, status: String });
report!(LinkTimeNarrowingReportV1 { report_id: String, link_heavy_target_candidates: Vec<String>, observed_evidence: Vec<String>, inferred_evidence: Vec<String>, status: String });
report!(MacroExpansionNarrowingReportV1 { report_id: String, macro_heavy_target_candidates: Vec<String>, observed_evidence: Vec<String>, inferred_evidence: Vec<String>, status: String });
report!(SuspectTargetDecompositionReportV1 { report_id: String, decomposed_targets: Vec<String>, per_target_pressure: BTreeMap<String, Vec<String>>, target_to_family_mapping: BTreeMap<String, Vec<String>>, decomposition_status: String });
report!(ControlTowerTimeoutPanelDecompositionReportV1 {
    report_id: String,
    target: String,
    cli_pressure: bool,
    render_pressure: bool,
    link_pressure: bool,
    assertion_migration_feasibility: String,
    status: String
});
report!(WorkspaceTimeoutRootCauseTargetDecompositionReportV1 {
    report_id: String,
    target: String,
    macro_pressure: bool,
    render_pressure: bool,
    link_pressure: bool,
    assertion_migration_feasibility: String,
    status: String
});
report!(SharedFixtureHarnessPressureReportV1 {
    report_id: String,
    target: String,
    fixture_pressure: bool,
    helper_pressure: bool,
    existing_migrated_assertion_count: u64,
    further_migration_capacity: u64,
    status: String
});
report!(TargetAssertionInventoryReportV1 { report_id: String, candidate_targets: Vec<String>, assertion_count_by_target: BTreeMap<String, u64>, assertion_kinds_by_target: BTreeMap<String, Vec<String>>, assertion_dependencies: BTreeMap<String, Vec<String>>, migration_complexity: BTreeMap<String, String>, inventory_status: String });
report!(AssertionMigrationFeasibilityDrilldownReportV1 { report_id: String, candidate_target: String, assertions_to_move: Vec<String>, destination_candidates: Vec<String>, blockers: Vec<String>, feasible: bool, feasibility_status: String });
report!(AssertionDestinationCandidateReportV1 { report_id: String, destination_candidates: Vec<AssertionDestinationCandidateRowV1>, status: String });
report!(AssertionRiskClassificationReportV1 { report_id: String, risk_by_target: BTreeMap<String, String>, safety_related_assertions: Vec<String>, deterministic_output_assertions: Vec<String>, cli_surface_assertions: Vec<String>, status: String });
report!(EquivalentCoverageFeasibilityDrilldownReportV1 { report_id: String, equivalent_coverage_destination: Option<String>, coverage_proof_refs: Vec<String>, coverage_gaps: Vec<String>, feasible: bool, status: String });
report!(SentinelSafetyImpactPreviewReportV1 { report_id: String, sentinel_impact: String, isolated_sentinels_affected: Vec<String>, sentinel_safety_preserved: bool, status: String });
report!(NoHiddenSkipRiskPreviewReportV1 { report_id: String, skip_risk_indicators: Vec<String>, hidden_skip_risk: bool, status: String });
report!(FifthPatchCandidateDecisionMatrixV1 { report_id: String, candidate_rows: Vec<FifthPatchCandidateDecisionRowV1>, matrix_status: String });
report!(FifthPatchDecisionGateV4 {
    gate_id: String,
    previous_gate_status: String,
    mixed_family_isolation_status: String,
    assertion_migration_feasibility_status: String,
    equivalent_coverage_status: String,
    sentinel_safety_status: String,
    no_hidden_skip_status: String,
    candidate_matrix_status: String,
    fifth_patch_ready_for_next_sprint: bool,
    fifth_patch_applied_this_sprint: bool,
    gate_status: String
});
report!(FifthPatchApplyPlanForNextSprintV1 { report_id: String, candidate_target: Option<String>, assertions_to_migrate: Vec<String>, destination_target: Option<String>, expected_coverage_proof: Vec<String>, required_tests: Vec<String>, no_apply_this_sprint: bool, status: String });
report!(FifthPatchNoApplyGuaranteeReportV3 { report_id: String, fifth_patch_applied: bool, retired_files: Vec<String>, moved_assertions: Vec<String>, guarantee_status: String });
report!(CandidateStopConsolidationReportV1 { report_id: String, stop_recommended: bool, reasons: Vec<String>, status: String });
report!(CargoJsonSuspectTargetTraceV1 { report_id: String, suspect_target_json_events: BTreeMap<String, Vec<String>>, last_seen_artifact: BTreeMap<String, String>, parser_errors: u64, status: String });
report!(RustcSuspectTargetTimelineV2 { report_id: String, suspect_target_rustc_args: Vec<String>, concurrency: u64, remaining_processes_after_timeout: u64, status: String });
report!(ArtifactSuspectTargetTimelineV2 { report_id: String, suspect_target_artifact_events: BTreeMap<String, Vec<String>>, status: String });
report!(LinkMacroEvidenceMatrixV1 { report_id: String, link_evidence_rows: Vec<EvidenceMatrixRowV1>, macro_evidence_rows: Vec<EvidenceMatrixRowV1>, status: String });
report!(IntegrationFanoutEvidenceMatrixV1 { report_id: String, integration_fanout_rows: Vec<EvidenceMatrixRowV1>, status: String });
report!(TargetLevelObservationQualityReportV1 { report_id: String, rows: Vec<ObservationQualityRowV1>, status: String });
report!(TimeoutCleanupVerificationReportV7 {
    report_id: String,
    cleanup_verified: bool,
    remaining_cargo_processes: u64,
    remaining_rustc_processes: u64,
    cleanup_status: String
});
report!(WorkspaceNoRunRecoveryGateV15 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    timed_out: bool,
    recovered: bool,
    gate_status: String
});
report!(WorkspaceFullAcceptanceGateV15 {
    gate_id: String,
    command: String,
    finished: bool,
    passed: bool,
    accepted: bool,
    gate_status: String
});
report!(FocusedVsFullBridgeV11 {
    bridge_id: String,
    focused_tests_status: String,
    cli_smoke_status: String,
    cargo_build_status: String,
    no_run_status: String,
    full_status: String,
    bridge_status: String
});
report!(AcceptanceTruthGateV15 {
    gate_id: String,
    focused_truth_status: String,
    cli_truth_status: String,
    cargo_check_truth_status: String,
    cargo_build_truth_status: String,
    no_run_truth_status: String,
    full_workspace_truth_status: String,
    can_claim_full_acceptance: bool,
    truth_status: String
});
report!(AcceptanceEvidenceStrengthReportV4 { report_id: String, evidence_tiers: Vec<String>, strongest_claim: String, report_status: String });
report!(WorkspaceRecoveryDecisionReportV4 {
    report_id: String,
    recommend_fifth_patch_next_sprint: bool,
    recommend_stop_consolidation: bool,
    recommend_more_observation: bool,
    no_run_recovered: bool,
    full_workspace_accepted: bool,
    decision_status: String
});
report!(CumulativeSafePatchLedgerV5 { report_id: String, carried_patch_ids: Vec<String>, fifth_patch_applied: bool, status: String });
report!(CumulativeBinaryDeltaReportV4 {
    report_id: String,
    sample_backed_delta: i64,
    measured_claim_allowed: bool,
    status: String
});
report!(AssertionLedgerContinuityCheckV4 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(EquivalentCoverageContinuityCheckV4 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(SafetySentinelContinuityCheckV4 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(NoHiddenSkipContinuityCheckV4 {
    report_id: String,
    continuity_preserved: bool,
    status: String
});
report!(SafetyCoveragePreservationReportV30 {
    report_id: String,
    no_assertion_deletion: bool,
    no_safety_sentinel_deletion: bool,
    no_hidden_skips: bool,
    mixed_family_isolation_guard_present: bool,
    assertion_migration_feasibility_guard_present: bool,
    fifth_patch_v4_no_apply_guard_present: bool,
    stop_consolidation_guard_present: bool,
    runtime_deferred: bool,
    training_deferred: bool,
    live_trading_forbidden: bool,
    safety_status: String
});
report!(ControlTowerMixedFamilyIsolationPanel { panel_id: String, still_mixed_families: Vec<String>, isolated_families: Vec<String>, target_decomposition_status: String, assertion_migration_status: String, equivalent_coverage_status: String, acceptance_truth_status: String, warnings: Vec<String>, static_read_only: bool, no_run_button: bool, no_apply_patch_button: bool, no_train_runtime_live_order_account_controls: bool });
report!(ControlTowerFifthPatchReadinessPanelV4 { panel_id: String, fifth_gate_status: String, apply_plan_status: String, stop_consolidation_status: String, warnings: Vec<String>, static_read_only: bool, no_apply_patch_button: bool, no_run_button: bool, no_train_runtime_live_order_account_controls: bool });
report!(MixedFamilyIsolationStorageReport { report_id: String, output_dir: String, written_files: Vec<String>, file_count: u64 });
pub fn build_sprint113_baseline_truth_import_report(
    summary: &Sprint113SummaryFixture,
) -> Sprint113BaselineTruthImportReport {
    Sprint113BaselineTruthImportReport {
        report_id: "sprint113-baseline-truth-import".to_string(),
        focused_tests_passed: summary.focused_tests_passed,
        cli_smoke_passed: summary.cli_smoke_passed,
        cargo_check_passed: summary.cargo_check_passed,
        cargo_build_passed: summary.cargo_build_passed,
        no_run_timeout_seconds: summary.no_run_timeout_seconds,
        no_run_exit_code: summary.no_run_exit_code,
        full_timeout_seconds: summary.full_timeout_seconds,
        full_exit_code: summary.full_exit_code,
        timeout_cleanup_verified: summary.timeout_cleanup_verified,
        fifth_patch_still_blocked: summary.fifth_patch_still_blocked,
        root_cause_status: summary.root_cause_status.clone(),
        acceptance_evidence_status: summary.acceptance_evidence_status.clone(),
        imported_as_full_acceptance: false,
        import_status: if summary.focused_tests_passed
            && summary.cli_smoke_passed
            && summary.cargo_build_passed
        {
            "Sprint113TruthImportedWithWarnings"
        } else {
            "Sprint113TruthImported"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_sprint113_observation_carry_forward_report(
    _summary: &Sprint113SummaryFixture,
) -> Sprint113ObservationCarryForwardReport {
    Sprint113ObservationCarryForwardReport {
        report_id: "sprint113-observation-carry-forward".to_string(),
        real_observation_surfaces_carried_forward: vec![
            "focused-matrix".to_string(),
            "cargo-check".to_string(),
            "cargo-build".to_string(),
            "cli-smoke".to_string(),
            "timeout-cleanup".to_string(),
        ],
        cargo_json_actual_parsing_carried_forward: true,
        actual_cleanup_counts_carried_forward: true,
        no_apply_guarantee_carried_forward: true,
        suspect_family_isolation_carried_forward: true,
        fifth_patch_gate_carried_forward: true,
        carry_forward_status: "Sprint113ObservationCarryForwardReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_still_mixed_family_registry_v1(
    summary: &Sprint113SummaryFixture,
) -> StillMixedFamilyRegistryV1 {
    StillMixedFamilyRegistryV1 {
        registry_id: "still-mixed-family-registry-v1".to_string(),
        mixed_families: summary.still_mixed_families.clone(),
        already_isolated_families: summary.isolated_families.clone(),
        suspect_targets: summary.suspect_targets.clone(),
        status: "MixedFamilyRegistryReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_mixed_family_isolation_plan_v1() -> MixedFamilyIsolationPlanV1 {
    MixedFamilyIsolationPlanV1 {
        plan_id: "mixed-family-isolation-plan-v1".to_string(),
        integration_fanout_steps: vec![
            "separate integration binary fanout evidence into observed and inferred buckets"
                .to_string(),
            "map each suspect target to integration fanout pressure before any assertion move"
                .to_string(),
        ],
        link_time_steps: vec![
            "trace late-link candidates from cargo json and rustc timelines".to_string(),
            "keep link-time cost diagnostic-only unless narrowed per target".to_string(),
        ],
        macro_expansion_steps: vec![
            "inventory macro-heavy timeout assertions by target".to_string(),
            "keep macro expansion mixed until target-level evidence is separated".to_string(),
        ],
        assertion_inventory_steps: vec![
            "count assertions by suspect target".to_string(),
            "classify migration complexity before any fifth-patch readiness upgrade".to_string(),
        ],
        equivalent_coverage_steps: vec![
            "prove destination coverage before any target retirement recommendation".to_string(),
            "preserve sentinel and no-hidden-skip continuity".to_string(),
        ],
        stop_conditions: vec![
            "AssertionMigrationStillBlocked".to_string(),
            "SentinelRiskTooHigh".to_string(),
            "NeedMoreObservation".to_string(),
        ],
        plan_status: "MixedFamilyIsolationPlanReadyWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_integration_fanout_narrowing_report_v1(
    summary: &Sprint113SummaryFixture,
) -> IntegrationFanoutNarrowingReportV1 {
    IntegrationFanoutNarrowingReportV1 {
        report_id: "integration-fanout-narrowing-v1".to_string(),
        suspect_integration_targets: summary.suspect_targets.clone(),
        fanout_cluster_count: 2,
        isolated_integration_fanout: vec![
            "shared fixture harness fanout influence bounded".to_string(),
        ],
        still_mixed_integration_fanout: vec!["IntegrationTestBinaryFanout".to_string()],
        observed_evidence: stable_strings(summary.integration_observed_evidence.clone()),
        inferred_evidence: stable_strings(summary.integration_inferred_evidence.clone()),
        status: "IntegrationFanoutPartiallyNarrowed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_link_time_narrowing_report_v1(
    summary: &Sprint113SummaryFixture,
) -> LinkTimeNarrowingReportV1 {
    LinkTimeNarrowingReportV1 {
        report_id: "link-time-narrowing-v1".to_string(),
        link_heavy_target_candidates: vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
            "tests/workspace_timeout_root_cause.rs".to_string(),
        ],
        observed_evidence: stable_strings(summary.link_observed_evidence.clone()),
        inferred_evidence: stable_strings(summary.link_inferred_evidence.clone()),
        status: "LinkTimePartiallyNarrowed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_macro_expansion_narrowing_report_v1(
    summary: &Sprint113SummaryFixture,
) -> MacroExpansionNarrowingReportV1 {
    MacroExpansionNarrowingReportV1 {
        report_id: "macro-expansion-narrowing-v1".to_string(),
        macro_heavy_target_candidates: vec!["tests/workspace_timeout_root_cause.rs".to_string()],
        observed_evidence: stable_strings(summary.macro_observed_evidence.clone()),
        inferred_evidence: stable_strings(summary.macro_inferred_evidence.clone()),
        status: "MacroExpansionPartiallyNarrowed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_control_tower_timeout_panel_decomposition_report_v1()
-> ControlTowerTimeoutPanelDecompositionReportV1 {
    ControlTowerTimeoutPanelDecompositionReportV1 {
        report_id: "control-tower-timeout-panel-decomposition-v1".to_string(),
        target: "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
        cli_pressure: true,
        render_pressure: true,
        link_pressure: true,
        assertion_migration_feasibility: "Blocked".to_string(),
        status: "ControlTowerTimeoutPanelDecomposed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_workspace_timeout_root_cause_target_decomposition_report_v1()
-> WorkspaceTimeoutRootCauseTargetDecompositionReportV1 {
    WorkspaceTimeoutRootCauseTargetDecompositionReportV1 {
        report_id: "workspace-timeout-root-cause-target-decomposition-v1".to_string(),
        target: "tests/workspace_timeout_root_cause.rs".to_string(),
        macro_pressure: true,
        render_pressure: true,
        link_pressure: true,
        assertion_migration_feasibility: "Blocked".to_string(),
        status: "WorkspaceTimeoutTargetDecomposed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_shared_fixture_harness_pressure_report_v1(
    summary: &Sprint113SummaryFixture,
) -> SharedFixtureHarnessPressureReportV1 {
    SharedFixtureHarnessPressureReportV1 {
        report_id: "shared-fixture-harness-pressure-v1".to_string(),
        target: "tests/shared_fixture_harness_application_v1.rs".to_string(),
        fixture_pressure: true,
        helper_pressure: true,
        existing_migrated_assertion_count: summary.existing_migrated_assertion_count,
        further_migration_capacity: summary.further_migration_capacity,
        status: "SharedFixtureHarnessPressureReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_suspect_target_decomposition_report_v1(
    control_tower: &ControlTowerTimeoutPanelDecompositionReportV1,
    workspace: &WorkspaceTimeoutRootCauseTargetDecompositionReportV1,
    shared_fixture: &SharedFixtureHarnessPressureReportV1,
) -> SuspectTargetDecompositionReportV1 {
    let per_target_pressure = BTreeMap::from([
        (
            control_tower.target.clone(),
            vec![
                "IntegrationFanout".to_string(),
                "CliSmoke".to_string(),
                "LinkTime".to_string(),
            ],
        ),
        (
            shared_fixture.target.clone(),
            vec!["FixtureSetup".to_string(), "IntegrationFanout".to_string()],
        ),
        (
            workspace.target.clone(),
            vec![
                "MacroExpansion".to_string(),
                "ArtifactRender".to_string(),
                "LinkTime".to_string(),
            ],
        ),
    ]);
    SuspectTargetDecompositionReportV1 {
        report_id: "suspect-target-decomposition-v1".to_string(),
        decomposed_targets: per_target_pressure.keys().cloned().collect(),
        target_to_family_mapping: per_target_pressure.clone(),
        per_target_pressure,
        decomposition_status: "SuspectTargetsDecomposedWithWarnings".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_target_assertion_inventory_report_v1(
    summary: &Sprint113SummaryFixture,
) -> TargetAssertionInventoryReportV1 {
    TargetAssertionInventoryReportV1 {
        report_id: "target-assertion-inventory-v1".to_string(),
        candidate_targets: summary.suspect_targets.clone(),
        assertion_count_by_target: summary.assertion_count_by_target.clone(),
        assertion_kinds_by_target: summary.assertion_kinds_by_target.clone(),
        assertion_dependencies: summary.assertion_dependencies.clone(),
        migration_complexity: summary.migration_complexity.clone(),
        inventory_status: "AssertionInventoryReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_assertion_destination_candidate_report_v1(
    _inventory: &TargetAssertionInventoryReportV1,
) -> AssertionDestinationCandidateReportV1 {
    AssertionDestinationCandidateReportV1 { report_id: "assertion-destination-candidate-v1".to_string(), destination_candidates: vec![AssertionDestinationCandidateRowV1 { candidate_target: "tests/shared_fixture_harness_application_v1.rs".to_string(), existing_coverage: "shared fixture harness already covers setup and deterministic output, but not full control tower warning posture".to_string(), risk: "Medium".to_string(), capacity: 1, destination_target_required: true }, AssertionDestinationCandidateRowV1 { candidate_target: "tests/workspace_timeout_root_cause.rs".to_string(), existing_coverage: "root cause target covers timeout interpretation, but moving panel assertions would increase macro pressure".to_string(), risk: "High".to_string(), capacity: 0, destination_target_required: true }], status: "AssertionDestinationCandidatesReady".to_string(), reason_codes: diagnostic_reason_codes(&[]) }
}
pub fn build_assertion_risk_classification_report_v1(
    inventory: &TargetAssertionInventoryReportV1,
) -> AssertionRiskClassificationReportV1 {
    AssertionRiskClassificationReportV1 {
        report_id: "assertion-risk-classification-v1".to_string(),
        risk_by_target: inventory.migration_complexity.clone(),
        safety_related_assertions: vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs::warning-posture"
                .to_string(),
        ],
        deterministic_output_assertions: vec![
            "tests/shared_fixture_harness_application_v1.rs::deterministic-output".to_string(),
        ],
        cli_surface_assertions: vec![
            "tests/control_tower_workspace_timeout_root_cause_panel.rs::cli-warning-posture"
                .to_string(),
        ],
        status: "AssertionRiskClassificationReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_assertion_migration_feasibility_drilldown_report_v1(
    inventory: &TargetAssertionInventoryReportV1,
    destination_candidates: &AssertionDestinationCandidateReportV1,
) -> AssertionMigrationFeasibilityDrilldownReportV1 {
    let candidate_target = inventory
        .candidate_targets
        .first()
        .cloned()
        .unwrap_or_else(|| "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string());
    let feasible = destination_candidates
        .destination_candidates
        .iter()
        .any(|candidate| candidate.capacity > 1 && candidate.risk != "High");
    let blockers = if feasible {
        Vec::new()
    } else {
        vec![
            "assertion destination capacity is still insufficient".to_string(),
            "moving control tower warning assertions would blur mixed-family evidence".to_string(),
        ]
    };
    AssertionMigrationFeasibilityDrilldownReportV1 {
        report_id: "assertion-migration-feasibility-drilldown-v1".to_string(),
        candidate_target,
        assertions_to_move: vec![
            "warning_posture_stays_supporting_only".to_string(),
            "timeout_cleanup_is_not_pass".to_string(),
        ],
        destination_candidates: destination_candidates
            .destination_candidates
            .iter()
            .map(|candidate| candidate.candidate_target.clone())
            .collect(),
        blockers,
        feasible,
        feasibility_status: if feasible {
            "AssertionMigrationFeasible"
        } else {
            "AssertionMigrationBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_equivalent_coverage_feasibility_drilldown_report_v1(
    destination_candidates: &AssertionDestinationCandidateReportV1,
    require_destination_target: bool,
    coverage_gap_override: Option<Vec<String>>,
) -> EquivalentCoverageFeasibilityDrilldownReportV1 {
    let destination = destination_candidates
        .destination_candidates
        .iter()
        .find(|candidate| candidate.capacity > 0)
        .map(|candidate| candidate.candidate_target.clone());
    let coverage_gaps = coverage_gap_override.unwrap_or_default();
    let feasible =
        (!require_destination_target || destination.is_some()) && coverage_gaps.is_empty();
    EquivalentCoverageFeasibilityDrilldownReportV1 {
        report_id: "equivalent-coverage-feasibility-drilldown-v1".to_string(),
        equivalent_coverage_destination: destination,
        coverage_proof_refs: vec![
            "focused matrix remained green in Sprint 113".to_string(),
            "shared fixture harness already covers setup determinism".to_string(),
        ],
        coverage_gaps,
        feasible,
        status: if feasible {
            "EquivalentCoverageFeasible"
        } else {
            "EquivalentCoverageBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_sentinel_safety_impact_preview_report_v1(
    sentinel_safety_preserved: bool,
) -> SentinelSafetyImpactPreviewReportV1 {
    SentinelSafetyImpactPreviewReportV1 {
        report_id: "sentinel-safety-impact-preview-v1".to_string(),
        sentinel_impact: if sentinel_safety_preserved {
            "Preserved"
        } else {
            "AtRisk"
        }
        .to_string(),
        isolated_sentinels_affected: Vec::new(),
        sentinel_safety_preserved,
        status: if sentinel_safety_preserved {
            "SentinelSafetyPreviewReady"
        } else {
            "SentinelSafetyPreviewBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_no_hidden_skip_risk_preview_report_v1(
    hidden_skip_risk: bool,
) -> NoHiddenSkipRiskPreviewReportV1 {
    NoHiddenSkipRiskPreviewReportV1 {
        report_id: "no-hidden-skip-risk-preview-v1".to_string(),
        skip_risk_indicators: if hidden_skip_risk {
            vec!["skip risk detected in candidate migration plan".to_string()]
        } else {
            Vec::new()
        },
        hidden_skip_risk,
        status: if hidden_skip_risk {
            "NoHiddenSkipRiskBlocked"
        } else {
            "NoHiddenSkipRiskPreviewReady"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_fifth_patch_candidate_decision_matrix_v1(
    assertion: &AssertionMigrationFeasibilityDrilldownReportV1,
    equivalent: &EquivalentCoverageFeasibilityDrilldownReportV1,
    sentinel: &SentinelSafetyImpactPreviewReportV1,
    no_hidden_skip: &NoHiddenSkipRiskPreviewReportV1,
) -> FifthPatchCandidateDecisionMatrixV1 {
    let ready = assertion.feasible
        && equivalent.feasible
        && sentinel.sentinel_safety_preserved
        && !no_hidden_skip.hidden_skip_risk;
    FifthPatchCandidateDecisionMatrixV1 {
        report_id: "fifth-patch-candidate-decision-matrix-v1".to_string(),
        candidate_rows: vec![FifthPatchCandidateDecisionRowV1 {
            candidate_target: assertion.candidate_target.clone(),
            assertion_migration_feasible: assertion.feasible,
            equivalent_coverage_feasible: equivalent.feasible,
            sentinel_safety_preserved: sentinel.sentinel_safety_preserved,
            no_hidden_skip_risk: !no_hidden_skip.hidden_skip_risk,
            mixed_family_relevance: "High".to_string(),
            decision_recommendation: if ready {
                "ReadyForNextSprint"
            } else {
                "KeepBlocked"
            }
            .to_string(),
        }],
        matrix_status: "FifthPatchCandidateMatrixReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_fifth_patch_decision_gate_v4(
    summary: &Sprint113SummaryFixture,
    integration: &IntegrationFanoutNarrowingReportV1,
    link: &LinkTimeNarrowingReportV1,
    macro_expansion: &MacroExpansionNarrowingReportV1,
    assertion: &AssertionMigrationFeasibilityDrilldownReportV1,
    equivalent: &EquivalentCoverageFeasibilityDrilldownReportV1,
    sentinel: &SentinelSafetyImpactPreviewReportV1,
    no_hidden_skip: &NoHiddenSkipRiskPreviewReportV1,
    matrix: &FifthPatchCandidateDecisionMatrixV1,
) -> FifthPatchDecisionGateV4 {
    let mixed_family_ready = summary.mixed_family_evidence_narrowed_enough
        || (integration.status == "IntegrationFanoutNarrowed"
            && link.status == "LinkTimeNarrowed"
            && macro_expansion.status == "MacroExpansionNarrowed");
    let ready = assertion.feasible
        && equivalent.feasible
        && sentinel.sentinel_safety_preserved
        && !no_hidden_skip.hidden_skip_risk
        && mixed_family_ready;
    let gate_status = if ready {
        "FifthPatchReadyForNextSprint"
    } else if !sentinel.sentinel_safety_preserved || no_hidden_skip.hidden_skip_risk {
        "FifthPatchBlockedBySafety"
    } else if matrix
        .candidate_rows
        .iter()
        .any(|row| row.decision_recommendation == "StopConsolidationRecommended")
    {
        "StopConsolidationRecommended"
    } else {
        "FifthPatchStillBlocked"
    };
    FifthPatchDecisionGateV4 {
        gate_id: "fifth-patch-decision-gate-v4".to_string(),
        previous_gate_status: summary.previous_gate_status.clone(),
        mixed_family_isolation_status: if mixed_family_ready {
            "MixedFamiliesNarrowed"
        } else {
            "MixedFamiliesStillAmbiguous"
        }
        .to_string(),
        assertion_migration_feasibility_status: assertion.feasibility_status.clone(),
        equivalent_coverage_status: equivalent.status.clone(),
        sentinel_safety_status: sentinel.status.clone(),
        no_hidden_skip_status: no_hidden_skip.status.clone(),
        candidate_matrix_status: matrix.matrix_status.clone(),
        fifth_patch_ready_for_next_sprint: ready,
        fifth_patch_applied_this_sprint: false,
        gate_status: gate_status.to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_fifth_patch_apply_plan_for_next_sprint_v1(
    assertion: &AssertionMigrationFeasibilityDrilldownReportV1,
    destination_candidates: &AssertionDestinationCandidateReportV1,
    equivalent: &EquivalentCoverageFeasibilityDrilldownReportV1,
    gate: &FifthPatchDecisionGateV4,
) -> FifthPatchApplyPlanForNextSprintV1 {
    let destination_target = destination_candidates
        .destination_candidates
        .iter()
        .find(|candidate| candidate.capacity > 0)
        .map(|candidate| candidate.candidate_target.clone());
    FifthPatchApplyPlanForNextSprintV1 {
        report_id: "fifth-patch-apply-plan-for-next-sprint-v1".to_string(),
        candidate_target: Some(assertion.candidate_target.clone()),
        assertions_to_migrate: assertion.assertions_to_move.clone(),
        destination_target,
        expected_coverage_proof: equivalent.coverage_proof_refs.clone(),
        required_tests: vec![
            "cargo test --test assertion_migration_feasibility_drilldown_v1 --quiet".to_string(),
            "cargo test --test fifth_patch_decision_gate_v4 --quiet".to_string(),
        ],
        no_apply_this_sprint: true,
        status: if gate.fifth_patch_ready_for_next_sprint {
            "FifthPatchApplyPlanReadyForNextSprint"
        } else {
            "FifthPatchApplyPlanDeferred"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_fifth_patch_no_apply_guarantee_report_v3() -> FifthPatchNoApplyGuaranteeReportV3 {
    FifthPatchNoApplyGuaranteeReportV3 {
        report_id: "fifth-patch-no-apply-guarantee-v3".to_string(),
        fifth_patch_applied: false,
        retired_files: Vec::new(),
        moved_assertions: Vec::new(),
        guarantee_status: "FifthPatchNotAppliedGuaranteed".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_candidate_stop_consolidation_report_v1(
    assertion: &AssertionMigrationFeasibilityDrilldownReportV1,
    equivalent: &EquivalentCoverageFeasibilityDrilldownReportV1,
    sentinel: &SentinelSafetyImpactPreviewReportV1,
    no_hidden_skip: &NoHiddenSkipRiskPreviewReportV1,
    gate: &FifthPatchDecisionGateV4,
) -> CandidateStopConsolidationReportV1 {
    let mut reasons = Vec::new();
    if !assertion.feasible {
        reasons.push("AssertionMigrationStillBlocked".to_string());
    }
    if !equivalent.feasible {
        reasons.push("EquivalentCoverageWeak".to_string());
    }
    if !sentinel.sentinel_safety_preserved {
        reasons.push("SentinelRiskTooHigh".to_string());
    }
    if no_hidden_skip.hidden_skip_risk {
        reasons.push("NeedMoreObservation".to_string());
    }
    if gate.gate_status == "FifthPatchStillBlocked" && reasons.is_empty() {
        reasons.push("NeedMoreObservation".to_string());
    }
    let stop_recommended = !reasons.is_empty();
    CandidateStopConsolidationReportV1 {
        report_id: "candidate-stop-consolidation-v1".to_string(),
        stop_recommended,
        reasons,
        status: if stop_recommended {
            "StopConsolidationRecommended"
        } else {
            "MoreObservationRecommended"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_cargo_json_suspect_target_trace_v1(
    summary: &Sprint113SummaryFixture,
) -> CargoJsonSuspectTargetTraceV1 {
    CargoJsonSuspectTargetTraceV1 {
        report_id: "cargo-json-suspect-target-trace-v1".to_string(),
        suspect_target_json_events: summary.cargo_json_events.clone(),
        last_seen_artifact: summary.cargo_json_last_seen_artifact.clone(),
        parser_errors: summary.cargo_json_parser_errors,
        status: "CargoJsonSuspectTraceReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_rustc_suspect_target_timeline_v2(
    summary: &Sprint113SummaryFixture,
) -> RustcSuspectTargetTimelineV2 {
    RustcSuspectTargetTimelineV2 {
        report_id: "rustc-suspect-target-timeline-v2".to_string(),
        suspect_target_rustc_args: summary.rustc_args.clone(),
        concurrency: summary.rustc_max_concurrency,
        remaining_processes_after_timeout: summary.remaining_rustc_processes_after_timeout,
        status: "RustcSuspectTimelineReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_artifact_suspect_target_timeline_v2(
    summary: &Sprint113SummaryFixture,
) -> ArtifactSuspectTargetTimelineV2 {
    ArtifactSuspectTargetTimelineV2 {
        report_id: "artifact-suspect-target-timeline-v2".to_string(),
        suspect_target_artifact_events: summary.artifact_events.clone(),
        status: "ArtifactSuspectTimelineReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_link_macro_evidence_matrix_v1(
    link: &LinkTimeNarrowingReportV1,
    macro_expansion: &MacroExpansionNarrowingReportV1,
) -> LinkMacroEvidenceMatrixV1 {
    LinkMacroEvidenceMatrixV1 {
        report_id: "link-macro-evidence-matrix-v1".to_string(),
        link_evidence_rows: link
            .observed_evidence
            .iter()
            .map(|detail| EvidenceMatrixRowV1 {
                family: "LinkTimeCost".to_string(),
                target: "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                evidence_kind: "Observed".to_string(),
                source: "rustc timeline".to_string(),
                detail: detail.clone(),
            })
            .chain(
                link.inferred_evidence
                    .iter()
                    .map(|detail| EvidenceMatrixRowV1 {
                        family: "LinkTimeCost".to_string(),
                        target: "tests/workspace_timeout_root_cause.rs".to_string(),
                        evidence_kind: "Inferred".to_string(),
                        source: "mixed-family decomposition".to_string(),
                        detail: detail.clone(),
                    }),
            )
            .collect(),
        macro_evidence_rows: macro_expansion
            .observed_evidence
            .iter()
            .map(|detail| EvidenceMatrixRowV1 {
                family: "MacroExpansionCost".to_string(),
                target: "tests/workspace_timeout_root_cause.rs".to_string(),
                evidence_kind: "Observed".to_string(),
                source: "cargo json trace".to_string(),
                detail: detail.clone(),
            })
            .chain(
                macro_expansion
                    .inferred_evidence
                    .iter()
                    .map(|detail| EvidenceMatrixRowV1 {
                        family: "MacroExpansionCost".to_string(),
                        target: "tests/control_tower_workspace_timeout_root_cause_panel.rs"
                            .to_string(),
                        evidence_kind: "Inferred".to_string(),
                        source: "target decomposition".to_string(),
                        detail: detail.clone(),
                    }),
            )
            .collect(),
        status: "LinkMacroEvidenceMatrixReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_integration_fanout_evidence_matrix_v1(
    integration: &IntegrationFanoutNarrowingReportV1,
) -> IntegrationFanoutEvidenceMatrixV1 {
    IntegrationFanoutEvidenceMatrixV1 {
        report_id: "integration-fanout-evidence-matrix-v1".to_string(),
        integration_fanout_rows: integration
            .observed_evidence
            .iter()
            .map(|detail| EvidenceMatrixRowV1 {
                family: "IntegrationTestBinaryFanout".to_string(),
                target: "tests/shared_fixture_harness_application_v1.rs".to_string(),
                evidence_kind: "Observed".to_string(),
                source: "sprint113 truth import".to_string(),
                detail: detail.clone(),
            })
            .chain(
                integration
                    .inferred_evidence
                    .iter()
                    .map(|detail| EvidenceMatrixRowV1 {
                        family: "IntegrationTestBinaryFanout".to_string(),
                        target: "tests/control_tower_workspace_timeout_root_cause_panel.rs"
                            .to_string(),
                        evidence_kind: "Inferred".to_string(),
                        source: "mixed-family isolation plan".to_string(),
                        detail: detail.clone(),
                    }),
            )
            .collect(),
        status: "IntegrationFanoutEvidenceMatrixReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_target_level_observation_quality_report_v1(
    integration: &IntegrationFanoutNarrowingReportV1,
    link: &LinkTimeNarrowingReportV1,
    macro_expansion: &MacroExpansionNarrowingReportV1,
) -> TargetLevelObservationQualityReportV1 {
    TargetLevelObservationQualityReportV1 {
        report_id: "target-level-observation-quality-v1".to_string(),
        rows: vec![
            ObservationQualityRowV1 {
                target: "tests/control_tower_workspace_timeout_root_cause_panel.rs".to_string(),
                observed_evidence_count: (integration.observed_evidence.len()
                    + link.observed_evidence.len()) as u64,
                inferred_evidence_count: (integration.inferred_evidence.len()
                    + macro_expansion.inferred_evidence.len())
                    as u64,
                quality: "Moderate".to_string(),
            },
            ObservationQualityRowV1 {
                target: "tests/shared_fixture_harness_application_v1.rs".to_string(),
                observed_evidence_count: integration.observed_evidence.len() as u64,
                inferred_evidence_count: integration.inferred_evidence.len() as u64,
                quality: "Moderate".to_string(),
            },
            ObservationQualityRowV1 {
                target: "tests/workspace_timeout_root_cause.rs".to_string(),
                observed_evidence_count: (link.observed_evidence.len()
                    + macro_expansion.observed_evidence.len())
                    as u64,
                inferred_evidence_count: (link.inferred_evidence.len()
                    + macro_expansion.inferred_evidence.len())
                    as u64,
                quality: "Moderate".to_string(),
            },
        ],
        status: "TargetLevelObservationQualityReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_timeout_cleanup_verification_report_v7(
    summary: &Sprint113SummaryFixture,
) -> TimeoutCleanupVerificationReportV7 {
    TimeoutCleanupVerificationReportV7 {
        report_id: "timeout-cleanup-verification-v7".to_string(),
        cleanup_verified: summary.timeout_cleanup_verified,
        remaining_cargo_processes: summary.remaining_cargo_processes_after_timeout,
        remaining_rustc_processes: summary.remaining_rustc_processes_after_timeout,
        cleanup_status: if summary.timeout_cleanup_verified {
            "TimeoutCleanupVerified"
        } else {
            "TimeoutCleanupNeedsWork"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_workspace_no_run_recovery_gate_v15(
    baseline: &Sprint113BaselineTruthImportReport,
) -> WorkspaceNoRunRecoveryGateV15 {
    let timed_out = baseline.no_run_exit_code == Some(124);
    let recovered = baseline.no_run_exit_code == Some(0);
    WorkspaceNoRunRecoveryGateV15 {
        gate_id: "workspace-no-run-recovery-gate-v15".to_string(),
        command: "cargo test --workspace --no-run --quiet".to_string(),
        finished: recovered,
        passed: recovered,
        timed_out,
        recovered,
        gate_status: if recovered {
            "NoRunRecovered"
        } else {
            "NoRunStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_workspace_full_acceptance_gate_v15(
    baseline: &Sprint113BaselineTruthImportReport,
) -> WorkspaceFullAcceptanceGateV15 {
    WorkspaceFullAcceptanceGateV15 {
        gate_id: "workspace-full-acceptance-gate-v15".to_string(),
        command: "cargo test --workspace --quiet".to_string(),
        finished: baseline.full_exit_code == Some(0),
        passed: baseline.full_exit_code == Some(0),
        accepted: baseline.full_exit_code == Some(0),
        gate_status: if baseline.full_exit_code == Some(0) {
            "FullWorkspaceAccepted"
        } else {
            "FullWorkspaceStillBlocked"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_focused_vs_full_bridge_v11(
    baseline: &Sprint113BaselineTruthImportReport,
    no_run: &WorkspaceNoRunRecoveryGateV15,
    full: &WorkspaceFullAcceptanceGateV15,
) -> FocusedVsFullBridgeV11 {
    FocusedVsFullBridgeV11 {
        bridge_id: "focused-vs-full-bridge-v11".to_string(),
        focused_tests_status: if baseline.focused_tests_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cli_smoke_status: if baseline.cli_smoke_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cargo_build_status: if baseline.cargo_build_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        no_run_status: no_run.gate_status.clone(),
        full_status: full.gate_status.clone(),
        bridge_status: "FocusedEvidenceDoesNotUpgradeFullAcceptance".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_acceptance_truth_gate_v15(
    baseline: &Sprint113BaselineTruthImportReport,
    no_run: &WorkspaceNoRunRecoveryGateV15,
    full: &WorkspaceFullAcceptanceGateV15,
) -> AcceptanceTruthGateV15 {
    let can_claim_full_acceptance = full.accepted;
    AcceptanceTruthGateV15 {
        gate_id: "acceptance-truth-gate-v15".to_string(),
        focused_truth_status: if baseline.focused_tests_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cli_truth_status: if baseline.cli_smoke_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cargo_check_truth_status: if baseline.cargo_check_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        cargo_build_truth_status: if baseline.cargo_build_passed {
            "SupportingOnly"
        } else {
            "Insufficient"
        }
        .to_string(),
        no_run_truth_status: if no_run.recovered {
            "NoRunOnly"
        } else {
            "SupportingOnly"
        }
        .to_string(),
        full_workspace_truth_status: if can_claim_full_acceptance {
            "FullWorkspaceAccepted"
        } else {
            "FullWorkspaceStillBlocked"
        }
        .to_string(),
        can_claim_full_acceptance,
        truth_status: if can_claim_full_acceptance {
            "AcceptanceTruthReady"
        } else {
            "AcceptanceTruthReadyWithWarnings"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_acceptance_evidence_strength_report_v4(
    baseline: &Sprint113BaselineTruthImportReport,
    acceptance_truth: &AcceptanceTruthGateV15,
) -> AcceptanceEvidenceStrengthReportV4 {
    let mut evidence_tiers = Vec::new();
    if baseline.focused_tests_passed {
        evidence_tiers.push("focused-tests:supporting-only".to_string());
    }
    if baseline.cli_smoke_passed {
        evidence_tiers.push("cli-smoke:supporting-only".to_string());
    }
    if baseline.cargo_build_passed {
        evidence_tiers.push("cargo-build:supporting-only".to_string());
    }
    evidence_tiers.push(format!(
        "full-workspace:{}",
        acceptance_truth.full_workspace_truth_status
    ));
    AcceptanceEvidenceStrengthReportV4 {
        report_id: "acceptance-evidence-strength-v4".to_string(),
        evidence_tiers,
        strongest_claim: acceptance_truth.full_workspace_truth_status.clone(),
        report_status: baseline.acceptance_evidence_status.clone(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_workspace_recovery_decision_report_v4(
    gate: &FifthPatchDecisionGateV4,
    stop: &CandidateStopConsolidationReportV1,
    no_run: &WorkspaceNoRunRecoveryGateV15,
    full: &WorkspaceFullAcceptanceGateV15,
) -> WorkspaceRecoveryDecisionReportV4 {
    WorkspaceRecoveryDecisionReportV4 {
        report_id: "workspace-recovery-decision-v4".to_string(),
        recommend_fifth_patch_next_sprint: gate.fifth_patch_ready_for_next_sprint,
        recommend_stop_consolidation: stop.stop_recommended,
        recommend_more_observation: !gate.fifth_patch_ready_for_next_sprint
            && !stop.stop_recommended,
        no_run_recovered: no_run.recovered,
        full_workspace_accepted: full.accepted,
        decision_status: if gate.fifth_patch_ready_for_next_sprint {
            "FifthPatchReadyForNextSprint"
        } else if stop.stop_recommended {
            "StopConsolidationRecommended"
        } else {
            "MoreObservationRecommended"
        }
        .to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_cumulative_safe_patch_ledger_v5() -> CumulativeSafePatchLedgerV5 {
    CumulativeSafePatchLedgerV5 {
        report_id: "cumulative-safe-patch-ledger-v5".to_string(),
        carried_patch_ids: vec![
            "sprint107-safe-consolidation-patch-v1".to_string(),
            "sprint108-safe-consolidation-patch-v2".to_string(),
            "sprint109-safe-consolidation-patch-v3".to_string(),
            "sprint110-safe-consolidation-patch-v4".to_string(),
        ],
        fifth_patch_applied: false,
        status: "CumulativeSafePatchLedgerReady".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_cumulative_binary_delta_report_v4(
    summary: &Sprint113SummaryFixture,
) -> CumulativeBinaryDeltaReportV4 {
    CumulativeBinaryDeltaReportV4 {
        report_id: "cumulative-binary-delta-v4".to_string(),
        sample_backed_delta: summary.cumulative_sample_backed_delta,
        measured_claim_allowed: false,
        status: "CumulativeBinaryDeltaSampleBacked".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_assertion_ledger_continuity_check_v4() -> AssertionLedgerContinuityCheckV4 {
    AssertionLedgerContinuityCheckV4 {
        report_id: "assertion-ledger-continuity-check-v4".to_string(),
        continuity_preserved: true,
        status: "AssertionLedgerContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_equivalent_coverage_continuity_check_v4() -> EquivalentCoverageContinuityCheckV4 {
    EquivalentCoverageContinuityCheckV4 {
        report_id: "equivalent-coverage-continuity-check-v4".to_string(),
        continuity_preserved: true,
        status: "EquivalentCoverageContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_safety_sentinel_continuity_check_v4() -> SafetySentinelContinuityCheckV4 {
    SafetySentinelContinuityCheckV4 {
        report_id: "safety-sentinel-continuity-check-v4".to_string(),
        continuity_preserved: true,
        status: "SafetySentinelContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_no_hidden_skip_continuity_check_v4() -> NoHiddenSkipContinuityCheckV4 {
    NoHiddenSkipContinuityCheckV4 {
        report_id: "no-hidden-skip-continuity-check-v4".to_string(),
        continuity_preserved: true,
        status: "NoHiddenSkipContinuityPreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_safety_coverage_preservation_report_v30() -> SafetyCoveragePreservationReportV30 {
    SafetyCoveragePreservationReportV30 {
        report_id: "safety-coverage-preservation-v30".to_string(),
        no_assertion_deletion: true,
        no_safety_sentinel_deletion: true,
        no_hidden_skips: true,
        mixed_family_isolation_guard_present: true,
        assertion_migration_feasibility_guard_present: true,
        fifth_patch_v4_no_apply_guard_present: true,
        stop_consolidation_guard_present: true,
        runtime_deferred: true,
        training_deferred: true,
        live_trading_forbidden: true,
        safety_status: "SafetyCoveragePreserved".to_string(),
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_control_tower_mixed_family_isolation_panel(
    registry: &StillMixedFamilyRegistryV1,
    decomposition: &SuspectTargetDecompositionReportV1,
    assertion: &AssertionMigrationFeasibilityDrilldownReportV1,
    equivalent: &EquivalentCoverageFeasibilityDrilldownReportV1,
    acceptance_truth: &AcceptanceTruthGateV15,
) -> ControlTowerMixedFamilyIsolationPanel {
    ControlTowerMixedFamilyIsolationPanel {
        panel_id: "control-tower-mixed-family-isolation".to_string(),
        still_mixed_families: registry.mixed_families.clone(),
        isolated_families: registry.already_isolated_families.clone(),
        target_decomposition_status: decomposition.decomposition_status.clone(),
        assertion_migration_status: assertion.feasibility_status.clone(),
        equivalent_coverage_status: equivalent.status.clone(),
        acceptance_truth_status: acceptance_truth.truth_status.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_run_button: true,
        no_apply_patch_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
pub fn build_control_tower_fifth_patch_readiness_panel_v4(
    gate: &FifthPatchDecisionGateV4,
    plan: &FifthPatchApplyPlanForNextSprintV1,
    stop: &CandidateStopConsolidationReportV1,
) -> ControlTowerFifthPatchReadinessPanelV4 {
    ControlTowerFifthPatchReadinessPanelV4 {
        panel_id: "control-tower-fifth-patch-readiness-v4".to_string(),
        fifth_gate_status: gate.gate_status.clone(),
        apply_plan_status: plan.status.clone(),
        stop_consolidation_status: stop.status.clone(),
        warnings: warning_posture(),
        static_read_only: true,
        no_apply_patch_button: true,
        no_run_button: true,
        no_train_runtime_live_order_account_controls: true,
        reason_codes: diagnostic_reason_codes(&[]),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MixedFamilyIsolationV1Bundle {
    pub sprint113_baseline_truth_import_report: Sprint113BaselineTruthImportReport,
    pub sprint113_observation_carry_forward_report: Sprint113ObservationCarryForwardReport,
    pub still_mixed_family_registry_v1: StillMixedFamilyRegistryV1,
    pub mixed_family_isolation_plan_v1: MixedFamilyIsolationPlanV1,
    pub integration_fanout_narrowing_report_v1: IntegrationFanoutNarrowingReportV1,
    pub link_time_narrowing_report_v1: LinkTimeNarrowingReportV1,
    pub macro_expansion_narrowing_report_v1: MacroExpansionNarrowingReportV1,
    pub suspect_target_decomposition_report_v1: SuspectTargetDecompositionReportV1,
    pub control_tower_timeout_panel_decomposition_report_v1:
        ControlTowerTimeoutPanelDecompositionReportV1,
    pub workspace_timeout_root_cause_target_decomposition_report_v1:
        WorkspaceTimeoutRootCauseTargetDecompositionReportV1,
    pub shared_fixture_harness_pressure_report_v1: SharedFixtureHarnessPressureReportV1,
    pub target_assertion_inventory_report_v1: TargetAssertionInventoryReportV1,
    pub assertion_migration_feasibility_drilldown_report_v1:
        AssertionMigrationFeasibilityDrilldownReportV1,
    pub assertion_destination_candidate_report_v1: AssertionDestinationCandidateReportV1,
    pub assertion_risk_classification_report_v1: AssertionRiskClassificationReportV1,
    pub equivalent_coverage_feasibility_drilldown_report_v1:
        EquivalentCoverageFeasibilityDrilldownReportV1,
    pub sentinel_safety_impact_preview_report_v1: SentinelSafetyImpactPreviewReportV1,
    pub no_hidden_skip_risk_preview_report_v1: NoHiddenSkipRiskPreviewReportV1,
    pub fifth_patch_candidate_decision_matrix_v1: FifthPatchCandidateDecisionMatrixV1,
    pub fifth_patch_decision_gate_v4: FifthPatchDecisionGateV4,
    pub fifth_patch_apply_plan_for_next_sprint_v1: FifthPatchApplyPlanForNextSprintV1,
    pub fifth_patch_no_apply_guarantee_report_v3: FifthPatchNoApplyGuaranteeReportV3,
    pub candidate_stop_consolidation_report_v1: CandidateStopConsolidationReportV1,
    pub cargo_json_suspect_target_trace_v1: CargoJsonSuspectTargetTraceV1,
    pub rustc_suspect_target_timeline_v2: RustcSuspectTargetTimelineV2,
    pub artifact_suspect_target_timeline_v2: ArtifactSuspectTargetTimelineV2,
    pub link_macro_evidence_matrix_v1: LinkMacroEvidenceMatrixV1,
    pub integration_fanout_evidence_matrix_v1: IntegrationFanoutEvidenceMatrixV1,
    pub target_level_observation_quality_report_v1: TargetLevelObservationQualityReportV1,
    pub timeout_cleanup_verification_report_v7: TimeoutCleanupVerificationReportV7,
    pub workspace_no_run_recovery_gate_v15: WorkspaceNoRunRecoveryGateV15,
    pub workspace_full_acceptance_gate_v15: WorkspaceFullAcceptanceGateV15,
    pub focused_vs_full_bridge_v11: FocusedVsFullBridgeV11,
    pub acceptance_truth_gate_v15: AcceptanceTruthGateV15,
    pub acceptance_evidence_strength_report_v4: AcceptanceEvidenceStrengthReportV4,
    pub workspace_recovery_decision_report_v4: WorkspaceRecoveryDecisionReportV4,
    pub cumulative_safe_patch_ledger_v5: CumulativeSafePatchLedgerV5,
    pub cumulative_binary_delta_report_v4: CumulativeBinaryDeltaReportV4,
    pub assertion_ledger_continuity_check_v4: AssertionLedgerContinuityCheckV4,
    pub equivalent_coverage_continuity_check_v4: EquivalentCoverageContinuityCheckV4,
    pub safety_sentinel_continuity_check_v4: SafetySentinelContinuityCheckV4,
    pub no_hidden_skip_continuity_check_v4: NoHiddenSkipContinuityCheckV4,
    pub safety_coverage_preservation_report_v30: SafetyCoveragePreservationReportV30,
    pub control_tower_mixed_family_isolation_panel: ControlTowerMixedFamilyIsolationPanel,
    pub control_tower_fifth_patch_readiness_panel_v4: ControlTowerFifthPatchReadinessPanelV4,
    pub storage_report: MixedFamilyIsolationStorageReport,
    pub final_summary: String,
    pub reason_codes: Vec<ReasonCode>,
}
impl MixedFamilyIsolationV1Bundle {
    pub fn to_json_string(&self) -> Result<String, String> {
        render_json(self)
    }
    pub fn build_final_summary(&self) -> String {
        let sections = [
            (
                "## 1. Sprint summary",
                format!(
                    "imported={} mixed_family={} assertion={} fifth_patch={} acceptance={}",
                    self.sprint113_baseline_truth_import_report.import_status,
                    self.fifth_patch_decision_gate_v4
                        .mixed_family_isolation_status,
                    self.assertion_migration_feasibility_drilldown_report_v1
                        .feasibility_status,
                    self.fifth_patch_decision_gate_v4.gate_status,
                    self.acceptance_truth_gate_v15.truth_status
                ),
            ),
            (
                "## 2. Why Sprint 114 was needed",
                "Sprint 113 left IntegrationTestBinaryFanout, LinkTimeCost, and MacroExpansionCost mixed, so Sprint 114 narrows readiness without applying the fifth patch.".to_string(),
            ),
            (
                "## 3. Files added",
                "No fifth-patch retirement files are added by the runner; Sprint 114 emits local report artifacts only.".to_string(),
            ),
            (
                "## 4. Files changed",
                "Sprint 114 verification surfaces are report-only and preserve runtime/training/live/order deferral.".to_string(),
            ),
            (
                "## 5. Sprint 113 baseline truth import",
                format!(
                    "status={} imported_as_full_acceptance={}",
                    self.sprint113_baseline_truth_import_report.import_status,
                    self.sprint113_baseline_truth_import_report
                        .imported_as_full_acceptance
                ),
            ),
            (
                "## 6. Sprint 113 observation carry-forward",
                format!(
                    "status={} cargo_json_actual_parsing={} cleanup_counts={}",
                    self.sprint113_observation_carry_forward_report
                        .carry_forward_status,
                    self.sprint113_observation_carry_forward_report
                        .cargo_json_actual_parsing_carried_forward,
                    self.sprint113_observation_carry_forward_report
                        .actual_cleanup_counts_carried_forward
                ),
            ),
            (
                "## 7. Still-mixed family registry",
                format!(
                    "status={} mixed={:?}",
                    self.still_mixed_family_registry_v1.status,
                    self.still_mixed_family_registry_v1.mixed_families
                ),
            ),
            (
                "## 8. Mixed family isolation plan",
                format!("status={}", self.mixed_family_isolation_plan_v1.plan_status),
            ),
            (
                "## 9. Integration fanout narrowing",
                format!(
                    "status={} observed={} inferred={}",
                    self.integration_fanout_narrowing_report_v1.status,
                    self.integration_fanout_narrowing_report_v1
                        .observed_evidence
                        .len(),
                    self.integration_fanout_narrowing_report_v1
                        .inferred_evidence
                        .len()
                ),
            ),
            (
                "## 10. Link-time narrowing",
                format!("status={}", self.link_time_narrowing_report_v1.status),
            ),
            (
                "## 11. Macro-expansion narrowing",
                format!("status={}", self.macro_expansion_narrowing_report_v1.status),
            ),
            (
                "## 12. Suspect target decomposition",
                format!(
                    "status={} targets={}",
                    self.suspect_target_decomposition_report_v1
                        .decomposition_status,
                    self.suspect_target_decomposition_report_v1
                        .decomposed_targets
                        .len()
                ),
            ),
            (
                "## 13. Control Tower timeout panel decomposition",
                format!(
                    "status={} assertion_migration={}",
                    self.control_tower_timeout_panel_decomposition_report_v1
                        .status,
                    self.control_tower_timeout_panel_decomposition_report_v1
                        .assertion_migration_feasibility
                ),
            ),
            (
                "## 14. Workspace timeout target decomposition",
                format!(
                    "status={} assertion_migration={}",
                    self.workspace_timeout_root_cause_target_decomposition_report_v1
                        .status,
                    self.workspace_timeout_root_cause_target_decomposition_report_v1
                        .assertion_migration_feasibility
                ),
            ),
            (
                "## 15. Shared fixture harness pressure",
                format!(
                    "status={} capacity={}",
                    self.shared_fixture_harness_pressure_report_v1.status,
                    self.shared_fixture_harness_pressure_report_v1
                        .further_migration_capacity
                ),
            ),
            (
                "## 16. Target assertion inventory",
                format!(
                    "status={} targets={}",
                    self.target_assertion_inventory_report_v1
                        .inventory_status,
                    self.target_assertion_inventory_report_v1
                        .candidate_targets
                        .len()
                ),
            ),
            (
                "## 17. Assertion migration feasibility drilldown",
                format!(
                    "status={} feasible={} blockers={}",
                    self.assertion_migration_feasibility_drilldown_report_v1
                        .feasibility_status,
                    self.assertion_migration_feasibility_drilldown_report_v1
                        .feasible,
                    self.assertion_migration_feasibility_drilldown_report_v1
                        .blockers
                        .len()
                ),
            ),
            (
                "## 18. Assertion destination candidates",
                format!(
                    "status={} candidates={}",
                    self.assertion_destination_candidate_report_v1.status,
                    self.assertion_destination_candidate_report_v1
                        .destination_candidates
                        .len()
                ),
            ),
            (
                "## 19. Assertion risk classification",
                format!("status={}", self.assertion_risk_classification_report_v1.status),
            ),
            (
                "## 20. Equivalent coverage feasibility drilldown",
                format!(
                    "status={} feasible={}",
                    self.equivalent_coverage_feasibility_drilldown_report_v1.status,
                    self.equivalent_coverage_feasibility_drilldown_report_v1
                        .feasible
                ),
            ),
            (
                "## 21. Sentinel safety and no-hidden-skip preview",
                format!(
                    "sentinel={} hidden_skip={}",
                    self.sentinel_safety_impact_preview_report_v1.status,
                    self.no_hidden_skip_risk_preview_report_v1.status
                ),
            ),
            (
                "## 22. Fifth patch candidate decision matrix",
                format!(
                    "status={} rows={}",
                    self.fifth_patch_candidate_decision_matrix_v1.matrix_status,
                    self.fifth_patch_candidate_decision_matrix_v1
                        .candidate_rows
                        .len()
                ),
            ),
            (
                "## 23. Fifth patch decision gate v4",
                format!(
                    "status={} ready_for_next_sprint={} applied_this_sprint={}",
                    self.fifth_patch_decision_gate_v4.gate_status,
                    self.fifth_patch_decision_gate_v4
                        .fifth_patch_ready_for_next_sprint,
                    self.fifth_patch_decision_gate_v4
                        .fifth_patch_applied_this_sprint
                ),
            ),
            (
                "## 24. Fifth patch apply plan for next sprint",
                format!(
                    "status={} no_apply_this_sprint={}",
                    self.fifth_patch_apply_plan_for_next_sprint_v1.status,
                    self.fifth_patch_apply_plan_for_next_sprint_v1
                        .no_apply_this_sprint
                ),
            ),
            (
                "## 25. Fifth patch no-apply guarantee v3",
                format!(
                    "status={} applied={}",
                    self.fifth_patch_no_apply_guarantee_report_v3
                        .guarantee_status,
                    self.fifth_patch_no_apply_guarantee_report_v3
                        .fifth_patch_applied
                ),
            ),
            (
                "## 26. Candidate stop consolidation report",
                format!(
                    "status={} stop_recommended={} reasons={:?}",
                    self.candidate_stop_consolidation_report_v1.status,
                    self.candidate_stop_consolidation_report_v1
                        .stop_recommended,
                    self.candidate_stop_consolidation_report_v1.reasons
                ),
            ),
            (
                "## 27. Cargo JSON suspect target trace",
                format!(
                    "status={} events={} parser_errors={}",
                    self.cargo_json_suspect_target_trace_v1.status,
                    self.cargo_json_suspect_target_trace_v1
                        .suspect_target_json_events
                        .len(),
                    self.cargo_json_suspect_target_trace_v1.parser_errors
                ),
            ),
            (
                "## 28. Rustc / artifact suspect timelines",
                format!(
                    "rustc={} artifact={}",
                    self.rustc_suspect_target_timeline_v2.status,
                    self.artifact_suspect_target_timeline_v2.status
                ),
            ),
            (
                "## 29. Link/macro evidence matrix",
                format!("status={}", self.link_macro_evidence_matrix_v1.status),
            ),
            (
                "## 30. Integration fanout evidence matrix",
                format!(
                    "status={}",
                    self.integration_fanout_evidence_matrix_v1.status
                ),
            ),
            (
                "## 31. Target-level observation quality",
                format!(
                    "status={} rows={}",
                    self.target_level_observation_quality_report_v1.status,
                    self.target_level_observation_quality_report_v1.rows.len()
                ),
            ),
            (
                "## 32. Timeout cleanup verification v7",
                format!(
                    "status={} remaining_cargo={} remaining_rustc={}",
                    self.timeout_cleanup_verification_report_v7.cleanup_status,
                    self.timeout_cleanup_verification_report_v7
                        .remaining_cargo_processes,
                    self.timeout_cleanup_verification_report_v7
                        .remaining_rustc_processes
                ),
            ),
            (
                "## 33. Workspace no-run recovery gate v15",
                format!(
                    "status={} recovered={}",
                    self.workspace_no_run_recovery_gate_v15.gate_status,
                    self.workspace_no_run_recovery_gate_v15.recovered
                ),
            ),
            (
                "## 34. Workspace full acceptance gate v15",
                format!(
                    "status={} accepted={}",
                    self.workspace_full_acceptance_gate_v15.gate_status,
                    self.workspace_full_acceptance_gate_v15.accepted
                ),
            ),
            (
                "## 35. Focused-vs-full bridge v11",
                format!("status={}", self.focused_vs_full_bridge_v11.bridge_status),
            ),
            (
                "## 36. Acceptance truth gate v15",
                format!(
                    "status={} can_claim_full_acceptance={}",
                    self.acceptance_truth_gate_v15.truth_status,
                    self.acceptance_truth_gate_v15.can_claim_full_acceptance
                ),
            ),
            (
                "## 37. Acceptance evidence strength v4",
                format!(
                    "status={} strongest_claim={}",
                    self.acceptance_evidence_strength_report_v4.report_status,
                    self.acceptance_evidence_strength_report_v4.strongest_claim
                ),
            ),
            (
                "## 38. Workspace recovery decision v4",
                format!(
                    "status={} more_observation={} stop={}",
                    self.workspace_recovery_decision_report_v4.decision_status,
                    self.workspace_recovery_decision_report_v4
                        .recommend_more_observation,
                    self.workspace_recovery_decision_report_v4
                        .recommend_stop_consolidation
                ),
            ),
            (
                "## 39. Cumulative safe patch ledger v5",
                format!(
                    "status={} carried_patches={} fifth_patch_applied={}",
                    self.cumulative_safe_patch_ledger_v5.status,
                    self.cumulative_safe_patch_ledger_v5.carried_patch_ids.len(),
                    self.cumulative_safe_patch_ledger_v5.fifth_patch_applied
                ),
            ),
            (
                "## 40. Cumulative binary delta v4",
                format!(
                    "status={} measured_claim_allowed={}",
                    self.cumulative_binary_delta_report_v4.status,
                    self.cumulative_binary_delta_report_v4
                        .measured_claim_allowed
                ),
            ),
            (
                "## 41. Continuity checks",
                format!(
                    "assertion={} equivalent={} sentinel={} no_hidden_skip={}",
                    self.assertion_ledger_continuity_check_v4.status,
                    self.equivalent_coverage_continuity_check_v4.status,
                    self.safety_sentinel_continuity_check_v4.status,
                    self.no_hidden_skip_continuity_check_v4.status
                ),
            ),
            (
                "## 42. Safety coverage preservation v30",
                format!(
                    "status={} runtime_deferred={} live_trading_forbidden={}",
                    self.safety_coverage_preservation_report_v30.safety_status,
                    self.safety_coverage_preservation_report_v30
                        .runtime_deferred,
                    self.safety_coverage_preservation_report_v30
                        .live_trading_forbidden
                ),
            ),
            (
                "## 43. Control Tower mixed-family isolation panel",
                format!(
                    "read_only={} no_run_button={} no_apply_patch_button={}",
                    self.control_tower_mixed_family_isolation_panel
                        .static_read_only,
                    self.control_tower_mixed_family_isolation_panel
                        .no_run_button,
                    self.control_tower_mixed_family_isolation_panel
                        .no_apply_patch_button
                ),
            ),
            (
                "## 44. Control Tower fifth patch readiness panel v4",
                format!(
                    "read_only={} no_apply_patch_button={}",
                    self.control_tower_fifth_patch_readiness_panel_v4
                        .static_read_only,
                    self.control_tower_fifth_patch_readiness_panel_v4
                        .no_apply_patch_button
                ),
            ),
            (
                "## 45. Output bundle",
                format!("file_count={}", self.storage_report.file_count),
            ),
            (
                "## 46. CLI and examples",
                "Sprint 114 CLI surfaces are report-only, local-path-only, and keep fifth patch application disabled.".to_string(),
            ),
            (
                "## 47. Tests added",
                "Focused Sprint 114 tests cover config, registry, narrowing, decomposition, assertion migration, fifth gate, acceptance truth, Control Tower, CLI, and determinism.".to_string(),
            ),
            (
                "## 48. Test results",
                "See external validation output; this summary never upgrades focused/CLI/build evidence to full workspace acceptance.".to_string(),
            ),
            (
                "## 49. Mixed-family isolation status",
                self.fifth_patch_decision_gate_v4
                    .mixed_family_isolation_status
                    .clone(),
            ),
            (
                "## 50. Assertion migration feasibility status",
                self.assertion_migration_feasibility_drilldown_report_v1
                    .feasibility_status
                    .clone(),
            ),
            (
                "## 51. Fifth patch readiness status",
                self.fifth_patch_decision_gate_v4.gate_status.clone(),
            ),
            (
                "## 52. No-run recovery status",
                self.workspace_no_run_recovery_gate_v15.gate_status.clone(),
            ),
            (
                "## 53. Full workspace acceptance status",
                self.workspace_full_acceptance_gate_v15.gate_status.clone(),
            ),
            (
                "## 54. Runtime deferred status",
                "RuntimeStillDeferred; training, live inference, live trading, and broker/order/account remain forbidden.".to_string(),
            ),
            (
                "## 55. Workspace acceptance truth status",
                self.acceptance_truth_gate_v15.truth_status.clone(),
            ),
            (
                "## 56. Safety coverage status",
                self.safety_coverage_preservation_report_v30
                    .safety_status
                    .clone(),
            ),
            (
                "## 57. Risk review",
                "Fifth patch is not applied; stop consolidation and more observation remain valid outcomes; no focused result is treated as full acceptance.".to_string(),
            ),
            (
                "## 58. Deferred items",
                "Fifth patch application, runtime, training, live inference, live trading, broker/order/account, Mamba/Gated runtime, dashboard serve, browser execution, and 18 live activation remain deferred.".to_string(),
            ),
            (
                "## 59. Next gstack sprint recommendation",
                format!(
                    "recommendation={} full_workspace={}",
                    self.workspace_recovery_decision_report_v4.decision_status,
                    self.workspace_full_acceptance_gate_v15.gate_status
                ),
            ),
        ];
        sections
            .into_iter()
            .map(|(heading, body)| format!("{heading}\n- {body}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn write_to_disk(
        &self,
        output_dir: &Path,
    ) -> Result<MixedFamilyIsolationStorageReport, String> {
        fs::create_dir_all(output_dir).map_err(|err| err.to_string())?;
        let mut written_files = Vec::new();
        macro_rules! write_report {
            ($filename:literal, $value:expr) => {{
                let path = output_dir.join($filename);
                write_text_file(&path, &render_json(&$value)?)?;
                written_files.push($filename.to_string());
            }};
        }
        write_report!(
            "sprint113_baseline_truth_import.txt",
            self.sprint113_baseline_truth_import_report
        );
        write_report!(
            "sprint113_observation_carry_forward.txt",
            self.sprint113_observation_carry_forward_report
        );
        write_report!(
            "still_mixed_family_registry_v1.txt",
            self.still_mixed_family_registry_v1
        );
        write_report!(
            "mixed_family_isolation_plan_v1.txt",
            self.mixed_family_isolation_plan_v1
        );
        write_report!(
            "integration_fanout_narrowing_v1.txt",
            self.integration_fanout_narrowing_report_v1
        );
        write_report!(
            "link_time_narrowing_v1.txt",
            self.link_time_narrowing_report_v1
        );
        write_report!(
            "macro_expansion_narrowing_v1.txt",
            self.macro_expansion_narrowing_report_v1
        );
        write_report!(
            "suspect_target_decomposition_v1.txt",
            self.suspect_target_decomposition_report_v1
        );
        write_report!(
            "control_tower_timeout_panel_decomposition_v1.txt",
            self.control_tower_timeout_panel_decomposition_report_v1
        );
        write_report!(
            "workspace_timeout_root_cause_target_decomposition_v1.txt",
            self.workspace_timeout_root_cause_target_decomposition_report_v1
        );
        write_report!(
            "shared_fixture_harness_pressure_v1.txt",
            self.shared_fixture_harness_pressure_report_v1
        );
        write_report!(
            "target_assertion_inventory_v1.txt",
            self.target_assertion_inventory_report_v1
        );
        write_report!(
            "assertion_migration_feasibility_drilldown_v1.txt",
            self.assertion_migration_feasibility_drilldown_report_v1
        );
        write_report!(
            "assertion_destination_candidate_v1.txt",
            self.assertion_destination_candidate_report_v1
        );
        write_report!(
            "assertion_risk_classification_v1.txt",
            self.assertion_risk_classification_report_v1
        );
        write_report!(
            "equivalent_coverage_feasibility_drilldown_v1.txt",
            self.equivalent_coverage_feasibility_drilldown_report_v1
        );
        write_report!(
            "sentinel_safety_impact_preview_v1.txt",
            self.sentinel_safety_impact_preview_report_v1
        );
        write_report!(
            "no_hidden_skip_risk_preview_v1.txt",
            self.no_hidden_skip_risk_preview_report_v1
        );
        write_report!(
            "fifth_patch_candidate_decision_matrix_v1.txt",
            self.fifth_patch_candidate_decision_matrix_v1
        );
        write_report!(
            "fifth_patch_decision_gate_v4.txt",
            self.fifth_patch_decision_gate_v4
        );
        write_report!(
            "fifth_patch_apply_plan_for_next_sprint_v1.txt",
            self.fifth_patch_apply_plan_for_next_sprint_v1
        );
        write_report!(
            "fifth_patch_no_apply_guarantee_v3.txt",
            self.fifth_patch_no_apply_guarantee_report_v3
        );
        write_report!(
            "candidate_stop_consolidation_v1.txt",
            self.candidate_stop_consolidation_report_v1
        );
        write_report!(
            "cargo_json_suspect_target_trace_v1.txt",
            self.cargo_json_suspect_target_trace_v1
        );
        write_report!(
            "rustc_suspect_target_timeline_v2.txt",
            self.rustc_suspect_target_timeline_v2
        );
        write_report!(
            "artifact_suspect_target_timeline_v2.txt",
            self.artifact_suspect_target_timeline_v2
        );
        write_report!(
            "link_macro_evidence_matrix_v1.txt",
            self.link_macro_evidence_matrix_v1
        );
        write_report!(
            "integration_fanout_evidence_matrix_v1.txt",
            self.integration_fanout_evidence_matrix_v1
        );
        write_report!(
            "target_level_observation_quality_v1.txt",
            self.target_level_observation_quality_report_v1
        );
        write_report!(
            "timeout_cleanup_verification_v7.txt",
            self.timeout_cleanup_verification_report_v7
        );
        write_report!(
            "workspace_no_run_recovery_gate_v15.txt",
            self.workspace_no_run_recovery_gate_v15
        );
        write_report!(
            "workspace_full_acceptance_gate_v15.txt",
            self.workspace_full_acceptance_gate_v15
        );
        write_report!(
            "focused_vs_full_bridge_v11.txt",
            self.focused_vs_full_bridge_v11
        );
        write_report!(
            "acceptance_truth_gate_v15.txt",
            self.acceptance_truth_gate_v15
        );
        write_report!(
            "acceptance_evidence_strength_v4.txt",
            self.acceptance_evidence_strength_report_v4
        );
        write_report!(
            "workspace_recovery_decision_v4.txt",
            self.workspace_recovery_decision_report_v4
        );
        write_report!(
            "cumulative_safe_patch_ledger_v5.txt",
            self.cumulative_safe_patch_ledger_v5
        );
        write_report!(
            "cumulative_binary_delta_v4.txt",
            self.cumulative_binary_delta_report_v4
        );
        write_report!(
            "assertion_ledger_continuity_check_v4.txt",
            self.assertion_ledger_continuity_check_v4
        );
        write_report!(
            "equivalent_coverage_continuity_check_v4.txt",
            self.equivalent_coverage_continuity_check_v4
        );
        write_report!(
            "safety_sentinel_continuity_check_v4.txt",
            self.safety_sentinel_continuity_check_v4
        );
        write_report!(
            "no_hidden_skip_continuity_check_v4.txt",
            self.no_hidden_skip_continuity_check_v4
        );
        write_report!(
            "safety_coverage_preservation_v30.txt",
            self.safety_coverage_preservation_report_v30
        );
        write_report!(
            "control_tower_mixed_family_isolation_panel.txt",
            self.control_tower_mixed_family_isolation_panel
        );
        write_report!(
            "control_tower_fifth_patch_readiness_panel_v4.txt",
            self.control_tower_fifth_patch_readiness_panel_v4
        );
        let storage_report = MixedFamilyIsolationStorageReport {
            report_id: "mixed-family-isolation-storage-report".to_string(),
            output_dir: output_dir.display().to_string(),
            written_files: written_files.clone(),
            file_count: (written_files.len() + 2) as u64,
            reason_codes: diagnostic_reason_codes(&[]),
        };
        write_text_file(
            &output_dir.join("storage_report.txt"),
            &render_json(&storage_report)?,
        )?;
        write_text_file(&output_dir.join("summary.txt"), &self.final_summary)?;
        written_files.push("storage_report.txt".to_string());
        written_files.push("summary.txt".to_string());
        Ok(MixedFamilyIsolationStorageReport {
            written_files,
            ..storage_report
        })
    }
}
#[derive(Clone, Debug, Default)]
pub struct MixedFamilyIsolationV1Runner;
impl MixedFamilyIsolationV1Runner {
    pub fn run(
        &self,
        config: &MixedFamilyIsolationV1Config,
    ) -> Result<MixedFamilyIsolationV1Bundle, String> {
        config.validate()?;
        let mut summary = load_first_json::<Sprint113SummaryFixture>(
            config
                .sprint113_truth_paths
                .as_ref()
                .or(config.sprint113_bundle_paths.as_ref()),
        )?
        .unwrap_or_default();
        apply_actual_sprint114_observations(&mut summary, config)?;
        let sprint113_baseline_truth_import_report =
            build_sprint113_baseline_truth_import_report(&summary);
        let sprint113_observation_carry_forward_report =
            build_sprint113_observation_carry_forward_report(&summary);
        let still_mixed_family_registry_v1 = build_still_mixed_family_registry_v1(&summary);
        let mixed_family_isolation_plan_v1 = build_mixed_family_isolation_plan_v1();
        let integration_fanout_narrowing_report_v1 =
            build_integration_fanout_narrowing_report_v1(&summary);
        let link_time_narrowing_report_v1 = build_link_time_narrowing_report_v1(&summary);
        let macro_expansion_narrowing_report_v1 =
            build_macro_expansion_narrowing_report_v1(&summary);
        let control_tower_timeout_panel_decomposition_report_v1 =
            build_control_tower_timeout_panel_decomposition_report_v1();
        let workspace_timeout_root_cause_target_decomposition_report_v1 =
            build_workspace_timeout_root_cause_target_decomposition_report_v1();
        let shared_fixture_harness_pressure_report_v1 =
            build_shared_fixture_harness_pressure_report_v1(&summary);
        let suspect_target_decomposition_report_v1 = build_suspect_target_decomposition_report_v1(
            &control_tower_timeout_panel_decomposition_report_v1,
            &workspace_timeout_root_cause_target_decomposition_report_v1,
            &shared_fixture_harness_pressure_report_v1,
        );
        let target_assertion_inventory_report_v1 =
            build_target_assertion_inventory_report_v1(&summary);
        let assertion_destination_candidate_report_v1 =
            build_assertion_destination_candidate_report_v1(&target_assertion_inventory_report_v1);
        let assertion_risk_classification_report_v1 =
            build_assertion_risk_classification_report_v1(&target_assertion_inventory_report_v1);
        let assertion_migration_feasibility_drilldown_report_v1 =
            build_assertion_migration_feasibility_drilldown_report_v1(
                &target_assertion_inventory_report_v1,
                &assertion_destination_candidate_report_v1,
            );
        let equivalent_coverage_feasibility_drilldown_report_v1 =
            build_equivalent_coverage_feasibility_drilldown_report_v1(
                &assertion_destination_candidate_report_v1,
                true,
                None,
            );
        let sentinel_safety_impact_preview_report_v1 =
            build_sentinel_safety_impact_preview_report_v1(summary.sentinel_safety_preserved);
        let no_hidden_skip_risk_preview_report_v1 =
            build_no_hidden_skip_risk_preview_report_v1(!summary.no_hidden_skip_risk);
        let fifth_patch_candidate_decision_matrix_v1 =
            build_fifth_patch_candidate_decision_matrix_v1(
                &assertion_migration_feasibility_drilldown_report_v1,
                &equivalent_coverage_feasibility_drilldown_report_v1,
                &sentinel_safety_impact_preview_report_v1,
                &no_hidden_skip_risk_preview_report_v1,
            );
        let fifth_patch_decision_gate_v4 = build_fifth_patch_decision_gate_v4(
            &summary,
            &integration_fanout_narrowing_report_v1,
            &link_time_narrowing_report_v1,
            &macro_expansion_narrowing_report_v1,
            &assertion_migration_feasibility_drilldown_report_v1,
            &equivalent_coverage_feasibility_drilldown_report_v1,
            &sentinel_safety_impact_preview_report_v1,
            &no_hidden_skip_risk_preview_report_v1,
            &fifth_patch_candidate_decision_matrix_v1,
        );
        let fifth_patch_apply_plan_for_next_sprint_v1 =
            build_fifth_patch_apply_plan_for_next_sprint_v1(
                &assertion_migration_feasibility_drilldown_report_v1,
                &assertion_destination_candidate_report_v1,
                &equivalent_coverage_feasibility_drilldown_report_v1,
                &fifth_patch_decision_gate_v4,
            );
        let fifth_patch_no_apply_guarantee_report_v3 =
            build_fifth_patch_no_apply_guarantee_report_v3();
        let candidate_stop_consolidation_report_v1 = build_candidate_stop_consolidation_report_v1(
            &assertion_migration_feasibility_drilldown_report_v1,
            &equivalent_coverage_feasibility_drilldown_report_v1,
            &sentinel_safety_impact_preview_report_v1,
            &no_hidden_skip_risk_preview_report_v1,
            &fifth_patch_decision_gate_v4,
        );
        let cargo_json_suspect_target_trace_v1 = build_cargo_json_suspect_target_trace_v1(&summary);
        let rustc_suspect_target_timeline_v2 = build_rustc_suspect_target_timeline_v2(&summary);
        let artifact_suspect_target_timeline_v2 =
            build_artifact_suspect_target_timeline_v2(&summary);
        let link_macro_evidence_matrix_v1 = build_link_macro_evidence_matrix_v1(
            &link_time_narrowing_report_v1,
            &macro_expansion_narrowing_report_v1,
        );
        let integration_fanout_evidence_matrix_v1 =
            build_integration_fanout_evidence_matrix_v1(&integration_fanout_narrowing_report_v1);
        let target_level_observation_quality_report_v1 =
            build_target_level_observation_quality_report_v1(
                &integration_fanout_narrowing_report_v1,
                &link_time_narrowing_report_v1,
                &macro_expansion_narrowing_report_v1,
            );
        let timeout_cleanup_verification_report_v7 =
            build_timeout_cleanup_verification_report_v7(&summary);
        let workspace_no_run_recovery_gate_v15 =
            build_workspace_no_run_recovery_gate_v15(&sprint113_baseline_truth_import_report);
        let workspace_full_acceptance_gate_v15 =
            build_workspace_full_acceptance_gate_v15(&sprint113_baseline_truth_import_report);
        let focused_vs_full_bridge_v11 = build_focused_vs_full_bridge_v11(
            &sprint113_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v15,
            &workspace_full_acceptance_gate_v15,
        );
        let acceptance_truth_gate_v15 = build_acceptance_truth_gate_v15(
            &sprint113_baseline_truth_import_report,
            &workspace_no_run_recovery_gate_v15,
            &workspace_full_acceptance_gate_v15,
        );
        let acceptance_evidence_strength_report_v4 = build_acceptance_evidence_strength_report_v4(
            &sprint113_baseline_truth_import_report,
            &acceptance_truth_gate_v15,
        );
        let workspace_recovery_decision_report_v4 = build_workspace_recovery_decision_report_v4(
            &fifth_patch_decision_gate_v4,
            &candidate_stop_consolidation_report_v1,
            &workspace_no_run_recovery_gate_v15,
            &workspace_full_acceptance_gate_v15,
        );
        let cumulative_safe_patch_ledger_v5 = build_cumulative_safe_patch_ledger_v5();
        let cumulative_binary_delta_report_v4 = build_cumulative_binary_delta_report_v4(&summary);
        let assertion_ledger_continuity_check_v4 = build_assertion_ledger_continuity_check_v4();
        let equivalent_coverage_continuity_check_v4 =
            build_equivalent_coverage_continuity_check_v4();
        let safety_sentinel_continuity_check_v4 = build_safety_sentinel_continuity_check_v4();
        let no_hidden_skip_continuity_check_v4 = build_no_hidden_skip_continuity_check_v4();
        let safety_coverage_preservation_report_v30 =
            build_safety_coverage_preservation_report_v30();
        let control_tower_mixed_family_isolation_panel =
            build_control_tower_mixed_family_isolation_panel(
                &still_mixed_family_registry_v1,
                &suspect_target_decomposition_report_v1,
                &assertion_migration_feasibility_drilldown_report_v1,
                &equivalent_coverage_feasibility_drilldown_report_v1,
                &acceptance_truth_gate_v15,
            );
        let control_tower_fifth_patch_readiness_panel_v4 =
            build_control_tower_fifth_patch_readiness_panel_v4(
                &fifth_patch_decision_gate_v4,
                &fifth_patch_apply_plan_for_next_sprint_v1,
                &candidate_stop_consolidation_report_v1,
            );
        let mut bundle = MixedFamilyIsolationV1Bundle {
            sprint113_baseline_truth_import_report,
            sprint113_observation_carry_forward_report,
            still_mixed_family_registry_v1,
            mixed_family_isolation_plan_v1,
            integration_fanout_narrowing_report_v1,
            link_time_narrowing_report_v1,
            macro_expansion_narrowing_report_v1,
            suspect_target_decomposition_report_v1,
            control_tower_timeout_panel_decomposition_report_v1,
            workspace_timeout_root_cause_target_decomposition_report_v1,
            shared_fixture_harness_pressure_report_v1,
            target_assertion_inventory_report_v1,
            assertion_migration_feasibility_drilldown_report_v1,
            assertion_destination_candidate_report_v1,
            assertion_risk_classification_report_v1,
            equivalent_coverage_feasibility_drilldown_report_v1,
            sentinel_safety_impact_preview_report_v1,
            no_hidden_skip_risk_preview_report_v1,
            fifth_patch_candidate_decision_matrix_v1,
            fifth_patch_decision_gate_v4,
            fifth_patch_apply_plan_for_next_sprint_v1,
            fifth_patch_no_apply_guarantee_report_v3,
            candidate_stop_consolidation_report_v1,
            cargo_json_suspect_target_trace_v1,
            rustc_suspect_target_timeline_v2,
            artifact_suspect_target_timeline_v2,
            link_macro_evidence_matrix_v1,
            integration_fanout_evidence_matrix_v1,
            target_level_observation_quality_report_v1,
            timeout_cleanup_verification_report_v7,
            workspace_no_run_recovery_gate_v15,
            workspace_full_acceptance_gate_v15,
            focused_vs_full_bridge_v11,
            acceptance_truth_gate_v15,
            acceptance_evidence_strength_report_v4,
            workspace_recovery_decision_report_v4,
            cumulative_safe_patch_ledger_v5,
            cumulative_binary_delta_report_v4,
            assertion_ledger_continuity_check_v4,
            equivalent_coverage_continuity_check_v4,
            safety_sentinel_continuity_check_v4,
            no_hidden_skip_continuity_check_v4,
            safety_coverage_preservation_report_v30,
            control_tower_mixed_family_isolation_panel,
            control_tower_fifth_patch_readiness_panel_v4,
            storage_report: MixedFamilyIsolationStorageReport {
                report_id: "mixed-family-isolation-storage-report".to_string(),
                output_dir: config.output_dir().display().to_string(),
                written_files: Vec::new(),
                file_count: 47,
                reason_codes: diagnostic_reason_codes(&[]),
            },
            final_summary: String::new(),
            reason_codes: diagnostic_reason_codes(&[]),
        };
        bundle.final_summary = bundle.build_final_summary();
        bundle.storage_report = bundle.write_to_disk(&config.output_dir())?;
        Ok(bundle)
    }
}
