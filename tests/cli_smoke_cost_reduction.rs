use soma_zero::{
    CliSmokeCostReductionConfig, CliSmokeCostReductionStatus, build_cli_smoke_cost_reduction_report,
};

#[test]
fn cli_smoke_cost_reduction_preserves_required_smoke() {
    let config = CliSmokeCostReductionConfig {
        reduction_id: "cli-smoke".to_string(),
        cli_smoke_tiering_paths: Vec::new(),
        timing_report_paths: Vec::new(),
        output_root: "target/sprint77-cli".to_string(),
        require_required_smoke: true,
        allow_representative_smoke_only_for_sprint: true,
        allow_exhaustive_smoke_only_for_full: true,
        reason_codes: Vec::new(),
    };
    let report = build_cli_smoke_cost_reduction_report(&config, None);
    assert!(!report.required_smoke.is_empty());
    assert!(
        report
            .required_smoke
            .iter()
            .any(|command| command.contains("--help"))
    );
    assert!(!report.moved_to_representative.is_empty());
    assert!(!report.moved_to_exhaustive.is_empty());
    assert_eq!(
        report.reduction_status,
        CliSmokeCostReductionStatus::SmokeCostReduced
    );
    assert_eq!(report, build_cli_smoke_cost_reduction_report(&config, None));
}
