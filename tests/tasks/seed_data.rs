use loco_rs::{boot::run_task, task, testing::prelude::*};
use locos::app::App;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_can_run_seed_data_append() {
    let boot = boot_test::<App>().await.unwrap();

    let vars = task::Vars::from_cli_args(vec![("refresh".to_string(), "false".to_string())]);

    assert!(
        run_task::<App>(&boot.app_context, Some(&"seed_data".to_string()), &vars)
            .await
            .is_ok()
    );
}
