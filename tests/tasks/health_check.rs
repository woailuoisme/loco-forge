use loco_rs::{boot::run_task, task, testing::prelude::*};
use locos::app::App;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_run_health_check_all() {
    let boot = boot_test::<App>().await.unwrap();

    run_task::<App>(
        &boot.app_context,
        Some(&"health:check".to_string()),
        &task::Vars::default(),
    )
    .await
    .unwrap();
}

#[tokio::test]
#[serial]
async fn test_can_run_health_check_target_db() {
    let boot = boot_test::<App>().await.unwrap();

    let vars = task::Vars::from_cli_args(vec![("target".to_string(), "db".to_string())]);

    run_task::<App>(&boot.app_context, Some(&"health:check".to_string()), &vars)
        .await
        .unwrap();
}
