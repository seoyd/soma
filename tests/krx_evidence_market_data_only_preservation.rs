mod support;

use soma_zero::{KrxEvidenceMarketDataOnlyPreservationStatus, Sprint91KrxEvidenceRecoveryRunner};
use support::sprint69_support as sprint;

#[test]
fn krx_evidence_market_data_only_boundary_stays_explicit() {
    let config = sprint::sprint91_config_from_example(
        "soma_krx_evidence_market_data_only_preservation.toml",
        "krx-market-data-only",
    );
    let report = Sprint91KrxEvidenceRecoveryRunner::default()
        .run_krx_evidence_market_data_only_preservation(&config)
        .expect("report");
    assert_eq!(
        report.market_data_status,
        KrxEvidenceMarketDataOnlyPreservationStatus::MarketDataOnlyPreserved
    );
    assert!(report.market_data_only_scope_preserved);
    assert!(report.no_order_path);
    assert!(report.no_account_path);
    assert!(report.no_balance_path);
    assert!(report.no_holding_path);
    assert!(report.no_orderable_quantity_path);
    assert!(report.no_correction_cancel_path);
    assert!(report.no_broker_execution_path);
}
