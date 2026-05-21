#[path = "support/sprint58_support.rs"]
mod sprint58_support;

use soma_zero::{OperationalRunbookV2FinalStatus, OperationalRunbookV2Runner};

#[test]
fn operational_runbook_v2_emits_expected_local_sequence() {
    let out = sprint58_support::output_dir("operational-runbook-v2");
    let report = OperationalRunbookV2Runner::default()
        .run(&sprint58_support::runbook_v2_config(&out))
        .expect("runbook");
    assert_eq!(
        report.final_status,
        OperationalRunbookV2FinalStatus::ReadyToRun
    );
    assert_eq!(
        report.ordered_steps.first().map(String::as_str),
        Some("step-01-kis-auth-closure")
    );
    assert!(
        report
            .command_suggestions
            .iter()
            .any(|command| command.contains("control-tower-auto-refresh"))
    );
    assert_eq!(report.blocked_steps, 0);
}
