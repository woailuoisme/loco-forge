use loco_rs::testing::prelude::*;
use locos::{app::App, common::diagnostics::DiagnosticsHub};

#[tokio::test]
#[serial_test::serial]
async fn test_diagnostics_hub_probes() {
    let boot = boot_test::<App>().await.unwrap();

    let db_health = DiagnosticsHub::check_db(&boot.app_context).await;
    assert!(db_health.is_up());
    assert!(db_health.error.is_none());

    let cache_health = DiagnosticsHub::check_cache(&boot.app_context).await;
    assert!(cache_health.is_up());
    assert_eq!(cache_health.kind, "memory");

    let storage_health = DiagnosticsHub::check_storage(&boot.app_context).await;
    assert!(storage_health.is_up());
    assert_eq!(storage_health.kind, "memory");

    let readiness = DiagnosticsHub::check_readiness(&boot.app_context).await;
    assert!(readiness.is_all_up());
    assert_eq!(readiness.status, "up");
}
